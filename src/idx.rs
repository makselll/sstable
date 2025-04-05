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
    pub value: String,
}

impl IDX {
    pub const KEY_LEN: usize = 1;

    fn get_timestamp_from_filename(path: &Path) -> u64 {
        path.file_stem()
            .and_then(|stem| stem.to_str())
            .and_then(|name| name.parse::<u64>().ok())
            .unwrap_or(0)  // Если не удается извлечь timestamp, возвращаем 0
    }

    pub fn search_key_in_all_files(key: &str) -> Option<IDXValue> {
        let mut idx_files = std::fs::read_dir(".")
            .unwrap()
            // Фильтруем все, что не удалось прочитать
            .filter_map(|res| res.ok())
            // Преобразуем элементы в пути
            .map(|dir_entry| dir_entry.path())
            // Оставляем только файлы с расширением .idx
            .filter_map(|path| {
                if path.extension().map_or(false, |ext| ext == "idx") {
                    Some(path)
                } else {
                    None
                }
            })
            // Преобразуем в вектор для сортировки
            .collect::<Vec<_>>();

        // Сортируем файлы по времени, извлекая unix timestamp из имени файла
        idx_files.sort_by(|a, b| {
            let a_timestamp = Self::get_timestamp_from_filename(a);
            let b_timestamp = Self::get_timestamp_from_filename(b);
            b_timestamp.cmp(&a_timestamp)  // Чтобы сортировать от самого нового
        });


        let mut value : Option<IDXValue> = None;

        for file in idx_files {
            let idx = Self::from(file).unwrap();
            value = match idx.get_value(&key) {
                Ok(value) => {
                    dbg!(&value);
                    Some(value)
                },
                Err(_) => continue,
            };
        }

        value
    }

    pub fn new() -> IDX {
        let time_now = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
        
        let sst_path = format!("{}.sst", time_now);
        let idx_path = format!("{}.idx", time_now);
        
        let sst = sst::SST::new(Path::new(&sst_path).to_path_buf());
        IDX{path: Path::new(&idx_path).to_path_buf(), sst}
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
        dbg!(node.key.as_str());

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

                if key.chars().all(|x| x.is_alphanumeric()) {
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
        if key.chars().all(|x| x.is_alphanumeric()) {
            return Err(Error::new(ErrorKind::Other, "Key must be alphanumeric and less than 11 chars"))
        }
        
        let offset = self.find_offset(key)?;

        match offset {
            Some(offset) => {Ok(IDXValue{value: self.sst.get(&key, offset)?})},
            None => {Err(Error::new(ErrorKind::NotFound, "Key not found"))}
        }
    }

    pub fn set_key(&self, key: &str, value: &str) -> Result<IDXKey, Error> {
        if key.chars().all(|x| x.is_alphanumeric()) {
            return Err(Error::new(ErrorKind::Other, "Key must be alphanumeric and less than 11 chars"))
        }

        let offset = self.sst.set(key, value)?;

        let mut file = self.get_file()?;
        file.seek(SeekFrom::End(0))?;

        file.write_all(&(key.len() as u8).to_le_bytes())?;
        file.write_all(key.as_bytes())?;
        file.write_all(&offset.to_le_bytes())?;

        Ok(IDXKey{key_len: key.len() as u8, key: key.to_string(), offset})

    }
}