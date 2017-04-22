use uuid::Uuid;
use std::path::PathBuf;
use std::fs::{File, DirEntry};
use super::*;
use super::serialize::ByteSerialization;
use base64::{encode, decode};
use std::io::{Read, Write};
use std::io;
use notify::{Watcher, RecursiveMode, watcher, DebouncedEvent, RecommendedWatcher};
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

#[derive(Debug, Eq, PartialEq)]
enum Error {
    FileAlreadyExists(String),
    FileDoesNotExist(String),
    WrongPrefix,
    IOError,
    ParseError(super::ParseError),
}
impl From<io::Error> for Error {
    fn from(a: io::Error) -> Self {
        Error::IOError
    }
}

impl From<ParseError> for Error {
    fn from(e: ParseError) -> Self {
        Error::ParseError(e)
    }
}

//#[derive(Debug)] not possible due to file watcher
pub struct ScanResult {
    repositories: Vec<(RepoHeader, PathBuf)>,
    files: Vec<(FileHeader, PathBuf)>,
    invalid: Vec<(Error, PathBuf)>,
    watcher: RecommendedWatcher,
    file_change_receiver: Receiver<DebouncedEvent>,
}

#[derive(Debug)]
pub enum CheckRes {
    Repo(RepoHeader, PathBuf),
    File(FileHeader, PathBuf),
    Error(Error, PathBuf),
}

impl ScanResult {
    fn new(watcher: RecommendedWatcher, file_change_receiver: Receiver<DebouncedEvent>) -> Self {
        ScanResult { repositories: Vec::new(), files: Vec::new(), invalid: Vec::new(), watcher: watcher, file_change_receiver: file_change_receiver }
    }
}

pub fn scan(folders: Vec<PathBuf>) -> Result<ScanResult, ()> {
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(10)).map_err(|e| ())?;
    for path in &folders {
        watcher.watch(path, RecursiveMode::Recursive).map_err(|e| ())?;
    }

    let check_results: Vec<CheckRes> = folders.into_iter().flat_map(|p| scan_folder(p)).collect();
    let mut s = ScanResult::new(watcher, rx);
    for i in check_results {
        match i {
            CheckRes::Repo(h, p) => s.repositories.push((h, p)),
            CheckRes::File(h, p) => s.files.push((h, p)),
            CheckRes::Error(e, p) => s.invalid.push((e, p)),
        };
    }
    Ok(s)
}

pub fn scan_folder(folder: PathBuf) -> Vec<CheckRes> {
    match folder.read_dir() {
        Err(_) => Vec::new(),
        Ok(file_iter) => {
            let results: Vec<CheckRes> = file_iter.map(|file| check_map_file(file)).filter(|r| r.is_ok()).map(|r| r.unwrap()).collect();
            results
        }
    }
}

fn check_map_file(dir_entry: Result<DirEntry, io::Error>) -> Result<CheckRes, ()> {
    if dir_entry.is_err() {
        return Err(());
    }
    let path = dir_entry.unwrap().path();
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

fn read_file_header(path: &PathBuf) -> Result<FileHeader, Error> {
    let f = File::open(path)?;
    let mut v = Vec::new();
    f.take(1000).read_to_end(&mut v)?;
    let mut cursor = Cursor::new(v.as_slice());
    let header = FileHeader::from_bytes(&mut cursor)?;
    Ok(header)
}


fn read_repo_header(path: &PathBuf) -> Result<RepoHeader, Error> {
    let f = File::open(path)?;
    let mut v = Vec::new();
    f.take(1000).read_to_end(&mut v)?;
    let mut cursor = Cursor::new(v.as_slice());
    let header= RepoHeader::from_bytes(&mut cursor)?;
    Ok(header)
}


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

fn check_file_prefix(id: &str, folder: &PathBuf, plain_files: bool) -> Result<MainHeader, Error> {
    if plain_files {
        check_json_file(&folder.join(format!("{}.json", id)))
    } else {
        let path = folder.join(id);
        check_bin_file(&path)
    }
}

fn check_bin_file(path: &PathBuf) -> Result<MainHeader, Error> {
    let file = File::open(path)?;
    let header_length = 23;
    let mut file = file.take(header_length);
    let mut header_content = Vec::new();
    file.read_to_end(&mut header_content)?;
    let h = MainHeader::from_bytes(&mut Cursor::new(header_content.as_slice()))?;
    Ok(h)
}

fn check_json_file(path: &PathBuf) -> Result<MainHeader, Error> {
    let file = File::open(path)?;
    let b64_len = 32;
    let mut file = file.take(b64_len);
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    let decode = decode(&content).map_err(|e| Error::WrongPrefix)?;
    let mut cursor = Cursor::new(decode.as_slice());
    let h = MainHeader::from_bytes(&mut cursor)?;
    Ok(h)
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
        let res = check_file_prefix("4711", &dir, false);
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
        let res = check_file_prefix("4711", &dir, false);
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
        let res = check_file_prefix("4711", &dir, true);
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
            let mut b64 = encode(c.as_slice()).replace("vq", "ee");
            f.write_all(b64.as_bytes()).unwrap();
        }
        let res = check_file_prefix("4711", &dir, true);
        assert_eq!(Err(Error::ParseError(ParseError::NoPrefix)), res);
    }
}