use std::fs::File;
use std::io::{Cursor, Read};
use uuid::Uuid;
use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};
use rand::os::OsRng;
use rand::Rng;

mod io;
pub mod serialize;

#[derive(Debug, Eq, PartialEq, Clone)]
enum ParseError {
    WrongValue(u64),
    IllegalPos(u64),
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
    SCrypt { iterations: u16, memory_costs: u16, parallelism: u16 },
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
        let it = rng.gen_range(1000, 30000);
        let mem = rng.gen_range(1024, 8192);
        let cpu = rng.gen_range(4, 64);
        let kdf = PasswordHashType::SCrypt { iterations: it, memory_costs: mem, parallelism: cpu };
        RepoHeader::new(kdf, EncryptionType::RingChachaPoly1305)
    }
    pub fn new(kdf: PasswordHashType, enc_type: EncryptionType) -> Self {
        let salt = random_vec(kdf.salt_len());
        let mh = MainHeader::new(FileVersion::RepositoryV1);
        RepoHeader { main_header: mh, encryption_type: enc_type, password_hash_type: kdf, salt: salt }
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
}

fn random_vec(len: usize) -> Vec<u8> {
    let mut rng = OsRng::new().unwrap();
    let mut salt = vec![0u8; len];

    rng.fill_bytes(salt.as_mut_slice());
    salt
}