use std::fmt::{Display, Formatter};
use ring_pwhash::scrypt::{scrypt, ScryptParams};
use ring::constant_time::verify_slices_are_equal;
use ring::aead::{AES_256_GCM, CHACHA20_POLY1305, Algorithm};
use std::time::{Instant};
use chrono::Duration;
use uuid::Uuid;
use std::fmt;
use rand::{OsRng, Rng};
use byteorder::{WriteBytesExt, LittleEndian};
use self::serialize::*;
use std::io::{Read, Write, Cursor};
use super::error::{ParseError};
use super::actor::dto::{EncTypeDto, PwKdfDto};


pub mod crypto;
pub mod file;
pub mod repository;
pub mod serialize;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum EncryptionType {
    None,
    RingChachaPoly1305,
    RingAESGCM,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum PasswordHashType {
    None,
    Argon2i { iterations: u16, memory_costs: u16, parallelism: u16 },
    SCrypt { iterations: u8, memory_costs: u32, parallelism: u32 },
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum FileVersion {
    RepositoryV1,
    FileV1,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct MainHeader {
    //beaf
    file_version: FileVersion,
    id: Uuid,
    version: u32,
}


impl Display for EncryptionType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            EncryptionType::None => write!(f, "None"),
            EncryptionType::RingChachaPoly1305 => write!(f, "ChachaPoly1305"),
            EncryptionType::RingAESGCM => write!(f, "AesGcm"),
        }
    }
}

impl Display for FileVersion {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            FileVersion::RepositoryV1 => write!(f, "Repository-Version 1"),
            FileVersion::FileV1 => write!(f, "File-Version 1"),
        }
    }
}


impl PasswordHashType {
    pub fn salt_len(&self) -> usize {
        match *self {
            PasswordHashType::SCrypt { iterations, memory_costs, parallelism } => 32,
            PasswordHashType::Argon2i { iterations, memory_costs, parallelism } => 32,
            PasswordHashType::None => 0,
        }
    }
    pub fn hash(&self, input: &[u8], salt: &[u8], len: usize) -> Vec<u8> {
        match *self {
            PasswordHashType::None => input.to_vec(),
            PasswordHashType::SCrypt { iterations, memory_costs, parallelism } => {
                let mut buff = vec![0u8; len];
                let param = ScryptParams::new(iterations, memory_costs, parallelism);
                let now = Instant::now();
                scrypt(input, salt, &param, buff.as_mut_slice());

                debug!("Scrypt took {}s", Duration::from_std(now.elapsed()).unwrap().num_milliseconds());
                buff
            }
            _ => unimplemented!()
        }
    }
}

impl<'a> From<&'a PwKdfDto> for PasswordHashType {
    fn from(dto: &PwKdfDto) -> Self {
        match dto {
            &PwKdfDto::SCrypt { iterations, memory_costs, parallelism } => PasswordHashType::SCrypt { iterations: iterations, memory_costs: memory_costs, parallelism: parallelism },
        }
    }
}

impl EncryptionType {
    pub fn key_len(&self) -> usize {
        match *self {
            EncryptionType::RingAESGCM => 32,
            EncryptionType::RingChachaPoly1305 => 32,
            EncryptionType::None => 0,
        }
    }

    pub fn nonce_len(&self) -> usize {
        match *self {
            EncryptionType::RingAESGCM => 12,
            EncryptionType::RingChachaPoly1305 => 12,
            EncryptionType::None => 0,
        }
    }

    pub fn hash_len(&self) -> usize {
        match *self {
            EncryptionType::RingAESGCM => 16,
            EncryptionType::RingChachaPoly1305 => 16,
            EncryptionType::None => 0,
        }
    }
    pub fn algorithm(&self) -> Option<&'static Algorithm> {
        match *self {
            EncryptionType::RingAESGCM => Some(&AES_256_GCM),
            EncryptionType::RingChachaPoly1305 => Some(&CHACHA20_POLY1305),
            EncryptionType::None => None,
        }
    }
}

