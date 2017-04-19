use std::fs::File;
use std::io::{Cursor, Read};
use uuid::Uuid;
use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};


#[derive(Debug, Eq, PartialEq)]
enum ParseError {
    WrongValue(u64),
    IllegalPos(u64),
    NoPrefix,
    NoValidUuid(u64),
}

pub trait ByteSerialization: Sized {
    fn to_bytes(&self, vec: &mut Vec<u8>);
    fn from_bytes(input: &mut Cursor<&[u8]>) -> Result<Self, ParseError>;
}

#[derive(Debug, Eq, PartialEq)]
pub enum EncryptionType {
    None,
    RingChachaPoly1305,
    RingAESGCM,
}

#[derive(Debug, Eq, PartialEq)]
pub enum PasswordHashType {
    None,
    Argon2i { iterations: u16, memory_costs: u16, parallelism: u16 },
    SCrypt { iterations: u16, memory_costs: u16, parallelism: u16 },
}

#[derive(Debug, Eq, PartialEq)]
pub struct MainHeader {
    //beaf
    file_version: u8,
    id: Uuid,
    version: u32,
}

#[derive(Debug, Eq, PartialEq)]
pub struct RepoHeader {
    main_header: MainHeader,
    encryption_type: EncryptionType,
    password_hash_type: PasswordHashType,
    salt: Vec<u8>,
}

pub struct FileHeader {
    main_header: MainHeader,
    repository_id: Uuid,
    encryption_type: EncryptionType,
    //nonce header length
    //nonce content length
    header_length: u32,
    nonce_header: Vec<u8>,
    nonce_content: Vec<u8>,
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
            _ => Err(ParseError::WrongValue(pos))
        }
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
                vec.write_u16::<LittleEndian>(iterations);
                vec.write_u16::<LittleEndian>(memory_costs);
                vec.write_u16::<LittleEndian>(parallelism);
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
                let iterations = read_u16(input)?;
                let mem = read_u16(input)?;
                let cpu = read_u16(input)?;
                Ok(PasswordHashType::SCrypt { iterations: iterations, memory_costs: mem, parallelism: cpu })
            }
            _ => Err(ParseError::WrongValue(pos))
        }
    }
}

impl ByteSerialization for MainHeader {
    fn to_bytes(&self, vec: &mut Vec<u8>) {
        vec.push(0xBE);
        vec.push(0xAF);
        vec.push(self.file_version);
        vec.extend_from_slice(self.id.as_bytes());
        vec.write_u32::<LittleEndian>(self.version);
    }

    fn from_bytes(input: &mut Cursor<&[u8]>) -> Result<Self, ParseError> {
        let b1 = read_u8(input)?;
        let b2 = read_u8(input)?;
        if b1 != 0xBE || b2 != 0xAF {
            return Err(ParseError::NoPrefix);
        }

        let file_version = read_u8(input)?;
        let id = read_uuid(input)?;
        let version = read_u32(input)?;

        Ok(MainHeader { id: id, version: version, file_version: file_version })
    }
}

impl ByteSerialization for RepoHeader {
    fn to_bytes(&self, vec: &mut Vec<u8>) {
        self.main_header.to_bytes(vec);
        self.encryption_type.to_bytes(vec);
        self.password_hash_type.to_bytes(vec);

        let salt_len = self.salt.len() as u8;
        vec.push(salt_len);
        vec.append(&mut self.salt.clone());
    }

    fn from_bytes(input: &mut Cursor<&[u8]>) -> Result<Self, ParseError> {
        let main_header = MainHeader::from_bytes(input)?;
        let enc_type = EncryptionType::from_bytes(input)?;
        let pwh_type = PasswordHashType::from_bytes(input)?;
        let length = read_u8(input)?;
        let mut buff = Vec::with_capacity(length as usize);
        read_buff(input, &mut buff.as_mut_slice())?;
        Ok(RepoHeader { salt: buff, encryption_type: enc_type, main_header: main_header, password_hash_type: pwh_type })
    }
}

