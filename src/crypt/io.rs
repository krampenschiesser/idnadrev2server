use uuid::Uuid;
use std::path::PathBuf;
use std::fs::{File, DirEntry, rename, remove_file};
use super::*;
use super::serialize::ByteSerialization;
use base64::{encode, decode};
use std::io::{Read, Write};
use std::io;
use notify::{Watcher, RecursiveMode, watcher, DebouncedEvent, RecommendedWatcher};
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;
use super::crypt::HashedPw;
use super::error::*;


pub struct TempFile {
    path: PathBuf
}

//#[derive(Debug)] not possible due to file watcher
pub struct ScanResult {
    repositories: Vec<Repository>,
    files: Vec<(FileHeader, PathBuf)>,
    invalid: Vec<(CryptError, PathBuf)>,
    watcher: RecommendedWatcher,
    file_change_receiver: Receiver<DebouncedEvent>,
}

#[derive(Debug)]
pub enum CheckRes {
    Repo(RepoHeader, PathBuf),
    File(FileHeader, PathBuf),
    Error(CryptError, PathBuf),
}

impl CheckRes {
    fn get_path(&self) -> PathBuf {
        match *self {
            CheckRes::Repo(_, ref p) | CheckRes::File(_, ref p) | CheckRes::Error(_, ref p) => p.clone()
        }
    }
}

impl ScanResult {
    fn new(watcher: RecommendedWatcher, file_change_receiver: Receiver<DebouncedEvent>) -> Self {
        ScanResult { repositories: Vec::new(), files: Vec::new(), invalid: Vec::new(), watcher: watcher, file_change_receiver: file_change_receiver }
    }

    pub fn get_repository(&self, id: &Uuid) -> Option<Repository> {
        let result = self.repositories.iter().find(|repo| {
            repo.get_id() == *id
        });
        match result {
            Some(repo) => Some(repo.clone()),
            None => None,
        }
    }

    pub fn get_files_for_repo(&self, repo_id: &Uuid) -> Vec<(FileHeader, PathBuf)> {
        self.files.iter().filter(|ref t| t.0.get_repository_id() == *repo_id).map(|e| e.clone()).collect()
    }
}

impl TempFile {
    fn new() -> Self {
        let tempdir = std::env::temp_dir();
        let name = format!("{}", Uuid::new_v4().simple());
        TempFile::new_in_path(tempdir.join(name))
    }

    fn new_in_path(path: PathBuf) -> Self {
        TempFile { path: path }
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        match remove_file(self.path.clone()) {
            Err(d) => println!("Could not close temp file {}: {}", path_to_str(&self.path), d),
            _ => (),
        }
    }
}

impl Repository {
    pub fn load(path: PathBuf) -> Result<Self, CryptError> {
        let mut f = File::open(path.clone())?;
        let mut v = Vec::new();
        f.read_to_end(&mut v)?;

        let mut c = Cursor::new(v.as_slice());
        let mut repo = Repository::from_bytes(&mut c)?;
        repo.path = Some(path);
        Ok(repo)
    }
}

impl EncryptedFile {
    pub fn load_head(header: &FileHeader, key: &HashedPw, path: &PathBuf) -> Result<Self, CryptError> {
        let f = File::open(path.clone())?;
        let mut f = f.take(header.byte_len() as u64 + header.header_length as u64);
        let mut v = Vec::new();
        f.read_to_end(&mut v)?;
        let mut c = Cursor::new(v.as_slice());

        let additional = header.get_additional_data();
        c.set_position(header.byte_len() as u64);

        let mut buff = vec![0u8; header.header_length as usize];
        c.read_exact(buff.as_mut_slice())?;

        let plaintext = crypt::decrypt(&header.encryption_type, &header.nonce_header, key, buff.as_slice(), additional.as_slice())?;

        let plaintext = String::from_utf8(plaintext)?;

        let result = EncryptedFile { encryption_header: header.clone(), path: Some(path.clone()), content: None, header: plaintext };
        Ok(result)
    }

