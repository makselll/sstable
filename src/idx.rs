use std::cmp::Ordering;
use std::fs::{File, OpenOptions};
use std::io::{Error, ErrorKind, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
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
    value: String,
}

impl IDX {
    pub const KEY_LEN: usize = 1;

    pub fn new() -> IDX {
        let time_now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        
        let sst_path = format!("{}.sst", time_now);
        let idx_path = format!("{}.idx", time_now);
        
        let sst = sst::SST::new(Path::new(&sst_path).to_path_buf());
        IDX{path: Path::new(&idx_path).to_path_buf(), sst}
    }
    
    pub fn fill_from_avl(&self, tree: &AVLTree) -> Result<(), Error> {
        self.insert_avl_node(tree.root.as_ref().unwrap())?;
        Ok(())
    }
    
    fn insert_avl_node(&self, node: &AVLNode) -> Result<(), Error> {
        self.set_key(node.key.as_str(), node.value.as_str())?;
        
        if let Some(left) = &node.left {
            self.insert_avl_node(left)?;
        }
    
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

                if key.chars().all(|x| x.is_alphabetic()) {
                    return Ok(mid);
                }

            }
            mid -= 1;
        }

        Ok(0)
    }

    fn get_file(&self) -> Result<File, Error> {
        // Попытка открыть файл для чтения и записи
        return OpenOptions::new().create(true).read(true).write(true).open(&self.path);

    }

    fn find_offset(&self, key: &str) ->  Result<Option<u64>, Error> {
        /* Find what offset we should get to find a key in sst */
        let mut file = self.get_file()?;

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
        let offset = self.find_offset(key)?;

        match offset {
            Some(offset) => {Ok(IDXValue{value: self.sst.get(&key, offset)?})},
            None => {Err(Error::new(ErrorKind::NotFound, "Key not found"))}
        }
    }

    pub fn set_key(&self, key: &str, value: &str) -> Result<IDXKey, Error> {
        let offset = self.sst.set(key, value)?;

        let mut file = self.get_file()?;
        file.seek(SeekFrom::End(0))?;

        file.write_all(&(key.len() as u8).to_le_bytes())?;
        file.write_all(key.as_bytes())?;
        file.write_all(&offset.to_le_bytes())?;

        Ok(IDXKey{key_len: key.len() as u8, key: key.to_string(), offset})

    }
}