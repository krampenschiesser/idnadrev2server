use super::{EncryptionType, PasswordHashType};
use std::path::PathBuf;
use ring::constant_time::verify_slices_are_equal;

#[derive(Clone)]
pub struct PlainPw {
    content: Vec<u8>
}

#[derive(Clone)]
pub struct HashedPw {
    content: Vec<u8>
}

#[derive(Clone, Debug)]
pub struct DoubleHashedPw {
    content: Vec<u8>
}

impl PlainPw {
    pub fn new(pw_plain: &[u8]) -> Self {
        PlainPw { content: pw_plain.to_vec() }
    }

    pub fn as_slice(&self) -> &[u8] {
        self.content.as_slice()
    }
}

impl<'a> From<&'a str> for PlainPw {
    fn from(i: &str) -> Self {
        PlainPw::new(i.as_bytes())
    }
}

impl From<String> for PlainPw {
    fn from(i: String) -> Self {
        PlainPw::new(i.as_bytes())
    }
}

impl HashedPw {
    pub fn new(plain: PlainPw, enc_type: &EncryptionType, hash_type: &PasswordHashType, salt: &[u8]) -> Self {
        let len = enc_type.key_len();
        let v = hash_type.hash(plain.as_slice(), salt, len);
        HashedPw { content: v }
    }

    pub fn as_slice(&self) -> &[u8] {
        self.content.as_slice()
    }
}

impl PartialEq for HashedPw {
    fn eq(&self, other: &HashedPw) -> bool {
        match verify_slices_are_equal(self.as_slice(), other.as_slice()) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}


impl DoubleHashedPw {
    pub fn new(hashed: &HashedPw, enc_type: &EncryptionType, hash_type: &PasswordHashType, salt: &[u8]) -> Self {
        let len = enc_type.key_len();
        let v = hash_type.hash(hashed.as_slice(), salt, len);
        DoubleHashedPw { content: v }
    }

    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        DoubleHashedPw { content: bytes }
    }

    pub fn as_slice(&self) -> &[u8] {
        self.content.as_slice()
    }

    pub fn len(&self) -> usize {
        self.content.len()
    }
}

impl PartialEq for DoubleHashedPw {
    fn eq(&self, other: &DoubleHashedPw) -> bool {
        match verify_slices_are_equal(self.as_slice(), other.as_slice()) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}