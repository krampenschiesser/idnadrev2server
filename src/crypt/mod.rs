use std::fs::File;
use std::io::{Cursor, Read};
use std::path::PathBuf;
use uuid::Uuid;
use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};
use rand::os::OsRng;
use rand::Rng;
use std;
use crypt::serialize::ByteSerialization;

mod io;
mod crypt;
pub mod actor;
pub mod serialize;

#[derive(Debug, Eq, PartialEq, Clone)]
enum ParseError {
    WrongValue(u64),
    IllegalPos(u64),
    InvalidUtf8,
    IoError,
    NoPrefix,
    NoValidUuid(u64),
    UnknownFileVersion(u8),
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum EncryptionType {
    None,
    RingChachaPoly1305,
    RingAESGCM,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum PasswordHashType {
    None,
    Argon2i { iterations: u16, memory_costs: u16, parallelism: u16 },
    SCrypt { iterations: u8, memory_costs: u32, parallelism: u32 },
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum FileVersion {
    RepositoryV1,
    FileV1,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct MainHeader {
    //beaf
    file_version: FileVersion,
    id: Uuid,
    version: u32,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct RepoHeader {
    pub main_header: MainHeader,
    pub encryption_type: EncryptionType,
    pub password_hash_type: PasswordHashType,
    pub salt: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Repository {
    header: RepoHeader,
    hash: Vec<u8>,
    name: String,
    path: Option<PathBuf>,
}

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

impl PasswordHashType {
    pub fn salt_len(&self) -> usize {
        match *self {
            PasswordHashType::SCrypt { iterations, memory_costs, parallelism } => 32,
            PasswordHashType::Argon2i { iterations, memory_costs, parallelism } => 32,
            PasswordHashType::None => 0,
        }
    }
}

impl EncryptionType {
    pub fn key_len(&self) -> usize {
        match *self {
            EncryptionType::RingAESGCM => 32,
            EncryptionType::RingChachaPoly1305 => 32,
            EncryptionType::None => 0,
        }
    }

    pub fn nonce_len(&self) -> usize {
        match *self {
            EncryptionType::RingAESGCM => 12,
            EncryptionType::RingChachaPoly1305 => 12,
            EncryptionType::None => 0,
        }
    }

    pub fn hash_len(&self) -> usize {
        match *self {
            EncryptionType::RingAESGCM => 16,
            EncryptionType::RingChachaPoly1305 => 16,
            EncryptionType::None => 0,
        }
    }
}

impl MainHeader {
    fn new(file_version: FileVersion) -> Self {
        let id = Uuid::new_v4();
        MainHeader { id: id, version: 0, file_version: file_version }
    }
}

impl RepoHeader {
    pub fn new_default_random() -> Self {
        let mut rng = OsRng::new().unwrap();
        let it = rng.gen_range(1, 5);
        let mem = rng.gen_range(1024, 4096);
        let cpu = 1;//rng.gen_range(1, 4);
        //        let it=16;
        //        let mem = 8;
        //        let cpu=1;
        let kdf = PasswordHashType::SCrypt { iterations: it, memory_costs: mem, parallelism: cpu };
        RepoHeader::new(kdf, EncryptionType::RingChachaPoly1305)
    }
    pub fn new(kdf: PasswordHashType, enc_type: EncryptionType) -> Self {
        let salt = random_vec(kdf.salt_len());
        let mh = MainHeader::new(FileVersion::RepositoryV1);
        RepoHeader { main_header: mh, encryption_type: enc_type, password_hash_type: kdf, salt: salt }
    }

    pub fn get_id(&self) -> Uuid {
        self.main_header.id.clone()
    }
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
}

impl Repository {
    pub fn new(name: &str, pw: &[u8], header: RepoHeader) -> Self {
        let checksum = {
            let len = header.encryption_type.hash_len();
            let ref kdf = header.password_hash_type;
            let v = kdf.hash(pw, len);
            kdf.hash(v.as_slice(), len)
        };
        Repository { header: header, hash: checksum, name: name.into(), path: None }
    }

    pub fn get_id(&self) -> Uuid {
        self.header.get_id()
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }
}

fn random_vec(len: usize) -> Vec<u8> {
    let mut rng = OsRng::new().unwrap();
    let mut salt = vec![0u8; len];

    rng.fill_bytes(salt.as_mut_slice());
    salt
}

impl From<std::string::FromUtf8Error> for ParseError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        ParseError::InvalidUtf8
    }
}

impl From<std::io::Error> for ParseError {
    fn from(e: std::io::Error) -> Self {
        ParseError::IoError
    }
}