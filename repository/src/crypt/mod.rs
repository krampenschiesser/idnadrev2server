use chacha20_poly1305_aead::{decrypt, encrypt};
use failure::Error;

pub type Nonce = [u8];
pub type AAD = [u8];
pub type VerificationTag = [u8; 16];
pub type PlaintextVec = Vec<u8>;
pub type CipherTextVec = Vec<u8>;
pub type Plaintext = [u8];
pub type CipherText = [u8];

#[derive(Clone)]
pub struct HashedPw {
    content: Vec<u8>
}

#[derive(Clone, Debug)]
pub struct DoubleHashedPw {
    content: Vec<u8>
}

impl<'a> From<&'a [u8]> for DoubleHashedPw {
    fn from(data: &'a [u8]) -> Self {
        DoubleHashedPw {
            content: Vec::from(data),
        }
    }
}

pub trait DeEncrypter {
    fn encrypt(key: &HashedPw, nonce: &Nonce, aad: &AAD, input: &Plaintext) -> Result<(CipherTextVec, VerificationTag), Error>;
    fn decrypt(key: &HashedPw, nonce: &Nonce, aad: &AAD, tag: &VerificationTag, input: &CipherText) -> Result<PlaintextVec, Error>;
}

impl DeEncrypter for ::pb::file::EncryptionType {
    fn encrypt(key: &HashedPw, nonce: &Nonce, aad: &AAD, input: &Plaintext) -> Result<(CipherTextVec, VerificationTag), Error> {
        let mut output = Vec::with_capacity(input.len());
        let verification_tag = encrypt(key.content.as_ref(), nonce, aad, input, &mut output)?;
        Ok((output, verification_tag))
    }

    fn decrypt(key: &HashedPw, nonce: &Nonce, aad: &AAD, tag: &VerificationTag, input: &CipherText) -> Result<PlaintextVec, Error> {
        let mut output = Vec::with_capacity(input.len());
        decrypt(key.content.as_ref(), nonce, aad, input, tag, &mut output)?;
        Ok(output)
    }
}