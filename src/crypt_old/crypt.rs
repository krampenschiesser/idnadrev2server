use ring::aead::{open_in_place, seal_in_place, OpeningKey, SealingKey, Algorithm, AES_256_GCM, CHACHA20_POLY1305};
use ring_pwhash::scrypt::{scrypt, ScryptParams};
use ring::constant_time::verify_slices_are_equal;
use std::time::{Instant};
use chrono::Duration;
use super::{EncryptionType, PasswordHashType, Repository};
use super::error::*;
use std::path::PathBuf;

#[derive(Clone)]
pub struct PlainPw {
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

impl <'a>From<&'a str> for PlainPw {
    fn from(i: &str) -> Self {
        PlainPw::new(i.as_bytes())
    }
}

impl From<String> for PlainPw {
    fn from(i: String) -> Self {
        PlainPw::new(i.as_bytes())
    }
}

#[derive(Clone)]
pub struct HashedPw {
    content: Vec<u8>
}

impl HashedPw {
    pub fn new(plain: PlainPw, enc_type: &EncryptionType, hash_type: &PasswordHashType) -> Self {
        let len = enc_type.key_len();
        let v = hash_type.hash(plain.as_slice(), len);
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

#[derive(Clone, Debug)]
pub struct DoubleHashedPw {
    content: Vec<u8>
}

impl DoubleHashedPw {
    pub fn new(hashed: &HashedPw, enc_type: &EncryptionType, hash_type: &PasswordHashType) -> Self {
        let len = enc_type.key_len();
        let v = hash_type.hash(hashed.as_slice(), len);
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

                debug!("Scrypt took {}s", Duration::from_std(now.elapsed()).unwrap().num_milliseconds());
                buff
            }
            _ => unimplemented!()
        }
    }
}

impl Repository {
    pub fn hash_key(&self, pw_plain: PlainPw) -> HashedPw {
        Repository::hash_key_ext(&self.header.encryption_type, &self.header.password_hash_type, pw_plain)
    }

    pub fn hash_pw(&self, pw: &HashedPw) -> DoubleHashedPw {
        Repository::hash_pw_ext(&self.header.encryption_type, &self.header.password_hash_type, pw)
    }

    pub fn hash_key_ext(enc_type: &EncryptionType, hash_type: &PasswordHashType, pw_plain: PlainPw) -> HashedPw {
        HashedPw::new(pw_plain, enc_type, hash_type)
    }

    pub fn hash_pw_ext(enc_type: &EncryptionType, hash_type: &PasswordHashType, pw: &HashedPw) -> DoubleHashedPw {
        DoubleHashedPw::new(pw, enc_type, hash_type)
    }

    pub fn check_plain_pw(&self, pw_plain: PlainPw) -> bool {
        let single = self.hash_key(pw_plain);
        let double = self.hash_pw(&single);

        double == self.hash
    }

    pub fn check_hashed_key(&self, pw: &HashedPw) -> bool {
        let double = self.hash_pw(&pw);
        double == self.hash
    }

    pub fn get_path(&self) -> Option<PathBuf> {
        self.path.clone()
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::{HashedPw, PlainPw};
    use super::super::{EncryptionType, PasswordHashType};

    fn hashed_key() -> HashedPw {
        let plainpw = PlainPw::new("hello".as_bytes());
        HashedPw::new(plainpw, &EncryptionType::RingChachaPoly1305, &PasswordHashType::SCrypt { iterations: 1, memory_costs: 1, parallelism: 1 })
    }

    fn nonce() -> Vec<u8> {
        super::super::random_vec(12)
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