impl<'a> From<&'a EncTypeDto> for EncryptionType {
    fn from(dto: &EncTypeDto) -> Self {
        match dto {
            &EncTypeDto::AES => EncryptionType::RingAESGCM,
            &EncTypeDto::ChaCha => EncryptionType::RingChachaPoly1305,
        }
    }
}


impl MainHeader {
    pub fn new(file_version: FileVersion) -> Self {
        let id = Uuid::new_v4();
        MainHeader { id: id, version: 0, file_version: file_version }
    }

    pub fn get_file_version(&self) -> &FileVersion {
        &self.file_version
    }
}

fn random_vec(len: usize) -> Vec<u8> {
    let mut rng = OsRng::new().unwrap();
    let mut salt = vec![0u8; len];

    rng.fill_bytes(salt.as_mut_slice());
    salt
}


impl ByteSerialization for FileVersion {
    fn to_bytes(&self, vec: &mut Vec<u8>) {
        match *self {
            FileVersion::RepositoryV1 => vec.push(1u8),
            FileVersion::FileV1 => vec.push(0u8),
        }
    }

    fn from_bytes(input: &mut Cursor<&[u8]>) -> Result<Self, ParseError> {
        let x = read_u8(input)?;
        match x {
            0 => Ok(FileVersion::FileV1),
            1 => Ok(FileVersion::RepositoryV1),
            _ => Err(ParseError::UnknownFileVersion(x)),
        }
    }
    fn byte_len(&self) -> usize {
        1
    }
}

impl ByteSerialization for EncryptionType {
    fn to_bytes(&self, vec: &mut Vec<u8>) {
        let val = match *self {
            EncryptionType::None => 0u8,
            EncryptionType::RingChachaPoly1305 => 1u8,
            EncryptionType::RingAESGCM => 2u8,
        };
        vec.push(val)
    }

    fn from_bytes(input: &mut Cursor<&[u8]>) -> Result<Self, ParseError> {
        let pos = input.position();
        let x = read_u8(input)?;
        match x {
            0 => Ok(EncryptionType::None),
            1 => Ok(EncryptionType::RingChachaPoly1305),
            2 => Ok(EncryptionType::RingAESGCM),
            _ => Err(ParseError::WrongValue(pos, x))
        }
    }
    fn byte_len(&self) -> usize {
        1
    }
}

impl ByteSerialization for PasswordHashType {
    fn to_bytes(&self, vec: &mut Vec<u8>) {
        match *self {
            PasswordHashType::None => {
                vec.push(0u8);
            }
            PasswordHashType::Argon2i { iterations, memory_costs, parallelism } => {
                vec.push(1u8);
                vec.write_u16::<LittleEndian>(iterations);
                vec.write_u16::<LittleEndian>(memory_costs);
                vec.write_u16::<LittleEndian>(parallelism);
            }
            PasswordHashType::SCrypt { iterations, memory_costs, parallelism } => {
                vec.push(2u8);
                vec.write_u8(iterations);
                vec.write_u32::<LittleEndian>(memory_costs);
                vec.write_u32::<LittleEndian>(parallelism);
            }
        };
    }

    fn from_bytes(input: &mut Cursor<&[u8]>) -> Result<Self, ParseError> {
        let pos = input.position();
        let x = read_u8(input)?;

        match x {
            0 => Ok(PasswordHashType::None),
            1 => {
                let iterations = read_u16(input)?;
                let mem = read_u16(input)?;
                let cpu = read_u16(input)?;
                Ok(PasswordHashType::Argon2i { iterations: iterations, memory_costs: mem, parallelism: cpu })
            }
            2 => {
                let iterations = read_u8(input)?;
                let mem = read_u32(input)?;
                let cpu = read_u32(input)?;
                Ok(PasswordHashType::SCrypt { iterations: iterations, memory_costs: mem, parallelism: cpu })
            }
            _ => Err(ParseError::WrongValue(pos, x))
        }
    }
    fn byte_len(&self) -> usize {
        match *self {
            PasswordHashType::None => 1,
            PasswordHashType::Argon2i { .. } => 1 + 2 * 3,
            PasswordHashType::SCrypt { .. } => 1 + 1 + 2 * 4,
        }
    }
}

