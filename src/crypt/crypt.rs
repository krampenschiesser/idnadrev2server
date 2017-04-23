use ring::aead::{open_in_place, seal_in_place, OpeningKey, SealingKey, Algorithm, AES_256_GCM, CHACHA20_POLY1305};
use ring_pwhash::scrypt::{scrypt, ScryptParams};
use ring::constant_time::verify_slices_are_equal;
use super::{EncryptionType, PasswordHashType, Repository};
use std::time::{Instant};
use chrono::Duration;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum CryptError {
    KeyFailure,
    DecryptFailue,
    EncryptFailue,
}

pub fn encrypt(enctype: &EncryptionType, nonce: &[u8], key_data: &[u8], data: &[u8], additional: &[u8]) -> Result<Vec<u8>, CryptError> {
    let alg = enctype.algorithm();
    match alg {
        None => Ok(data.to_vec()),
        Some(a) => {
            let key = SealingKey::new(a, key_data).map_err(|e| CryptError::KeyFailure)?;
            let mut ciphertext = data.to_vec();
            ciphertext.resize(data.len() + enctype.hash_len(), 0);
            seal_in_place(&key, nonce, additional, ciphertext.as_mut_slice(), enctype.hash_len()).map_err(|e| CryptError::EncryptFailue)?;
            Ok(ciphertext)
        }
    }
}

pub fn decrypt(enctype: &EncryptionType, nonce: &[u8], key_data: &[u8], data: &[u8], additional: &[u8]) -> Result<Vec<u8>, CryptError> {
    let alg = enctype.algorithm();
    match alg {
        None => Ok(data.to_vec()),
        Some(a) => {
            let key = OpeningKey::new(a, key_data).map_err(|e| CryptError::KeyFailure)?;
            let mut ciphertext = data.to_vec();
            open_in_place(&key, nonce, additional, 0, ciphertext.as_mut_slice()).map_err(|e| CryptError::DecryptFailue)?;
            let content_length = ciphertext.len() - enctype.hash_len();
            ciphertext.resize(content_length, 0);
            Ok(ciphertext)
        }
    }
}


impl EncryptionType {
    pub fn algorithm(&self) -> Option<&'static Algorithm> {
        match *self {
            EncryptionType::RingAESGCM => Some(&AES_256_GCM),
            EncryptionType::RingChachaPoly1305 => Some(&CHACHA20_POLY1305),
            EncryptionType::None => None,
        }
    }
}

impl PasswordHashType {
    pub fn hash(&self, input: &[u8], len: usize) -> Vec<u8> {
        match *self {
            PasswordHashType::None => input.to_vec(),
            PasswordHashType::SCrypt { iterations, memory_costs, parallelism } => {
                let mut buff = vec![0u8; len];
                let param = ScryptParams::new(iterations, memory_costs, parallelism);
                let now = Instant::now();
                scrypt(input, input, &param, buff.as_mut_slice());

                println!("Scrypt took {}s", Duration::from_std(now.elapsed()).unwrap().num_milliseconds());
                buff
            }
            _ => unimplemented!()
        }
    }
}

impl Repository {
    pub fn check_pw(&self, pw: &[u8]) -> bool {
        let len = self.header.encryption_type.hash_len();
        let ref kdf = self.header.password_hash_type;
        let v = kdf.hash(pw, len);
        let checksum = kdf.hash(v.as_slice(), len);

        match verify_slices_are_equal(checksum.as_slice(), self.hash.as_slice()) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use ring_pwhash::scrypt::{scrypt, ScryptParams};
    use rand::os::OsRng;
    use rand::Rng;

    fn key_data() -> Vec<u8> {
        let pwh = super::super::PasswordHashType::SCrypt { iterations: 3, memory_costs: 2, parallelism: 1 };
        pwh.hash("hello".as_bytes(), 32)
    }

    fn nonce() -> Vec<u8> {
        super::super::random_vec(12)
    }

    fn encrypt_decrypt<F, A>(enctype: EncryptionType, mut ciphertext_mod: F, expect_error: bool, additional_mod: A)
        where F: FnMut(Vec<u8>) -> Vec<u8>, A: Fn(&str) -> &str {
        println!("Using: {:?}", enctype);
        let plaintext = "Hello Sauerland!";
        let additional = "Murks";
        let data = plaintext.as_bytes();
        let additional_data = additional.as_bytes();
        let nonce = nonce();

        println!("Input: {:?}", plaintext);
        println!("Data: {:?}", data);
        let mut ciphertext = encrypt(&enctype, nonce.as_slice(), key_data().as_slice(), data, additional_data).unwrap();
        println!("Ciphertext: {:?}", ciphertext);
        let mut ciphertext = ciphertext_mod(ciphertext);
        println!("Ciphertext modified: {:?}", ciphertext);
        let decrypt_result = decrypt(&enctype, nonce.as_slice(), key_data().as_slice(), ciphertext.as_slice(), additional_mod(additional).as_bytes());
        if expect_error {
            match decrypt_result {
                Err(CryptError::DecryptFailue) => return,
                Err(_) => panic!("Should not happen at all. Decryption should have failed"),
                Ok(_) => panic!("Decryption should have failed but was OK"),
            }
        }

        let result_data = decrypt_result.unwrap();
        let result = String::from_utf8(result_data).unwrap();
        println!("Result {:?}", result);

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
            let len = v.len();
            v[0] = 0;
            v
        };
        encrypt_decrypt(EncryptionType::RingChachaPoly1305, modifier, true, |e| e);
    }

    #[test]
    fn modify_additional_data() {
        encrypt_decrypt(EncryptionType::RingChachaPoly1305, |e| e, true, |i| "bla");
    }
}