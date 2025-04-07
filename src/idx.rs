use std::cmp::Ordering;
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{Error, ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::{Duration, SystemTime};
use crate::avl::{AVLNode, AVLTree};
use crate::sst;

pub struct IDX {
    path: PathBuf,
    sst: sst::SST,
}


#[allow(dead_code)]
#[derive(Debug)]
pub struct IDXKey {
    key_len: u8,
    key: String,
    offset: u64
}

#[allow(dead_code)]
#[derive(Debug)]
pub struct IDXValue {
    pub key: String,
    pub value: String,
}

impl IDX {
    pub const KEY_LEN: usize = 1;

    fn get_timestamp_from_filename(path: &Path) -> u64 {
        path.file_stem()
            .and_then(|stem| stem.to_str())
            .and_then(|name| name.parse::<u64>().ok())
            .unwrap_or(0)
    }

    pub fn search_key_in_all_files(key: &str) -> Option<IDXValue> {
        let mut idx_files = std::fs::read_dir(".")
            .unwrap()
            .filter_map(|res| res.ok())
            .map(|dir_entry| dir_entry.path())
            .filter_map(|path| {
                if path.extension().map_or(false, |ext| ext == "idx") {
                    Some(path)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        
        idx_files.sort_by(|a, b| {
            let a_timestamp = Self::get_timestamp_from_filename(a);
            let b_timestamp = Self::get_timestamp_from_filename(b);
            b_timestamp.cmp(&a_timestamp)
        });


        let mut value : Option<IDXValue> = None;

        for file in idx_files {
            let idx = Self::from(file).unwrap();
            value = match idx.get_value(&key) {
                Ok(value) => {
                    Some(value)
                },
                Err(_) => continue,
            };
        }

        value
    }

    pub fn new(mut file_name: Option<String>) -> IDX {
        if file_name.is_none() {
            file_name = Some(SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs().to_string());
        } 
        
        let sst_path = format!("{}.sst", file_name.clone().unwrap());
        let idx_path = format!("{}.idx", file_name.unwrap());
        
        let sst = sst::SST::new(Path::new(&sst_path).to_path_buf());
        IDX{path: Path::new(&idx_path).to_path_buf(), sst}
    }
    
    pub fn clear(&mut self) -> Result<(), Error> {
        fs::remove_file(&self.path)?;
        fs::remove_file(&self.sst.path)
    }

    pub fn from(idx_file: PathBuf) -> Result<IDX, Error> {
        let file_name = idx_file.file_stem();
        if file_name.is_none() {
            return Err(Error::new(ErrorKind::Other, "No Filename"));
        }

        let sst_file = Path::new(&format!("{}.sst", file_name.unwrap().to_str().unwrap())).to_path_buf();
        let sst = sst::SST::new(sst_file);
        Ok(IDX{path: idx_file, sst})
    }
    
    pub fn fill_from_avl(&self, tree: &AVLTree) -> Result<(), Error> {
        self.insert_avl_node(tree.root.as_ref().unwrap())?;
        Ok(())
    }

    fn insert_avl_node(&self, node: &AVLNode) -> Result<(), Error> {
        if let Some(left) = &node.left {
            self.insert_avl_node(left)?;
        }

        self.set_key(node.key.as_str(), node.value.as_str())?;
        
        if let Some(right) = &node.right {
            self.insert_avl_node(right)?;
        }

        Ok(())
    }

    fn get_key_size_from_byte_file(&self, file: &mut File) -> Result<u8, Error> {
        /* Extract key size */
        let mut key_len_buf = [0u8; IDX::KEY_LEN];
        file.read_exact(&mut key_len_buf)?;
        Ok(u8::from_le_bytes(key_len_buf))
    }

    fn get_key_from_byte_file(&self, file: &mut File, key_size: usize) -> Result<String, Error> {
        /* Extract key */
        let mut key_buf = vec![0u8; key_size];
        file.read_exact(&mut key_buf)?;
        Ok(String::from_utf8_lossy(&key_buf).to_string())
    }

    pub fn iter(&self) -> Result<IDXIter, Error> {
        let mut file = self.get_file(false)?;
        let position = file.seek(SeekFrom::Start(0))?;
        Ok(IDXIter { idx: self, position })
    }

    fn find_mid(&self, file: &mut File, mut mid: u64) -> Result<u64, Error> {
        /* Fine start of the struct for bin search */
        while mid > 0 {
            file.seek(SeekFrom::Start(mid))?;

            // Read key len
            let key_size = self.get_key_size_from_byte_file(file)?;

            // The key should be less than 256 and more than 0
            if key_size >= 1 && key_size <= u8::MAX {
                let key = match self.get_key_from_byte_file(file, key_size as usize) {
                    Ok(key) => {key}
                    Err(_) => {mid -= 1; continue }
                };

                if key.chars().all(|x| x.is_alphanumeric()) {
                    return Ok(mid);
                }

            }
            mid -= 1;
        }

        Ok(0)
    }

    fn get_file(&self, create: bool) -> Result<File, Error> {
        OpenOptions::new().create(create).read(true).write(true).open(&self.path)
    }

    fn find_offset(&self, key: &str) ->  Result<Option<u64>, Error> {
        /* Find what offset we should get to find a key in sst */
        let mut file = self.get_file(false)?;

        let mut left = 0;
        let mut right = file.metadata()?.len();

        while left < right {
            let mut mid = (left + right) / 2;
            mid = self.find_mid(&mut file, mid)?;

            file.seek(SeekFrom::Start(mid))?;

            let key_size = self.get_key_size_from_byte_file(&mut file)?;
            let found_string = match self.get_key_from_byte_file(&mut file, key_size as usize) {
                Ok(found_string) => found_string,
                Err(_) => return Ok(None)
            };

            match key.cmp(&found_string) {
                Ordering::Less => {
                    right = mid;
                },
                Ordering::Greater => {
                    left = mid + 4 + found_string.len() as u64 + 8;
                },
                Ordering::Equal => {
                    let mut offset_buf = [0u8; 8];
                    file.read_exact(&mut offset_buf)?;
                    return Ok(Some(u64::from_le_bytes(offset_buf)))
                }
            }
        }
        Ok(None)
    }

    pub fn get_value(&self, key: &str) -> Result<IDXValue, Error> {
        if key.len() >= 11 || !key.chars().all(|x| x.is_alphanumeric()) {
            return Err(Error::new(ErrorKind::Other, "Key must be alphanumeric and less than 11 chars"))
        }
        
        let offset = self.find_offset(key)?;

        match offset {
            Some(offset) => {Ok(IDXValue{key: key.to_string(), value: self.sst.get(&key, offset)?})},
            None => {Err(Error::new(ErrorKind::NotFound, "Key not found"))}
        }
    }

    pub fn set_key(&self, key: &str, value: &str) -> Result<IDXKey, Error> {
        if key.len() >= 11 || !key.chars().all(|x| x.is_alphanumeric()) {
            return Err(Error::new(ErrorKind::Other, "Key must be alphanumeric and less than 11 chars"))
        }

        let offset = self.sst.set(key, value)?;

        let mut file = self.get_file(true)?;
        file.seek(SeekFrom::End(0))?;

        file.write_all(&(key.len() as u8).to_le_bytes())?;
        file.write_all(key.as_bytes())?;
        file.write_all(&offset.to_le_bytes())?;

        Ok(IDXKey{key_len: key.len() as u8, key: key.to_string(), offset})

    }
    
    pub fn compaction() {
        loop {
            sleep(Duration::from_secs(1));
            println!("Checking IDX files to compaction");
            
            let mut idx_files = std::fs::read_dir(".")
                .unwrap()
                .filter_map(|res| res.ok())
                .map(|dir_entry| dir_entry.path())
                .filter_map(|path| {
                    if path.extension().map_or(false, |ext| ext == "idx") {
                        Some(path)
                    } else {
                        None
                    }
                })
                .filter_map(|path| {
                    if path.metadata().map_or(false, |m| m.len() < 5 * 1024 * 1024) {  // 20 MB
                        Some(path)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            idx_files.sort_by(|a, b| {
                let a_timestamp = Self::get_timestamp_from_filename(a);
                let b_timestamp = Self::get_timestamp_from_filename(b);
                b_timestamp.cmp(&a_timestamp)
            });
            
            while idx_files.len() > 2 {
                println!("Start compaction");
                dbg!(&idx_files);
                
                let a_idx_path = idx_files.pop().unwrap();
                let b_idx_path = idx_files.pop().unwrap();
                
                let mut tree = AVLTree::new();
                let mut a_idx = IDX::from(a_idx_path.clone()).unwrap();
                let mut b_idx = IDX::from(b_idx_path).unwrap();
                
                println!("Size of files to compact is {} MB", a_idx.sst.get_size().unwrap() + b_idx.sst.get_size().unwrap());
            
                tree.feel_from_idx(&a_idx);
                tree.feel_from_idx(&b_idx);


                let file_stem = a_idx_path.file_stem().unwrap().to_str().unwrap().to_string();
                let new_idx_file_name = file_stem.split("_");
                let new_idx_file_name_str = new_idx_file_name.clone().collect::<Vec<&str>>();

                let new_idx_file_name = if new_idx_file_name_str.len() == 1 {
                    Some(format!("{}_{}", new_idx_file_name_str[0].to_string(), 1))
                } else {
                    let base_name = new_idx_file_name_str[0];
                    let index = new_idx_file_name_str[1].parse::<u32>().unwrap_or(0) + 1;
                    Some(format!("{}_{}", base_name, index))
                };

                let new_idx = IDX::new(new_idx_file_name);
                new_idx.fill_from_avl(&tree).unwrap();

                a_idx.clear().unwrap();
                println!("First idx file was removed > {}", {a_idx.path.to_string_lossy()});
                b_idx.clear().unwrap();
                println!("Second idx file was removed > {}", {b_idx.path.to_string_lossy()});
                
                println!("Compaction complete, new file > {}, with size > {}", new_idx.path.to_string_lossy(),  new_idx.sst.get_size().unwrap());

                dbg!(&idx_files);
            }
        }
    }
}

pub struct IDXIter<'a> {
    idx: &'a IDX,
    position: u64,
}


impl<'a> Iterator for IDXIter<'a> {
    
    type Item = IDXValue;
    
    fn next(&mut self) -> Option<Self::Item> {
        let mut key_len_buf = [0u8; 1];
        let mut file = self.idx.get_file(false).unwrap();
        if file.seek(SeekFrom::Start(self.position)).is_err() {
            return None;
        }

        if file.read_exact(&mut key_len_buf).is_err() {
            return None;
        }

        let key_len = key_len_buf[0] as usize;
        let mut key_buf = vec![0u8; key_len];

        if file.read_exact(&mut key_buf).is_err() {
            return None;
        }

        let mut offset_buf = [0u8; 8];
        if file.read_exact(&mut offset_buf).is_err() {
            return None;
        }
        
        let key = String::from_utf8_lossy(&key_buf).to_string();
        let offset = u64::from_le_bytes(offset_buf);

        self.position += 1 + key_len as u64 + 8;

        Some(IDXValue {
            key: String::from_utf8_lossy(&key_buf).to_string(),
            value: self.idx.sst.get(key.as_str(), offset).unwrap()
        })
    }
}