impl ByteSerialization for MainHeader {
    fn to_bytes(&self, vec: &mut Vec<u8>) {
        vec.push(0xBE);
        vec.push(0xAF);
        self.file_version.to_bytes(vec);
        vec.extend_from_slice(self.id.as_bytes());
        vec.write_u32::<LittleEndian>(self.version);
    }

    fn from_bytes(input: &mut Cursor<&[u8]>) -> Result<Self, ParseError> {
        let b1 = read_u8(input)?;
        let b2 = read_u8(input)?;
        if b1 != 0xBE || b2 != 0xAF {
            return Err(ParseError::NoPrefix);
        }

        let file_version = FileVersion::from_bytes(input)?;
        let id = read_uuid(input)?;
        let version = read_u32(input)?;

        Ok(MainHeader { id: id, version: version, file_version: file_version })
    }
    fn byte_len(&self) -> usize {
        2 + 1 + UUID_LENGTH + 4
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;
    use super::crypto::*;

    #[test]
    fn enc_type() {
        let mut vec: Vec<u8> = Vec::new();
        EncryptionType::RingAESGCM.to_bytes(&mut vec);
        assert_eq! (1, vec.len());
        assert_eq!(2, vec[0]);

        assert_eq! (EncryptionType::None, EncryptionType::from_bytes(&mut Cursor::new(&[0])).unwrap());
        assert_eq!(EncryptionType::RingChachaPoly1305, EncryptionType::from_bytes(&mut Cursor::new(&[1])).unwrap());
        assert_eq! (EncryptionType::RingAESGCM, EncryptionType::from_bytes(&mut Cursor::new(&[2])).unwrap());

        assert_eq! (Some(ParseError::WrongValue(0, 42)), EncryptionType::from_bytes(&mut Cursor::new(&[42])).err());
    }

    #[test]
    fn enc_type_and_pw_type() {
        let mut vec: Vec<u8> = Vec::new();
        EncryptionType::RingAESGCM.to_bytes(&mut vec);
        PasswordHashType::None.to_bytes(&mut vec);
        assert_eq! (2, vec.len());
        assert_eq! (2, vec[0]);
        assert_eq! (0, vec[1]);
    }

    #[test]
    fn main_header() {
        let id = Uuid::nil();
        let header = MainHeader { file_version: FileVersion::RepositoryV1, id: id.clone(), version: 8 };
        let mut result = Vec::new();
        header.to_bytes(&mut result);

        let mut expected = Vec::new();
        expected.push(0xBE);
        expected.push(0xAF);
        expected.push(0x01);
        for i in 0..16 {
            expected.push(0u8);
        }
        expected.push(0x08);
        expected.push(0x00);
        expected.push(0x00);
        expected.push(0x00);

        assert_eq!(expected.len(), result.len());
        assert_eq! (expected, result);

        let mut c = Cursor::new(result.as_slice());
        let reparsed = MainHeader::from_bytes(&mut c).unwrap();
        assert_eq!(header, reparsed);
    }

    #[test]
    fn main_header_wrong_prefix() {
        let mut v = Vec::new();
        v.push(0xBE);
        v.push(0xAA);

        let error = MainHeader::from_bytes(&mut Cursor::new(v.as_slice()));
        assert_eq!(Err(ParseError::NoPrefix), error);
    }

    #[test]
    fn main_header_too_short() {
        let mut v = Vec::new();
        v.push(0xBE);
        v.push(0xAF);
        v.push(0x00);

        let error = MainHeader::from_bytes(&mut Cursor::new(v.as_slice()));
        assert_eq!(Err(ParseError::IllegalPos(3)), error);
    }

    #[test]
    fn pw_hash_type() {
        let pwh = PasswordHashType::SCrypt { iterations: 1, parallelism: 3, memory_costs: 2 };
        let mut result = Vec::new();
        pwh.to_bytes(&mut result);

        assert_eq!(10, result.len());
        let pwh2 = PasswordHashType::from_bytes(&mut Cursor::new(result.as_slice())).unwrap();
        assert_eq!(pwh, pwh2);
    }
}

