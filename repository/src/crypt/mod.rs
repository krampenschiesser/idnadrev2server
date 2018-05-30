use chacha20_poly1305_aead::{decrypt, encrypt};
use error::ErrorKind;
use failure::Error;
use std::ops::Deref;

pub type Nonce = [u8];
pub type AAD = [u8];
pub type Salt = [u8];
pub type VerificationTag = [u8];
pub type VerificationTagVec = Vec<u8>;
pub type PlaintextVec = Vec<u8>;
pub type CipherTextVec = Vec<u8>;
pub type Plaintext = [u8];
pub type CipherText = [u8];

#[derive(Clone)]
pub struct HashedPw {
    pub content: Vec<u8>
}

#[derive(Clone, Debug)]
pub struct DoubleHashedPw {
    pub content: Vec<u8>
}

impl Deref for HashedPw {
    type Target = Vec<u8>;

    fn deref(&self) -> &Vec<u8> {
        &self.content
    }
}

impl Deref for DoubleHashedPw {
    type Target = Vec<u8>;

    fn deref(&self) -> &Vec<u8> {
        &self.content
    }
}

impl<'a> From<&'a [u8]> for HashedPw {
    fn from(data: &'a [u8]) -> Self {
        HashedPw {
            content: Vec::from(data),
        }
    }
}


impl<'a> From<&'a [u8]> for DoubleHashedPw {
    fn from(data: &'a [u8]) -> Self {
        DoubleHashedPw {
            content: Vec::from(data),
        }
    }
}

pub trait DeEncrypter {
    fn encrypt(&self, key: &HashedPw, nonce: &Nonce, aad: &AAD, input: &Plaintext) -> Result<(CipherTextVec, VerificationTagVec), Error>;
    fn decrypt(&self, key: &HashedPw, nonce: &Nonce, aad: &AAD, tag: &VerificationTag, input: &CipherText) -> Result<PlaintextVec, Error>;
}

pub trait Hasher {
    fn hash_pw(&self, bytes: &Plaintext, salt: &Salt) -> HashedPw;
    fn double_hash_pw(&self, bytes: &Plaintext, salt: &Salt) -> DoubleHashedPw;
}

pub trait AuthTagProvider {
    fn get_auth_tag<'a, 'b>(&'b self, data: &'a [u8]) -> Result<(&'a [u8], &'a VerificationTag), Error>;
}


fn check_nonce(nonce: &Nonce) -> Result<(), Error> {
    if nonce.len() != 12 {
        let error_kind = ErrorKind::InvalidNonceLength { expected_size: 12, real_size: nonce.len() };
        return Err(Error::from(error_kind));
    }
    Ok(())
}

impl DeEncrypter for ::pb::file::EncryptionType {
    fn encrypt(&self, key: &HashedPw, nonce: &Nonce, aad: &AAD, input: &Plaintext) -> Result<(CipherTextVec, VerificationTagVec), Error> {
        check_nonce(nonce)?;
        let mut output = Vec::with_capacity(input.len());
        let verification_tag = encrypt(key.content.as_ref(), nonce, aad, input, &mut output)?;
        Ok((output, verification_tag.to_vec()))
    }

    fn decrypt(&self, key: &HashedPw, nonce: &Nonce, aad: &AAD, tag: &VerificationTag, input: &CipherText) -> Result<PlaintextVec, Error> {
        check_nonce(nonce)?;
        let mut output = Vec::with_capacity(input.len());
        decrypt(key.content.as_ref(), nonce, aad, input, tag, &mut output)?;
        Ok(output)
    }
}

impl AuthTagProvider for ::pb::file::EncryptionType {
    fn get_auth_tag<'a, 'b>(&'b self, data: &'a [u8]) -> Result<(&'a [u8], &'a VerificationTag), Error> {
        use error::ErrorKind;
        let tag_length = 16;
        if data.len() > tag_length {
            Ok(data.split_at(data.len() - tag_length))
        } else {
            Err(Error::from(ErrorKind::DataTooShort { msg: "provided data for auth tag".into(), expected_size: 16, real_size: data.len() }))
        }
    }
}

impl Hasher for ::pb::file::PasswordHashType {
    fn hash_pw(&self, bytes: &Plaintext, salt: &Salt) -> HashedPw {
        use argon2rs::{Argon2, Variant};

        let mut out = [0; 32];
        let a2 = Argon2::default(Variant::Argon2i);
        a2.hash(&mut out, bytes, salt, &[], &[]);
        HashedPw::from(out.as_ref())
    }
    fn double_hash_pw(&self, bytes: &Plaintext, salt: &Salt) -> DoubleHashedPw {
        let hash = self.hash_pw(bytes, salt);
        let hash = self.hash_pw(hash.as_ref(), salt);
        DoubleHashedPw{content: hash.content}
    }
}


#[cfg(test)]
mod test {
    use pb::file::EncryptionType;
    use super::*;

    #[test]
    fn test_auth_tag() {
        let data = [3u8; 20];
        assert_eq!(20, data.len());
        let (data, tag) = EncryptionType::ChachaPoly1305.get_auth_tag(&data).unwrap();
        assert_eq!(4, data.len());
        assert_eq!(16, tag.len());
    }

    #[test]
    fn test_auth_tag_too_short() {
        let data = [3u8, 12];
        assert!(EncryptionType::ChachaPoly1305.get_auth_tag(&data).is_err());
    }
}