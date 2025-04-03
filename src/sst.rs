use std::fs::{File, OpenOptions};
use std::io::{Error, Seek, SeekFrom, Read, Write, ErrorKind};
use std::path::{Path, PathBuf};

pub struct SST {
    path: PathBuf,
}

impl SST {
    pub fn new(path: &Path) -> SST {
        SST { path: path.to_path_buf() }
    }

    fn get_file(&self) -> Result<File, Error> {
        // Попытка открыть файл для чтения и записи
        let file = OpenOptions::new().read(true).write(true).open(&self.path);

        // Если файл открылся успешно, возвращаем его
        if file.is_ok() {
            return file
        }

        // Если файл не существует, создаем его и записываем символ "|"
        let mut new_file = match OpenOptions::new().create(true).write(true).read(true).open(&self.path) {
            Ok(new_file) => new_file,
            Err(error) => return Err(Error::new(ErrorKind::Other, error))
        };

        // Записываем символ "|"
        new_file.write_all("|".as_bytes())?;

        // Возвращаем успешно открытый файл
        Ok(new_file)
    }

    fn get_bin_key(&self, file: &mut File) -> Result<String, Error> {
        let mut key_len_buf = [0u8; 4];
        file.read_exact(&mut key_len_buf)?;
        let key_size = u32::from_le_bytes(key_len_buf) as usize;

        let mut key_buf = vec![0u8; key_size];
        file.read_exact(&mut key_buf)?;
        Ok(String::from_utf8_lossy(&key_buf).to_string())
    }
    
    pub fn get(&self, key: &str, offset: u64) -> Result<String, Error> {
        let mut file = self.get_file()?;
        file.seek(SeekFrom::Start(offset))?;
        
        let found_string = self.get_bin_key(&mut file)?;
        if key != found_string {
            return Err(Error::new(ErrorKind::Other, "key not found"));
        }
        
        Ok( self.get_bin_key(&mut file)?)
    }

    pub fn set(&self, key: &str, value: &str) -> Result<u64, Error> {
        let mut file = self.get_file()?;

        file.seek(SeekFrom::End(0))?;
        let offset = file.stream_position()?;

        file.write_all(&(key.len() as u32).to_le_bytes())?;
        file.write_all(key.as_bytes())?;
        file.write_all(&(value.len() as u32).to_le_bytes())?;
        file.write_all(value.as_bytes())?;
        
        Ok(offset)
    }
}