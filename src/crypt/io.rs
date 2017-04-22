use uuid::Uuid;
use std::path::PathBuf;
use std::fs::File;
use super::*;
use super::serialize::ByteSerialization;
use base64::{encode, decode};
use std::io::{Read, Write};

#[derive(Debug, Eq, PartialEq)]
enum Error {
    FileAlreadyExists(String),
    FileDoesNotExist(String),
    WrongPrefix,
    IOError,
    ParseError(super::ParseError),
}

pub fn save_file(encryption_header: FileHeader, plain_header: &str, plain_content: &[u8], pw: &[u8], ) {}

pub fn save_file_header(encryption_header: FileHeader, plain_header: &str, pw: &[u8]) {}

pub fn save_file_content(encryption_header: FileHeader, plain_content: &[u8], pw: &[u8]) {}

fn check_newfile(id: Uuid, version: u32, file: File) {}

fn check_plain_files_not_exist(id: &str, folder: &PathBuf) -> Result<(), Error> {
    check_file_not_exists(format!("{}.json", id).as_str(), folder)?;
    check_file_not_exists(id, folder)
}

fn check_plain_files_exist(id: &str, folder: &PathBuf) -> Result<(), Error> {
    check_file_exists(format!("{}.json", id).as_str(), &folder)?;
    check_file_exists(id, &folder)
}

fn check_file_not_exists(id: &str, folder: &PathBuf) -> Result<(), Error> {
    let main_path = folder.join(id);
    let r = check_file_exists(id, folder);
    match r {
        Ok(_) => Err(Error::FileAlreadyExists(path_to_str(&main_path))),
        Err(str) => Ok(())
    }
}

fn check_file_exists(id: &str, folder: &PathBuf) -> Result<(), Error> {
    let main_path = folder.join(id);
    if !main_path.exists() {
        return Err(Error::FileDoesNotExist(path_to_str(&main_path)));
    }
    Ok(())
}

fn check_file_prefix(id: &str, folder: &PathBuf, plain_files: bool, check_new: bool) -> Result<MainHeader, Error> {
    if plain_files {
        let file = File::open(folder.join(format!("{}.json", id))).map_err(|e| Error::IOError)?;
        let b64_len = 32;
        let mut file = file.take(b64_len);
        let mut content = String::new();
        file.read_to_string(&mut content).map_err(|e| Error::IOError)?;
        let decode = decode(&content).map_err(|e| Error::WrongPrefix)?;
        let mut cursor = Cursor::new(decode.as_slice());
        MainHeader::from_bytes(&mut cursor).map_err(|e| Error::ParseError(e))
    } else {
        let header_length = 23;
        let file = File::open(folder.join(id)).map_err(|e| Error::IOError)?;
        let mut file = file.take(header_length);
        let mut header_content = Vec::new();
        file.read_to_end(&mut header_content).map_err(|e| Error::IOError)?;
        MainHeader::from_bytes(&mut Cursor::new(header_content.as_slice())).map_err(|e| Error::ParseError(e))
    }
}

fn path_to_str(path: &PathBuf) -> String {
    match path.to_str() {
        Some(str) => String::from(str),
        None => String::from(path.to_string_lossy()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;

    #[test]
    fn file_existance() {
        let mut dir = TempDir::new("file_existance").unwrap().into_path();
        let err = check_file_exists("4711", &dir);
        assert_eq!(Err(Error::FileDoesNotExist(path_to_str(&dir.join("4711")))), err);

        let err = check_plain_files_exist("4711", &dir);
        assert_eq!(Err(Error::FileDoesNotExist(path_to_str(&dir.join("4711.json")))), err);
        {
            File::create(&dir.join("4711.json"));
        }
        let err = check_plain_files_exist("4711", &dir);
        assert_eq!(Err(Error::FileDoesNotExist(path_to_str(&dir.join("4711")))), err);
    }

    #[test]
    fn no_file_exists() {
        let mut dir = TempDir::new("file_not_existance").unwrap().into_path();
        {
            File::create(&dir.join("4711"));
        }

        let err = check_file_not_exists("4711", &dir);
        assert_eq!(Err(Error::FileAlreadyExists(path_to_str(&dir.join("4711")))), err);


        let err = check_plain_files_not_exist("4711", &dir);
        assert_eq!(Err(Error::FileAlreadyExists(path_to_str(&dir.join("4711")))), err);
        {
            File::create(&dir.join("4711.json"));
        }
        let err = check_plain_files_not_exist("4711", &dir);
        assert_eq!(Err(Error::FileAlreadyExists(path_to_str(&dir.join("4711.json")))), err);
    }

    #[test]
    fn bin_header_correct() {
        let header = MainHeader::new(FileVersion::FileV1);
        let mut dir = TempDir::new("header").unwrap().into_path();
        {
            let mut f = File::create(&dir.join("4711")).unwrap();
            let mut c = Vec::new();
            header.to_bytes(&mut c);
            f.write_all(c.as_slice()).unwrap();
        }
        let res = check_file_prefix("4711", &dir, false, false);
        assert_eq!(Ok(header), res);
    }

    #[test]
    fn bin_header_wrong() {
        let header = MainHeader::new(FileVersion::FileV1);
        let mut dir = TempDir::new("header").unwrap().into_path();
        {
            let mut f = File::create(&dir.join("4711")).unwrap();
            let mut c = Vec::new();
            header.to_bytes(&mut c);
            c[0] = 0xAA;
            f.write_all(c.as_slice()).unwrap();
        }
        let res = check_file_prefix("4711", &dir, false, false);
        assert_eq!(Err(Error::ParseError(ParseError::NoPrefix)), res);
    }

    #[test]
    fn plain_header_correct() {
        let header = MainHeader::new(FileVersion::FileV1);
        let mut dir = TempDir::new("header").unwrap().into_path();
        {
            let mut f = File::create(&dir.join("4711.json")).unwrap();
            let mut c = Vec::new();
            header.to_bytes(&mut c);
            let b64 = encode(c.as_slice());
            f.write_all(b64.as_bytes()).unwrap();
        }
        let res = check_file_prefix("4711", &dir, true, false);
        assert_eq!(Ok(header), res);
    }

    #[test]
    fn plain_header_wrong() {
        let header = MainHeader::new(FileVersion::FileV1);
        let mut dir = TempDir::new("header").unwrap().into_path();
        {
            let mut f = File::create(&dir.join("4711.json")).unwrap();
            let mut c = Vec::new();
            header.to_bytes(&mut c);
            let mut b64 = encode(c.as_slice()).replace("vq","ee");
            f.write_all(b64.as_bytes()).unwrap();
        }
        let res = check_file_prefix("4711", &dir, true, false);
        assert_eq!(Err(Error::ParseError(ParseError::NoPrefix)), res);
    }
}