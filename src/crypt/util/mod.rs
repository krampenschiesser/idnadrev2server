use super::structs::EncryptionType;
use super::structs::crypto::HashedPw;
use super::error::RingError;
use ring::aead::{open_in_place, seal_in_place, OpeningKey, SealingKey, Algorithm};
use rand::{OsRng,Rng};

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
