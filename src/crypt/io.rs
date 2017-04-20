use uuid::Uuid;
use std::path::PathBuf;
use std::fs::File;
use super::*;
use base64::{encode, decode};

enum Error {
    FileAlreadyExists(String),
    FileDoesNotExist(String),
    WrongPrefix,
    IOError,
}

pub fn save_file(encryption_header: FileHeader, plain_header: &str, plain_content: &[u8], pw: &[u8], ) {}

pub fn save_file_header(encryption_header: FileHeader, plain_header: &str, pw: &[u8]) {}

pub fn save_file_content(encryption_header: FileHeader, plain_content: &[u8], pw: &[u8]) {}

fn check_newfile(id: Uuid, version: u32, file: File) {}

fn check_plain_files_not_exist(id: &str, folder: PathBuf) -> Result<(), Error> {
    let main_path = folder.join(id);
    let header_path = folder.join(format!("{}.json", id));
    if header_path.exists() {
        return Err(Error::FileDoesNotExist(path_to_str(header_path)));
    }
    if main_path.exists() {
        return Err(Error::FileDoesNotExist(path_to_str(main_path)));
    }
    Ok(())
}

fn check_plain_files_exist(id: &str, folder: PathBuf) -> Result<(), Error> {
    let main_path = folder.join(id);
    let header_path = folder.join(format!("{}.json", id));
    if !header_path.exists() {
        return Err(Error::FileDoesNotExist(path_to_str(header_path)));
    }
    if !main_path.exists() {
        return Err(Error::FileDoesNotExist(path_to_str(main_path)));
    }
    Ok(())
}

fn check_file_not_exists(id: &str, folder: PathBuf) -> Result<(), Error> {
    let main_path = folder.join(id);
    let r = check_file_exists(id, folder);
    match r {
        Ok(_) => Err(Error::FileAlreadyExists(path_to_str(main_path))),
        Err(str) => Ok(())
    }
}

fn check_file_exists(id: &str, folder: PathBuf) -> Result<(), Error> {
    let main_path = folder.join(id);
    if !main_path.exists() {
        return Err(Error::FileDoesNotExist(path_to_str(main_path)));
    }
    Ok(())
}

fn check_file_prefix(id: &str, folder: PathBuf, plain_files: bool, check_new: bool) -> Result<(), Error> {
    if plain_files {
        let file = File::open(folder.join(format!("{}.json", id))).map_err(|e|Error::IOError)?;
        let b64_len = 28;
        let mut file = file.take(b64_len);
        let mut content = String::new();
        file.read_to_string(&mut content);
        let decode = decode(&content);
    } else {
        File::open(folder.join(id));
    };

    Ok(())
}

fn path_to_str(path: PathBuf) -> String {
    match path.to_str() {
        Some(str) => String::from(str),
        None => String::from(path.to_string_lossy()),
    }
}
