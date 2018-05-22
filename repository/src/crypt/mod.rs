use chacha20_poly1305_aead::{decrypt, encrypt};
use failure::Error;

pub type Nonce = [u8];
pub type AAD = [u8];

#[derive(Clone)]
pub struct HashedPw {
    content: Vec<u8>
}

#[derive(Clone, Debug)]
pub struct DoubleHashedPw {
    content: Vec<u8>
}

impl<'a> From<&'a[u8]> for DoubleHashedPw {
    fn from(data: &'a[u8]) -> Self {
        DoubleHashedPw {
            content: Vec::from(data),
        }
    }
}

pub enum EncryptionType {
    ChachaPoly1305,
}

impl EncryptionType {
    fn encrypt(key: &HashedPw, nonce: &Nonce, aad: &AAD, input: &[u8]) -> Result<Vec<u8>, Error> {
        let mut output = Vec::with_capacity(input.len());
        encrypt(key.content.as_ref(), nonce, aad, input, &mut output)?;
        Ok(output)
    }
}