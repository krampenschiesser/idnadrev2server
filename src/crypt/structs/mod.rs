use std::fmt::{Display,Formatter};
use ring_pwhash::scrypt::{scrypt, ScryptParams};
use ring::constant_time::verify_slices_are_equal;
use ring::aead::{AES_128_GCM,CHACHA20_POLY1305};
use std::time::{Instant};
use chrono::Duration;
use uuid::Uuid;

pub mod crypto;
pub mod file;
pub mod repository;
pub mod serialize;

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



impl Display for EncryptionType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            EncryptionType::None => write!(f, "None"),
            EncryptionType::RingChachaPoly1305=> write!(f, "ChachaPoly1305"),
            EncryptionType::RingAESGCM=> write!(f, "AesGcm"),
        }
    }
}
impl Display for FileVersion {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            FileVersion::RepositoryV1 => write!(f, "Repository-Version 1"),
            FileVersion::FileV1 => write!(f, "File-Version 1"),
        }
    }
}


impl PasswordHashType {
    pub fn salt_len(&self) -> usize {
        match *self {
            PasswordHashType::SCrypt { iterations, memory_costs, parallelism } => 32,
            PasswordHashType::Argon2i { iterations, memory_costs, parallelism } => 32,
            PasswordHashType::None => 0,
        }
    }
    pub fn hash(&self, input: &[u8], len: usize) -> Vec<u8> {
        match *self {
            PasswordHashType::None => input.to_vec(),
            PasswordHashType::SCrypt { iterations, memory_costs, parallelism } => {
                let mut buff = vec![0u8; len];
                let param = ScryptParams::new(iterations, memory_costs, parallelism);
                let now = Instant::now();
                scrypt(input, input, &param, buff.as_mut_slice());

                debug!("Scrypt took {}s", Duration::from_std(now.elapsed()).unwrap().num_milliseconds());
                buff
            }
            _ => unimplemented!()
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
    pub fn algorithm(&self) -> Option<&'static Algorithm> {
        match *self {
            EncryptionType::RingAESGCM => Some(&AES_256_GCM),
            EncryptionType::RingChachaPoly1305 => Some(&CHACHA20_POLY1305),
            EncryptionType::None => None,
        }
    }
}

impl MainHeader {
    fn new(file_version: FileVersion) -> Self {
        let id = Uuid::new_v4();
        MainHeader { id: id, version: 0, file_version: file_version }
    }
}

fn random_vec(len: usize) -> Vec<u8> {
    let mut rng = OsRng::new().unwrap();
    let mut salt = vec![0u8; len];

    rng.fill_bytes(salt.as_mut_slice());
    salt
}
