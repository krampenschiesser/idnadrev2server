use super::super::error::CryptError;
use super::super::structs::repository::{RepoHeader, Repository};
use super::super::structs::file::{FileHeader};
use super::super::structs::{MainHeader, FileVersion};
use super::super::actor::state::scanresult::{ScanResult, CheckRes};
use std::path::PathBuf;
use std::time::Duration;
use std::io::{Read, Write, Cursor};
use std::io;
use notify::{Watcher, RecursiveMode, watcher, DebouncedEvent, RecommendedWatcher};

pub fn scan(folders: &Vec<PathBuf>) -> Result<ScanResult, CryptError> {
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(10))?;
    for path in folders {
        watcher.watch(path, RecursiveMode::Recursive)?;
    }
    let check_results: Vec<CheckRes> = folders.into_iter().flat_map(|p| scan_folder(&p)).collect();

    let mut s = ScanResult::new(watcher, rx);
    for i in check_results {
        match i {
            CheckRes::Repo(_, p) => {
                let load = Repository::load(p);
                if load.is_ok() {
                    s.repositories.push(load.unwrap());
                }
            }
            CheckRes::File(h, p) => {
                s.files.insert(h.get_id(), (h, p));
            }
            CheckRes::Error(e, p) => s.invalid.push((e, p)),
        };
    }
    Ok(s)
}

pub fn scan_folder(folder: &PathBuf) -> Vec<CheckRes> {
    match folder.read_dir() {
        Err(_) => Vec::new(),
        Ok(file_iter) => {
            let results: Vec<CheckRes> = file_iter.map(|file| check_map_dir_entry(file)).filter(|r| r.is_ok()).map(|r| r.unwrap()).collect();
            results
        }
    }
}

fn check_map_dir_entry(dir_entry: Result<DirEntry, io::Error>) -> Result<CheckRes, ()> {
    if dir_entry.is_err() {
        return Err(());
    }
    let path = dir_entry.unwrap().path();
    check_map_path(&path)
}

pub fn check_map_path(path: &PathBuf) -> Result<CheckRes, ()> {
    let ext = path.extension();
    let is_json_file = match ext {
        Some(extension) => extension == ".json",
        _ => false,
    };

    let result = if is_json_file {
        check_json_file(&path)
    } else {
        check_bin_file(&path)
    };

    let val = match result {
        Ok(header) => {
            match header.file_version {
                FileVersion::FileV1 => {
                    match read_file_header(&path) {
                        Err(e) => CheckRes::Error(e, path.clone()),
                        Ok(f) => CheckRes::File(f, path.clone()),
                    }
                }
                FileVersion::RepositoryV1 => {
                    match read_repo_header(&path) {
                        Err(e) => CheckRes::Error(e, path.clone()),
                        Ok(r) => CheckRes::Repo(r, path.clone()),
                    }
                }
            }
        }
        Err(error) => {
            CheckRes::Error(error, path.clone())
        }
    };
    Ok(val)
}

pub fn read_file_header(path: &PathBuf) -> Result<FileHeader, CryptError> {
    let f = File::open(path)?;
    let mut v = Vec::new();
    f.take(1000).read_to_end(&mut v)?;
    let mut cursor = Cursor::new(v.as_slice());
    let header = FileHeader::from_bytes(&mut cursor)?;
    Ok(header)
}


pub fn read_repo_header(path: &PathBuf) -> Result<RepoHeader, CryptError> {
    let f = File::open(path)?;
    let mut v = Vec::new();
    f.take(1000).read_to_end(&mut v)?;
    let mut cursor = Cursor::new(v.as_slice());
    let header = RepoHeader::from_bytes(&mut cursor)?;
    Ok(header)
}


fn check_plain_files_not_exist(id: &str, folder: &PathBuf) -> Result<(), CryptError> {
    check_file_not_exists(format!("{}.json", id).as_str(), folder)?;
    check_file_not_exists(id, folder)
}

fn check_plain_files_exist(id: &str, folder: &PathBuf) -> Result<(), CryptError> {
    check_file_exists(format!("{}.json", id).as_str(), &folder)?;
    check_file_exists(id, &folder)
}

fn check_file_not_exists(id: &str, folder: &PathBuf) -> Result<(), CryptError> {
    let main_path = folder.join(id);
    let r = check_file_exists(id, folder);
    match r {
        Ok(_) => Err(CryptError::FileAlreadyExists(path_to_str(&main_path))),
        Err(_) => Ok(())
    }
}

fn check_file_exists(id: &str, folder: &PathBuf) -> Result<(), CryptError> {
    let main_path = folder.join(id);
    if !main_path.exists() {
        return Err(CryptError::FileDoesNotExist(path_to_str(&main_path)));
    }
    Ok(())
}

fn check_file_prefix(id: &str, folder: &PathBuf, plain_files: bool) -> Result<MainHeader, CryptError> {
    if plain_files {
        check_json_file(&folder.join(format!("{}.json", id)))
    } else {
        let path = folder.join(id);
        check_bin_file(&path)
    }
}

fn check_bin_file(path: &PathBuf) -> Result<MainHeader, CryptError> {
    let file = File::open(path)?;
    let header_length = 23;
    let mut file = file.take(header_length);
    let mut header_content = Vec::new();
    file.read_to_end(&mut header_content)?;
    let h = MainHeader::from_bytes(&mut Cursor::new(header_content.as_slice()))?;
    Ok(h)
}

fn check_json_file(path: &PathBuf) -> Result<MainHeader, CryptError> {
    let file = File::open(path)?;
    let b64_len = 32;
    let mut file = file.take(b64_len);
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let decode = decode(&content).map_err(|_| CryptError::ParseError(ParseError::NoPrefix))?;
    let mut cursor = Cursor::new(decode.as_slice());
    let h = MainHeader::from_bytes(&mut cursor)?;
    Ok(h)
}

pub fn path_to_str(path: &PathBuf) -> String {
    match path.to_str() {
        Some(str) => String::from(str),
        None => String::from(path.to_string_lossy()),
    }
}