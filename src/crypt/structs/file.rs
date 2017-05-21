// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use super::super::structs::repository::{RepoHeader, Repository};
use dto::EncryptionType;
use super::{MainHeader, FileVersion};
use super::crypto::HashedPw;
use super::super::util::{decrypt, encrypt};
use super::super::error::{CryptError, ParseError};
use super::super::util::tempfile::TempFile;
use super::super::util::random_vec;
use super::super::util::io::{path_to_str, read_file_header, read_repo_header};
use std::path::PathBuf;
use std::fs::{copy, File};
use std::io::{Read, Write, Cursor};
use uuid::Uuid;
use byteorder::{WriteBytesExt, LittleEndian};
use super::serialize::*;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct FileHeader {
    main_header: MainHeader,
    repository_id: Uuid,
    encryption_type: EncryptionType,
    //nonce header length
    //nonce content length
    header_length: u32,
    nonce_header: Vec<u8>,
    nonce_content: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct EncryptedFile {
    encryption_header: FileHeader,
    header: String,
    content: Option<Vec<u8>>,
    path: Option<PathBuf>,
}


impl FileHeader {
    pub fn new(repository: &RepoHeader) -> Self {
        let mh = MainHeader::new(FileVersion::FileV1);
        let enc_type = repository.encryption_type.clone();
        let nc = random_vec(enc_type.nonce_len());
        let nh = random_vec(enc_type.nonce_len());
        FileHeader { main_header: mh, repository_id: repository.main_header.id, encryption_type: enc_type, nonce_content: nc, nonce_header: nh, header_length: 0 }
    }

    pub fn get_id(&self) -> Uuid {
        self.main_header.id.clone()
    }
    pub fn get_repository_id(&self) -> Uuid {
        self.repository_id.clone()
    }

    pub fn set_header_length(&mut self, length: u32) {
        self.header_length = length;
    }

    pub fn get_additional_data(&self) -> Vec<u8> {
        let mut v = Vec::new();
        self.main_header.to_bytes(&mut v);
        v
    }

    pub fn get_version(&self) -> u32 {
        self.main_header.version
    }

    pub fn get_encryption_type(&self) -> &EncryptionType {
        &self.encryption_type
    }


    pub fn get_main_header(&self) -> &MainHeader {
        &self.main_header
    }
}

impl EncryptedFile {
    pub fn new(enc_header: FileHeader, header: &str) -> Self {
        EncryptedFile { path: None, content: None, encryption_header: enc_header, header: header.into() }
    }
    pub fn with_content(enc_header: FileHeader, header: &str, content: &[u8]) -> Self {
        let mut f = EncryptedFile::new(enc_header, header);
        f.content = Some(content.to_vec());
        f
    }

    pub fn set_path(&mut self, path: &PathBuf) {
        self.path = Some(path.clone());
    }

    pub fn set_content(&mut self, content: &[u8]) {
        self.content = Some(content.to_vec());
    }

    pub fn get_id(&self) -> Uuid {
        self.encryption_header.get_id()
    }

    pub fn set_header(&mut self, header: &str) {
        self.header = header.to_string();
    }

    pub fn increment_version(&mut self) {
        self.encryption_header.main_header.version += 1;
    }

    pub fn get_version(&self) -> u32 {
        self.encryption_header.get_version()
    }

    pub fn get_path(&self) -> Option<PathBuf> {
        self.path.clone()
    }
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

        let plaintext = decrypt(&header.encryption_type, &header.nonce_header, key, buff.as_slice(), additional.as_slice())?;

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

        let plaintext = decrypt(&header.encryption_type, &header.nonce_content, key, buff.as_slice(), additional.as_slice())?;
        Ok(plaintext)
    }

    pub fn save(&mut self, key: &HashedPw) -> Result<(), CryptError> {
        let path = self.path.as_ref().ok_or(CryptError::NoFilePath)?;
        if path.exists() {
            return Err(CryptError::IOError(format!("File {} already exists", path_to_str(path))));
        }
        let content = self.content.as_ref().ok_or(CryptError::NoFileContent)?;

        let additional = self.encryption_header.get_additional_data();

        let ref mut header = self.encryption_header;

        let encryptedheadertext = encrypt(&header.encryption_type, header.nonce_header.as_slice(), key, self.header.as_bytes(), additional.as_slice())?;
        header.set_header_length(encryptedheadertext.len() as u32);
        let encryptedcontent = encrypt(&header.encryption_type, header.nonce_content.as_slice(), key, content, additional.as_slice())?;

        let mut header_bytes = Vec::new();
        header.to_bytes(&mut header_bytes);

        let mut temp = TempFile::new();
        {
            let mut tempfile = File::create(temp.path.clone())?;
            tempfile.write(header_bytes.as_slice())?;
            tempfile.write(encryptedheadertext.as_slice())?;
            tempfile.write(encryptedcontent.as_slice())?;
            tempfile.sync_all()?;
        }

        copy(temp.path.clone(), path)?;

        Ok(())
    }

    pub fn update(&mut self, key: &HashedPw, content: Option<Vec<u8>>) -> Result<(), CryptError> {
        let original_enc_header = self.encryption_header.clone();
        self.increment_version();
        let path = self.path.as_ref().ok_or(CryptError::NoFilePath)?;
        if !path.exists() {
            return Err(CryptError::FileDoesNotExist(path_to_str(path)));
        }
        let header_on_filesystem = read_file_header(&path)?;
        if header_on_filesystem.get_version() >= self.get_version() {
            return Err(CryptError::OptimisticLockError(header_on_filesystem.get_version()));
        }

        let content = match content {
            Some(c) => Ok(c),
            None => EncryptedFile::load_content(&original_enc_header, key, &path),
        }?;
        //            EncryptedFile::load_content(&original_enc_header, key, &path)?;
        let additional = self.encryption_header.get_additional_data();
        let ref mut header = self.encryption_header;

        let encryptedheadertext = encrypt(&header.encryption_type, header.nonce_header.as_slice(), key, self.header.as_bytes(), additional.as_slice())?;
        header.set_header_length(encryptedheadertext.len() as u32);
        let encryptedcontent = encrypt(&header.encryption_type, header.nonce_content.as_slice(), key, content.as_slice(), additional.as_slice())?;

        let mut header_bytes = Vec::new();
        header.to_bytes(&mut header_bytes);

        let mut temp = TempFile::new();
        {
            let mut tempfile = File::create(temp.path.clone())?;
            tempfile.write(header_bytes.as_slice())?;
            tempfile.write(encryptedheadertext.as_slice())?;
            tempfile.write(encryptedcontent.as_slice())?;
            tempfile.sync_all()?;
        }

        copy(temp.path.clone(), path)?;
        Ok(())
    }

    pub fn get_header(&self) -> &String {
        &self.header
    }
    pub fn get_encryption_header(&self) -> &FileHeader {
        &self.encryption_header
    }
}