impl ByteSerialization for FileHeader {
    fn to_bytes(&self, vec: &mut Vec<u8>) {
        self.main_header.to_bytes(vec);
        vec.extend_from_slice(self.repository_id.as_bytes());
        self.encryption_type.to_bytes(vec);

        let nonce_header_len = self.nonce_header.len() as u8;
        let nonce_content_len = self.nonce_content.len() as u8;
        vec.write_u8(nonce_header_len);
        vec.write_u8(nonce_content_len);
        vec.write_u32::<LittleEndian>(self.header_length);
        vec.append(&mut self.nonce_header.clone());
        vec.append(&mut self.nonce_content.clone());
    }

    fn from_bytes(input: &mut Cursor<&[u8]>) -> Result<Self, ParseError> {
        let main_header = MainHeader::from_bytes(input)?;
        let repo_id = read_uuid(input)?;
        let enc_type = EncryptionType::from_bytes(input)?;

        let nonce_header_len = read_u8(input)?;
        let nonce_content_len = read_u8(input)?;
        let header_len = read_u32(input)?;

        let mut nonce_header = Vec::with_capacity(nonce_header_len as usize);
        read_buff(input, nonce_header.as_mut_slice())?;

        let mut nonce_content = Vec::with_capacity(nonce_content_len as usize);
        read_buff(input, nonce_content.as_mut_slice())?;

        Ok(FileHeader { main_header: main_header, repository_id: repo_id, encryption_type: enc_type, header_length: header_len, nonce_header: nonce_header, nonce_content: nonce_content })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enc_type() {
        let mut vec: Vec<u8> = Vec::new();
        EncryptionType::RingAESGCM.to_bytes(&mut vec);
        assert_eq! (1, vec.len());
        assert_eq!(2, vec[0]);

        assert_eq! (EncryptionType::None, EncryptionType::from_bytes(&mut Cursor::new(&[0])).unwrap());
        assert_eq!(EncryptionType::RingChachaPoly1305, EncryptionType::from_bytes(&mut Cursor::new(&[1])).unwrap());
        assert_eq! (EncryptionType::RingAESGCM, EncryptionType::from_bytes(&mut Cursor::new(&[2])).unwrap());

        assert_eq! (Some(ParseError::WrongValue(0)), EncryptionType::from_bytes(&mut Cursor::new(&[42])).err());
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
        let header = MainHeader { file_version: 42, id: id.clone(), version: 8 };
        let mut result = Vec::new();
        header.to_bytes(&mut result);

        let mut expected = Vec::new();
        expected.push(0xBE);
        expected.push(0xAF);
        expected.push(0x2A);
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

        assert_eq!(7, result.len());
        let pwh2 = PasswordHashType::from_bytes(&mut Cursor::new(result.as_slice())).unwrap();
        assert_eq!(pwh, pwh2);
    }
}

fn read_u8(input: &mut Cursor<&[u8]>) -> Result<u8, ParseError> {
    let pos = input.position();
    input.read_u8().map_err(|e| ParseError::IllegalPos(pos))
}

fn read_u16(input: &mut Cursor<&[u8]>) -> Result<u16, ParseError> {
    let pos = input.position();
    input.read_u16::<LittleEndian>().map_err(|e| ParseError::IllegalPos(pos))
}

fn read_u32(input: &mut Cursor<&[u8]>) -> Result<u32, ParseError> {
    let pos = input.position();
    input.read_u32::<LittleEndian>().map_err(|e| ParseError::IllegalPos(pos))
}

fn read_buff<'a>(input: &mut Cursor<&[u8]>, buff: &'a mut [u8]) -> Result<&'a [u8], ParseError> {
    let pos = input.position();
    input.read(buff).map_err(|e| ParseError::IllegalPos(pos))?;
    Ok(buff)
}

fn read_uuid(input: &mut Cursor<&[u8]>) -> Result<Uuid, ParseError> {
    let pos = input.position();
    let mut buff = [0u8; 16];
    input.read(&mut buff).map_err(|e| ParseError::IllegalPos(pos))?;
    Uuid::from_bytes(&buff).map_err(|e| ParseError::NoValidUuid(pos))
}