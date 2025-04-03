use std::cmp::Ordering;
use std::fs::{File, OpenOptions};
use std::path::{Path, PathBuf};
use std::env;
use std::io::{Error, ErrorKind, Read, Seek, SeekFrom, Write};

mod sst;

struct IDX {
    path: PathBuf,
    sst: sst::SST,
    key_max_len: usize,
}


#[allow(dead_code)]
#[derive(Debug)]
struct IDXKey {
    key_len: u8,
    key: String,
    offset: u64
}

#[allow(dead_code)]
#[derive(Debug)]
struct IDXValue {
    value: String,
}

impl IDX {
    fn new(path: &Path) -> IDX {
        let sst = sst::SST::new(Path::new("data.sst"));
        IDX{path: path.to_path_buf(), sst, key_max_len: 255}
    }

    fn get_bin_key(&self, file: &mut File) -> Result<String, Error> {
        let mut key_len_buf = [0u8; 4];
        file.read_exact(&mut key_len_buf)?;
        let key_size = u32::from_le_bytes(key_len_buf) as usize;

        let mut key_buf = vec![0u8; key_size];
        file.read_exact(&mut key_buf)?;
        Ok(String::from_utf8_lossy(&key_buf).to_string())
    }
    
    fn find_mid(&self, file: &mut File, mut mid: u64) -> Result<u64, Error> {
        while mid > 0 {
            file.seek(SeekFrom::Start(mid))?;

            // Читаем длину ключа
            let mut key_len_buf = [0u8; 4];
            file.read_exact(&mut key_len_buf)?;
            let key_len = u32::from_le_bytes(key_len_buf);
            
            // 
            if key_len >= 1 && key_len <= 11 {
                let mut key_buf = vec![0u8; key_len as usize];
                match file.read_exact(&mut key_buf) {
                    Ok(_) => {}
                    Err(_) => {mid -= 1; continue }
                };

                let key = String::from_utf8_lossy(&key_buf);

                if key.chars().all(|x| x.is_alphabetic()) {
                    return Ok(mid);
                }

            }

            mid -= 1;

        }

        Ok(0) // Если не нашли
    }

    fn get_file(&self) -> Result<File, Error> {
        // Попытка открыть файл для чтения и записи
        return OpenOptions::new().create(true).read(true).write(true).open(&self.path);

    }

    fn find_offset(&self, key: &str) ->  Result<Option<u64>, Error> {
        let mut file = self.get_file()?;
        
        let mut left = 0;
        let mut right = file.metadata()?.len();
        
        while left < right {
            let mut mid = (left + right) / 2;
            mid = self.find_mid(&mut file, mid)?;
            
            file.seek(SeekFrom::Start(mid))?;

            let found_string = match self.get_bin_key(&mut file) {
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

    fn get_value(&self, key: &str) -> Result<IDXValue, Error> {
        let offset = self.find_offset(key)?;

        match offset {
            Some(offset) => {Ok(IDXValue{value: self.sst.get(&key, offset)?})},
            None => {Err(Error::new(ErrorKind::NotFound, "Key not found"))}
        }
    }

    fn set_key(&self, key: &str, value: &str) -> Result<IDXKey, Error> {
        let offset = self.sst.set(key, value)?;

        let mut file = self.get_file()?;
        file.seek(SeekFrom::End(0))?;
    
        file.write_all(&(key.len() as u32).to_le_bytes())?;
        file.write_all(key.as_bytes())?;
        file.write_all(&offset.to_le_bytes())?;

        Ok(IDXKey{key_len: key.len() as u8, key: key.to_string(), offset})

    }
}



fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() <= 1 {
        panic!("No arguments provided, Use 'set' or 'get'");
    }


    if !vec!["set", "get"].contains(&args[1].as_str()) {
        panic!("Invalid arguments! Use 'set' or 'get'");
    }

    if &args[1] == "set" && args.len() != 4 {
        panic!("Invalid arguments! Use set 'key' 'value'");
    } else if &args[1] == "get" && args.len() != 3 {
        panic!("Invalid arguments! Use get 'key'");
    }

    let idx = IDX::new(&Path::new("map.idx"));

    
    if args[2].len() > idx.key_max_len || !args[2].chars().all(|x| x.is_alphabetic()) {
        panic!("Key must be alphabetic and less then 11 chars");
    }

    if &args[1] == "set" {
        match idx.set_key(args[2].as_str(), args[3].as_str()) {
            Ok(key) => println!("Key set {:?}", key),
            Err(e) => panic!("{}", e),
        }
    } else if &args[1] == "get" {
        match idx.get_value(args[2].as_str()) {
            Ok(value) => println!("Value get {:?}", value),
            Err(e) => panic!("{}", e),
        };

    }

}