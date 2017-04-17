extern crate uuid;

use std::fs::File;

use uuid::Uuid;

pub enum EncryptionType {
    None,
    RingChacha,
    RingAES,
}

pub enum FileHashType {
    None,
    Sha256,
    Poly1305,
    HMac,
}

pub enum PasswordHashType{
    None,
    SCrypt,
//    Argon2,
}

pub struct MainHeader {
    //beaf
    file_version: u8,
    id: Vec<u8>,
    version: u32,
}

pub struct RepoHeader {
    main_header: MainHeader,
    encryption_type: EncryptionType,
    file_hash_type: FileHashType,
    password_hash_type: PasswordHashType,
    iterations_password: u16,
    iterations_mac: u16,
    memory_costs: u16,
    cpu_costs: u16,
    salt: Vec<u8>,
    iv: Vec<u8>,
    hash: Vec<u8>,
}

pub struct FileHeader {
    main_header: MainHeader,
    encryption_type: EncryptionType,
    file_hash_type: FileHashType,
    iv_header: Vec<u8>,
    iv_content: Vec<u8>,
    hash_header: Vec<u8>,
    hash_content: Vec<u8>,
}