impl ByteSerialization for FileHeader {
    fn to_bytes(&self, vec: &mut Vec<u8>) {
        self.main_header.to_bytes(vec);
        vec.extend_from_slice(self.repository_id.as_bytes());
        self.encryption_type.to_bytes(vec);

        let nonce_header_len = self.nonce_header.len() as u8;
        let nonce_content_len = self.nonce_content.len() as u8;
        vec.write_u8(nonce_header_len);
        vec.write_u8(nonce_content_len);
        vec.write_u32::<LittleEndian>(self.header_length);
        vec.append(&mut self.nonce_header.clone());
        vec.append(&mut self.nonce_content.clone());
    }

    fn from_bytes(input: &mut Cursor<&[u8]>) -> Result<Self, ParseError> {
        let main_header = MainHeader::from_bytes(input)?;
        if main_header.file_version != FileVersion::FileV1 {
            return Err(ParseError::InvalidFileVersion(main_header.file_version));
        }
        let repo_id = read_uuid(input)?;
        let enc_type = EncryptionType::from_bytes(input)?;

        let nonce_header_len = read_u8(input)?;
        let nonce_content_len = read_u8(input)?;
        let header_len = read_u32(input)?;

        let mut nonce_header = vec![0u8; nonce_header_len as usize];
        read_buff(input, nonce_header.as_mut_slice())?;

        let mut nonce_content = vec![0u8; nonce_content_len as usize];
        read_buff(input, nonce_content.as_mut_slice())?;

        Ok(FileHeader { main_header: main_header, repository_id: repo_id, encryption_type: enc_type, header_length: header_len, nonce_header: nonce_header, nonce_content: nonce_content })
    }
    fn byte_len(&self) -> usize {
        self.main_header.byte_len() + UUID_LENGTH + self.encryption_type.byte_len() + 2 + 4 + self.nonce_header.len() + self.nonce_content.len()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use tempdir::TempDir;
    use super::super::crypto::PlainPw;
    use notify::{RecommendedWatcher, Watcher, RecursiveMode, DebouncedEvent};
    use std::sync::mpsc::channel;
    use std::time::Duration;
    use std::path::Path;
    use std::ffi::OsString;
    use std::fs::remove_file;
    use spectral::prelude::*;
    use super::super::super::util::io::{scan};

    #[test]
    fn encrypted_file() {
        let tempdir = TempDir::new("scanfolder").unwrap();
        let dir = tempdir.path();

        let repo_header = RepoHeader::new_for_test();
        let repo = Repository::new("test", PlainPw::new("password".as_bytes()), repo_header);
        let key = repo.hash_key(PlainPw::new("password".as_bytes()));

        let mut encrypted_file = EncryptedFile::with_content(FileHeader::new(&repo.get_header()), "header", "content".as_bytes());
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

    fn unwrap_filename(p: &Path) -> OsString {
        p.to_path_buf().file_name().unwrap().to_os_string()
    }

    fn create_temp_file() -> (EncryptedFile, HashedPw, PathBuf, TempDir) {
        let tempdir = TempDir::new("scanfolder").unwrap();
        let dir = tempdir.path().to_path_buf();

        let repo_header = RepoHeader::new_for_test();
        let repo = Repository::new("test", PlainPw::new("password".as_bytes()), repo_header);
        let key = repo.hash_key(PlainPw::new("password".as_bytes()));

        let mut encrypted_file = EncryptedFile::with_content(FileHeader::new(&repo.get_header()), "header", "content".as_bytes());
        {
            encrypted_file.set_path(&dir.join("myfile"));
            encrypted_file.save(&key).unwrap();
        }
        (encrypted_file, key, dir, tempdir)
    }

    #[test]
    fn update_header() {
        let (mut encrypted_file, key, dir, temp) = create_temp_file();
        let original_version = encrypted_file.get_version();
        encrypted_file.set_header("new header");
        encrypted_file.update(&key,None);

        let res = scan(&vec![dir.to_path_buf()]).unwrap();
        let tuple = res.get_files().get(&encrypted_file.get_id()).unwrap();
        let ref header = tuple.0;
        let ref path = tuple.1;
        let reloaded = EncryptedFile::load_head(header, &key, path).unwrap();
        assert_eq!(original_version + 1, reloaded.get_version());
        assert_eq!("new header", reloaded.get_header());
    }

    #[test]
    fn double_file_save() {
        let (mut encrypted_file, key, dir, temp) = create_temp_file();
        let res = encrypted_file.save(&key);
        match res {
            Ok(_) => panic!("Should have failed and not written file"),
            Err(CryptError::IOError(msg)) => assert_that(&msg).contains("already exists"),
            _ => panic!("Unknown error occured {:?}", res),
        }
    }

    #[test]
    fn update_header_nofile() {
        let (mut encrypted_file, key, dir, mut temp) = create_temp_file();

        //        remove_file(encrypted_file.get_path().unwrap()).unwrap();
        temp.close().unwrap();

        let res = encrypted_file.update(&key,None);
        match res {
            Err(CryptError::FileDoesNotExist(s)) => assert_that(&s).contains("myfile"),
            _ => panic!("Invalid result: {:?}", res),
        }
    }

    #[test]
    fn update_header_optimisticlockerror() {
        let (mut encrypted_file, key, dir, temp) = create_temp_file();
        let mut clone = encrypted_file.clone();
        clone.update(&key,None);

        let res = encrypted_file.update(&key,None);
        match res {
            Err(CryptError::OptimisticLockError(v)) => assert_eq!(1, v),
            _ => panic!("Invalid result: {:?}", res),
        }
    }

    #[test]
    fn serialize_file_header() {
        let rh = RepoHeader::new_for_test();
        let h = FileHeader::new(&rh);

        let mut result = Vec::new();
        h.to_bytes(&mut result);

        let main_header_len = 2 + 1 + UUID_LENGTH + 4;
        let repo_id = UUID_LENGTH;
        let enc_type_len = 1;
        let nonce1_len = 1;
        let nonce2_len = 1;
        let sizeof_header = 4;
        let expected_len = main_header_len + repo_id + enc_type_len + nonce1_len + nonce2_len + sizeof_header + 2 * 12;

        assert_eq!(expected_len, h.byte_len());
        assert_eq!(expected_len, result.len());

        let parse_back = FileHeader::from_bytes(&mut Cursor::new(result.as_slice())).unwrap();
        assert_eq!(h, parse_back);
    }
}