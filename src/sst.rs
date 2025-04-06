use std::fs::{File, OpenOptions};
use std::io::{Error, Seek, SeekFrom, Read, Write, ErrorKind};
use std::path::{PathBuf};
use crate::idx::IDX;

pub struct SST {
    pub path: PathBuf,
}

impl SST {
    const KEY_LEN: usize = IDX::KEY_LEN;
    const VALUE_LEN: usize = 4;
    
    pub fn new(path: PathBuf) -> SST {
        SST { path }
    }

    fn get_file(&self, create: bool) -> Result<File, Error> {
        OpenOptions::new().create(create).read(true).write(true).open(&self.path)
    }

    pub fn get_size(&self) -> Result<f64, Error> {
        /* In MB */
        let size_in_bytes = self.get_file(false)?.metadata()?.len();
        Ok((size_in_bytes as f64 / 1024.0 / 1024.0))
    }

    fn get_key_size_from_byte_file(&self, file: &mut File) -> Result<u8, Error> {
        /* Extract key size */
        let mut key_len_buf = [0u8; SST::KEY_LEN];
        file.read_exact(&mut key_len_buf)?;
        Ok(u8::from_le_bytes(key_len_buf))
    }

    fn get_key_from_byte_file(&self, file: &mut File, key_size: usize) -> Result<String, Error> {
        /* Extract key */
        let mut key_buf = vec![0u8; key_size];
        file.read_exact(&mut key_buf)?;
        Ok(String::from_utf8_lossy(&key_buf).to_string())
    }

    fn get_value_size_from_byte_file(&self, file: &mut File) -> Result<u32, Error> {
        /* Extract key size */
        let mut key_len_buf = [0u8; SST::VALUE_LEN];
        file.read_exact(&mut key_len_buf)?;
        Ok(u32::from_le_bytes(key_len_buf))
    }
    
    pub fn get(&self, key: &str, offset: u64) -> Result<String, Error> {
        let mut file = self.get_file(false)?;
        file.seek(SeekFrom::Start(offset))?;
        
        let key_size = self.get_key_size_from_byte_file(&mut file)?;
        let found_key = self.get_key_from_byte_file(&mut file, key_size as usize)?;
        if key != found_key {
            return Err(Error::new(ErrorKind::Other, "key not found"));
        }
        
        let value_len = self.get_value_size_from_byte_file(&mut file)?;
        Ok(self.get_key_from_byte_file(&mut file, value_len as usize)?)
    }

    pub fn set(&self, key: &str, value: &str) -> Result<u64, Error> {
        let mut file = self.get_file(true)?;

        file.seek(SeekFrom::End(0))?;
        let offset = file.stream_position()?;

        file.write_all(&(key.len() as u8).to_le_bytes())?;
        file.write_all(key.as_bytes())?;
        file.write_all(&(value.len() as u32).to_le_bytes())?;
        file.write_all(value.as_bytes())?;
        
        Ok(offset)
    }
}