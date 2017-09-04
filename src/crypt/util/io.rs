// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use super::super::error::{CryptError,ParseError};
use super::super::structs::repository::{RepoHeader, Repository};
use super::super::structs::file::{FileHeader};
use super::super::structs::{MainHeader, FileVersion};
use super::super::actor::state::scanresult::{ScanResult, CheckRes};
use super::super::structs::serialize::ByteSerialization;
use std::path::PathBuf;
use std::fs::{File,DirEntry};
use std::time::Duration;
use std::io::{Read, Cursor};
use std::io;
use notify::{Watcher, RecursiveMode, watcher};
use std::sync::mpsc::{channel};
use base64::{decode};

pub fn scan(folders: &Vec<PathBuf>) -> Result<ScanResult, CryptError> {
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(10))?;
    for path in folders {
        watcher.watch(path, RecursiveMode::Recursive)?;
    }
    let check_results: Vec<CheckRes> = folders.into_iter().flat_map(|p| scan_folder(&p)).collect();

    let mut s = ScanResult::new(watcher, rx,folders);
    for i in check_results {
        match i {
            CheckRes::Repo(_, p) => {
                let load = Repository::load(p);
                if load.is_ok() {
                    s.add_repo(load.unwrap());
                }
            }
            CheckRes::File(h, p) => {
                s.add_file(h,p);
            }
            CheckRes::Error(e, p) => s.add_invalid(e,p),
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
            match header.get_file_version() {
                &FileVersion::FileV1 => {
                    match read_file_header(&path) {
                        Err(e) => CheckRes::Error(e, path.clone()),
                        Ok(f) => CheckRes::File(f, path.clone()),
                    }
                }
                &FileVersion::RepositoryV1 => {
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


#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;
    use notify::{RecommendedWatcher, Watcher, RecursiveMode, DebouncedEvent};
    use std::sync::mpsc::channel;
    use std::time::Duration;
    use std::path::Path;
    use std::ffi::OsString;
    use std::fs::remove_file;
    use super::super::super::structs::repository::{RepoHeader};
    use std::io::Write;
    use base64::encode;

    #[test]
    fn file_existance() {
        let dir = TempDir::new("file_existance").unwrap().into_path();
        let err = check_file_exists("4711", &dir);
        assert_eq!(Err(CryptError::FileDoesNotExist(path_to_str(&dir.join("4711")))), err);

        let err = check_plain_files_exist("4711", &dir);
        assert_eq!(Err(CryptError::FileDoesNotExist(path_to_str(&dir.join("4711.json")))), err);
        {
            File::create(&dir.join("4711.json")).unwrap();
        }
        let err = check_plain_files_exist("4711", &dir);
        assert_eq!(Err(CryptError::FileDoesNotExist(path_to_str(&dir.join("4711")))), err);
    }

    #[test]
    fn no_file_exists() {
        let dir = TempDir::new("file_not_existance").unwrap().into_path();
        {
            File::create(&dir.join("4711")).unwrap();
        }

        let err = check_file_not_exists("4711", &dir);
        assert_eq!(Err(CryptError::FileAlreadyExists(path_to_str(&dir.join("4711")))), err);


        let err = check_plain_files_not_exist("4711", &dir);
        assert_eq!(Err(CryptError::FileAlreadyExists(path_to_str(&dir.join("4711")))), err);
        {
            File::create(&dir.join("4711.json")).unwrap();
        }
        let err = check_plain_files_not_exist("4711", &dir);
        assert_eq!(Err(CryptError::FileAlreadyExists(path_to_str(&dir.join("4711.json")))), err);
    }

    #[test]
    fn bin_header_correct() {
        let header = MainHeader::new(FileVersion::FileV1);
        let dir = TempDir::new("header").unwrap().into_path();
        {
            let mut f = File::create(&dir.join("4711")).unwrap();
            let mut c = Vec::new();
            header.to_bytes(&mut c);
            f.write_all(c.as_slice()).unwrap();
        }
        let res = check_file_prefix("4711", &dir, false);
        assert_eq!(Ok(header), res);
    }

    #[test]
    fn bin_header_wrong() {
        let header = MainHeader::new(FileVersion::FileV1);
        let dir = TempDir::new("header").unwrap().into_path();
        {
            let mut f = File::create(&dir.join("4711")).unwrap();
            let mut c = Vec::new();
            header.to_bytes(&mut c);
            c[0] = 0xAA;
            f.write_all(c.as_slice()).unwrap();
        }
        let res = check_file_prefix("4711", &dir, false);
        assert_eq!(Err(CryptError::ParseError(ParseError::NoPrefix)), res);
    }

    #[test]
    fn plain_header_correct() {
        let header = MainHeader::new(FileVersion::FileV1);
        let dir = TempDir::new("header").unwrap().into_path();
        {
            let mut f = File::create(&dir.join("4711.json")).unwrap();
            let mut c = Vec::new();
            header.to_bytes(&mut c);
            let b64 = encode(c.as_slice());
            f.write_all(b64.as_bytes()).unwrap();
        }
        let res = check_file_prefix("4711", &dir, true);
        assert_eq!(Ok(header), res);
    }

    #[test]
    fn plain_header_wrong() {
        let header = MainHeader::new(FileVersion::FileV1);
        let dir = TempDir::new("header").unwrap().into_path();
        {
            let mut f = File::create(&dir.join("4711.json")).unwrap();
            let mut c = Vec::new();
            header.to_bytes(&mut c);
            let b64 = encode(c.as_slice()).replace("vq", "ee");
            f.write_all(b64.as_bytes()).unwrap();
        }
        let res = check_file_prefix("4711", &dir, true);
        assert_eq!(Err(CryptError::ParseError(ParseError::NoPrefix)), res);
    }

    #[test]
    fn scan_folder() {
        let dir = TempDir::new("scanfolder").unwrap().into_path();
        {
            let mut repofile = File::create(&dir.join("repository")).unwrap();
            let mut file1 = File::create(&dir.join("file1")).unwrap();
            let mut file2 = File::create(&dir.join("file2")).unwrap();
            let mut file3 = File::create(&dir.join("errorfile")).unwrap();

            let repo_header = RepoHeader::new_for_test();
            let mut v = Vec::new();
            repo_header.to_bytes(&mut v);
            repofile.write_all(v.as_slice()).unwrap();

            let mut v = Vec::new();
            FileHeader::new(&repo_header).to_bytes(&mut v);
            file1.write_all(v.as_slice()).unwrap();

            let mut v = Vec::new();
            FileHeader::new(&repo_header).to_bytes(&mut v);
            file2.write_all(v.as_slice()).unwrap();

            file3.write_all("hello world".as_bytes()).unwrap();
        }
        let result: Vec<CheckRes> = super::scan_folder(&dir);
        assert_eq!(4, result.len());

        let repo = result.iter().find(|r| match **r {
            CheckRes::Repo(_, _) => true,
            _ => false
        }).unwrap();
        assert_eq!(dir.join("repository"), repo.get_path());


        let files: Vec<_> = result.iter().filter(|r| match **r {
            CheckRes::File(_, _) => true,
            _ => false
        }).collect();
        assert_eq!(2, files.len());


        let error = result.iter().find(|r| match **r {
            CheckRes::Error(_, _) => true,
            _ => false
        }).unwrap();
        assert_eq!(dir.join("errorfile"), error.get_path());
    }


    #[test]
    fn file_watcher() {
        let tempdir = TempDir::new("filewatcher").unwrap();

        let (tx, rx) = channel();

        let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_millis(10)).unwrap();
        watcher.watch(tempdir.path(), RecursiveMode::NonRecursive);

        let path = tempdir.path().join("testfile");
        {
            File::create(path.clone()).unwrap();
        }
        #[cfg(target_os = "macos")]
        {
            let change = rx.recv_timeout(Duration::from_millis(100)).unwrap();
            match change {
                DebouncedEvent::Create(p) => {
                    info!("Got {:?}", p);
                }
                _ => panic!("received invalid notification {:?}", &change)
            }
        }
        let change = rx.recv_timeout(Duration::from_millis(100)).unwrap();
        match change {
            DebouncedEvent::Create(ref p) => {
                assert_eq!(unwrap_filename(&path), unwrap_filename(p), "not the expected creation path. expected {:?} but got {:?}", unwrap_filename(&path), unwrap_filename(p));
            }
            _ => panic!("received invalid notification {:?}", &change)
        }
        remove_file(path.clone()).unwrap();

        let change = rx.recv_timeout(Duration::from_millis(100)).unwrap();
        match change {
            DebouncedEvent::NoticeRemove(ref p) => {
                assert_eq!(unwrap_filename(&path), unwrap_filename(p), "not the expected deletion path. expected {:?} but got {:?}", unwrap_filename(&path), unwrap_filename(p));
            }
            _ => panic!("received invalid notification {:?}", &change)
        }
    }

    fn unwrap_filename(p: &Path) -> OsString {
        p.to_path_buf().file_name().unwrap().to_os_string()
    }
}