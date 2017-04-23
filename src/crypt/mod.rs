use std::io::{Cursor};
use std::path::PathBuf;
use uuid::Uuid;
use rand::os::OsRng;
use rand::Rng;
use std;
use crypt::serialize::ByteSerialization;
use crypt::crypt::{DoubleHashedPw, PlainPw};
use std::str::Utf8Error;
use std::fmt::{Display,Formatter};
use std::fmt;
use std::error::Error;

mod io;
mod crypt;
pub mod actor;
pub mod serialize;

#[derive(Debug, Eq, PartialEq, Clone)]
enum ParseError {
    WrongValue(u64, u8),
    IllegalPos(u64),
    InvalidUtf8(String),
    IoError(String),
    NoPrefix,
    NoValidUuid(u64),
    UnknownFileVersion(u8),
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            ParseError::WrongValue(ref pos,ref  val) => write!(f, "Wrong value '{}' at pos {}", val, pos),
            ParseError::IllegalPos(ref pos) => write!(f, "Illegal position {}", pos),
            ParseError::InvalidUtf8(ref e) => write!(f, "No valid utf8: {}", e),
            ParseError::IoError(ref description) => write!(f, "IO Error happened: {}", description),
            ParseError::NoPrefix => write!(f, "No prefix present"),
            ParseError::NoValidUuid(ref pos) => write!(f, "No valid uuid at {}", pos),
            ParseError::UnknownFileVersion(ref version) => write!(f, "Unknown file version {}", version),
        }
    }
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
    hash: DoubleHashedPw,
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
    #[cfg(test)]
    pub fn new_for_test() -> Self {
        let it = 1;
        let mem = 1;
        let cpu = 1;
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

    pub fn set_content(&mut self, content: &[u8]) {
        self.content = Some(content.to_vec());
    }

    pub fn get_id(&self) -> Uuid {
        self.encryption_header.get_id()
    }
}

impl Repository {
    pub fn new(name: &str, pw: PlainPw, header: RepoHeader) -> Self {
        let checksum = {
            let v = Repository::hash_key_ext(&header.encryption_type, &header.password_hash_type, pw);
            Repository::hash_pw_ext(&header.encryption_type, &header.password_hash_type, &v)
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
        ParseError::InvalidUtf8(e.description().into())
    }
}

impl From<std::io::Error> for ParseError {
    fn from(e: std::io::Error) -> Self {
        ParseError::IoError(e.description().into())
    }
}