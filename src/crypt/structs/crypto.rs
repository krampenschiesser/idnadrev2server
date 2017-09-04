// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use dto::{PlainPw, EncryptionType, PasswordHashType};
use ring::constant_time::verify_slices_are_equal;


#[derive(Clone)]
pub struct HashedPw {
    content: Vec<u8>
}

#[derive(Clone, Debug)]
pub struct DoubleHashedPw {
    content: Vec<u8>
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