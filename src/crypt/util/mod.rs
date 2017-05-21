// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use dto::EncryptionType;
use super::structs::crypto::HashedPw;
use super::error::RingError;
use ring::aead::{open_in_place, seal_in_place, OpeningKey, SealingKey, Algorithm};
use rand::{OsRng, Rng};

pub mod io;
pub mod tempfile;

pub fn encrypt(enctype: &EncryptionType, nonce: &[u8], key: &HashedPw, data: &[u8], additional: &[u8]) -> Result<Vec<u8>, RingError> {
    let alg = enctype.algorithm();
    match alg {
        None => Ok(data.to_vec()),
        Some(a) => {
            let key = SealingKey::new(a, key.as_slice()).map_err(|_| RingError::KeyFailure)?;
            let mut ciphertext = data.to_vec();
            ciphertext.resize(data.len() + enctype.hash_len(), 0);
            seal_in_place(&key, nonce, additional, ciphertext.as_mut_slice(), enctype.hash_len()).map_err(|_| RingError::EncryptFailue)?;
            Ok(ciphertext)
        }
    }
}

pub fn decrypt(enctype: &EncryptionType, nonce: &[u8], key: &HashedPw, data: &[u8], additional: &[u8]) -> Result<Vec<u8>, RingError> {
    let alg = enctype.algorithm();
    match alg {
        None => Ok(data.to_vec()),
        Some(a) => {
            let key = OpeningKey::new(a, key.as_slice()).map_err(|_| RingError::KeyFailure)?;
            let mut ciphertext = data.to_vec();
            open_in_place(&key, nonce, additional, 0, ciphertext.as_mut_slice()).map_err(|_| RingError::DecryptFailue)?;
            let content_length = ciphertext.len() - enctype.hash_len();
            ciphertext.resize(content_length, 0);
            Ok(ciphertext)
        }
    }
}

pub fn random_vec(len: usize) -> Vec<u8> {
    let mut rng = OsRng::new().unwrap();
    let mut salt = vec![0u8; len];

    rng.fill_bytes(salt.as_mut_slice());
    salt
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::super::structs::crypto::{HashedPw, PlainPw};
    use dto::{EncryptionType, PasswordHashType};

    fn hashed_key() -> HashedPw {
        let plainpw = PlainPw::new("hello".as_bytes());
        let salt ="hello".as_bytes();
        HashedPw::new(plainpw, &EncryptionType::RingChachaPoly1305, &PasswordHashType::SCrypt { iterations: 1, memory_costs: 1, parallelism: 1 },salt)
    }

    fn nonce() -> Vec<u8> {
        random_vec(12)
    }

    fn encrypt_decrypt<F, A>(enctype: EncryptionType, mut ciphertext_mod: F, expect_error: bool, additional_mod: A)
        where F: FnMut(Vec<u8>) -> Vec<u8>, A: Fn(&str) -> &str {
        info!("Using: {:?}", enctype);
        let plaintext = "Hello Sauerland!";
        let additional = "Murks";
        let data = plaintext.as_bytes();
        let additional_data = additional.as_bytes();
        let nonce = nonce();

        info!("Input: {:?}", plaintext);
        info!("Data: {:?}", data);
        let ciphertext = encrypt(&enctype, nonce.as_slice(), &hashed_key(), data, additional_data).unwrap();
        info!("Ciphertext: {:?}", ciphertext);
        let ciphertext = ciphertext_mod(ciphertext);
        info!("Ciphertext modified: {:?}", ciphertext);
        let decrypt_result = decrypt(&enctype, nonce.as_slice(), &hashed_key(), ciphertext.as_slice(), additional_mod(additional).as_bytes());
        if expect_error {
            match decrypt_result {
                Err(RingError::DecryptFailue) => return,
                Err(_) => panic!("Should not happen at all. Decryption should have failed"),
                Ok(_) => panic!("Decryption should have failed but was OK"),
            }
        }

        let result_data = decrypt_result.unwrap();
        let result = String::from_utf8(result_data).unwrap();
        info!("Result {:?}", result);

        let mut ciphertext_shortened = ciphertext.clone();
        ciphertext_shortened.resize(data.len(), 0);
        if enctype != EncryptionType::None {
            assert_ne!(data, ciphertext_shortened.as_slice());
        }
        assert_eq!(plaintext, result);
    }

    #[test]
    fn chacha() {
        encrypt_decrypt(EncryptionType::RingChachaPoly1305, |e| e, false, |e| e);
    }

    #[test]
    fn aes() {
        encrypt_decrypt(EncryptionType::RingAESGCM, |e| e, false, |e| e);
    }

    #[test]
    fn none() {
        encrypt_decrypt(EncryptionType::None, |e| e, false, |e| e);
    }

    #[test]
    fn modify_hash() {
        let modifier = |mut v: Vec<u8>| {
            let len = v.len();
            v[len - 1] = 0;
            v
        };
        encrypt_decrypt(EncryptionType::RingChachaPoly1305, modifier, true, |e| e);
    }

    #[test]
    fn modify_ciphertext() {
        let modifier = |mut v: Vec<u8>| {
            v[0] = 0;
            v
        };
        encrypt_decrypt(EncryptionType::RingChachaPoly1305, modifier, true, |e| e);
    }

    #[test]
    fn modify_additional_data() {
        encrypt_decrypt(EncryptionType::RingChachaPoly1305, |e| e, true, |_| "bla");
    }
}