    pub fn load_content(header: &FileHeader, key: &HashedPw, path: &PathBuf) -> Result<Vec<u8>, CryptError> {
        let mut f = File::open(path.clone())?;
        let mut v = Vec::new();
        f.read_to_end(&mut v)?;
        let mut c = Cursor::new(v.as_slice());

        let additional = header.get_additional_data();
        c.set_position(header.byte_len() as u64 + header.header_length as u64);

        let mut buff = Vec::new();
        c.read_to_end(&mut buff)?;

        let plaintext = crypt::decrypt(&header.encryption_type, &header.nonce_content, key, buff.as_slice(), additional.as_slice())?;
        Ok(plaintext)
    }

    pub fn save(&mut self, key: &HashedPw) -> Result<(), CryptError> {
        let path = self.path.as_ref().ok_or(CryptError::NoFilePath)?;
        let content = self.content.as_ref().ok_or(CryptError::NoFileContent)?;

        let additional = self.encryption_header.get_additional_data();

        let ref mut header = self.encryption_header;

        let encryptedheadertext = crypt::encrypt(&header.encryption_type, header.nonce_header.as_slice(), key, self.header.as_bytes(), additional.as_slice())?;
        header.set_header_length(encryptedheadertext.len() as u32);
        let encryptedcontent = crypt::encrypt(&header.encryption_type, header.nonce_content.as_slice(), key, content, additional.as_slice())?;

        let mut header_bytes = Vec::new();
        header.to_bytes(&mut header_bytes);

        let temp = TempFile::new();
        {
            let mut tempfile = File::create(temp.path.clone())?;
            tempfile.write(header_bytes.as_slice())?;
            tempfile.write(encryptedheadertext.as_slice())?;
            tempfile.write(encryptedcontent.as_slice())?;
            tempfile.sync_all()?;
        }

        rename(temp.path.clone(), path)?;

        Ok(())
    }

    pub fn get_header(&self) -> &String {
        &self.header
    }
}

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
            CheckRes::File(h, p) => s.files.push((h, p)),
            CheckRes::Error(e, p) => s.invalid.push((e, p)),
        };
    }
    Ok(s)
}

pub fn scan_folder(folder: &PathBuf) -> Vec<CheckRes> {
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

fn read_file_header(path: &PathBuf) -> Result<FileHeader, CryptError> {
    let f = File::open(path)?;
    let mut v = Vec::new();
    f.take(1000).read_to_end(&mut v)?;
    let mut cursor = Cursor::new(v.as_slice());
    let header = FileHeader::from_bytes(&mut cursor)?;
    Ok(header)
}


fn read_repo_header(path: &PathBuf) -> Result<RepoHeader, CryptError> {
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
    let decode = decode(&content).map_err(|_| CryptError::WrongPrefix)?;
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
    use super::super::crypt::PlainPw;


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
    fn encrypted_file() {
        let tempdir = TempDir::new("scanfolder").unwrap();
        let dir = tempdir.path();

        let repo_header = RepoHeader::new_for_test();
        let repo = Repository::new("test", PlainPw::new("password".as_bytes()), repo_header);
        let key = repo.hash_key(PlainPw::new("password".as_bytes()));

        let mut encrypted_file = EncryptedFile::with_content(FileHeader::new(&repo.header), "header", "content".as_bytes());
        {
            encrypted_file.set_path(&dir.join("myfile"));
            encrypted_file.save(&key).unwrap();
        }
        let ref header = encrypted_file.encryption_header;
        let path = encrypted_file.path.as_ref().unwrap();
        let reloaded = EncryptedFile::load_head(header, &key, path).unwrap();
        let content = EncryptedFile::load_content(header, &key, path).unwrap();
        let contenttext = String::from_utf8(content).unwrap();
        assert_eq!("content", contenttext);
        assert_eq!("header", reloaded.header);
    }
}