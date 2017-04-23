use std::io::{Cursor, Read,Write};
use uuid::Uuid;
use byteorder::{ReadBytesExt, WriteBytesExt, LittleEndian};
use rand::os::OsRng;
use rand::Rng;
use super::*;

const UUID_LENGTH: usize = 16;

pub trait ByteSerialization: Sized {
    fn to_bytes(&self, vec: &mut Vec<u8>);
    fn from_bytes(input: &mut Cursor<&[u8]>) -> Result<Self, ParseError>;

    fn byte_len(&self) -> usize;
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
            _ => Err(ParseError::WrongValue(pos))
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
            _ => Err(ParseError::WrongValue(pos))
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
        let mut buff = vec![0u8; length as usize];
        read_buff(input, &mut buff.as_mut_slice())?;
        Ok(RepoHeader { salt: buff, encryption_type: enc_type, main_header: main_header, password_hash_type: pwh_type })
    }
    fn byte_len(&self) -> usize {
        self.main_header.byte_len() + self.encryption_type.byte_len() + self.password_hash_type.byte_len() + 1 + self.salt.len()
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

        let mut nonce_header = vec![0u8; nonce_header_len as usize];
        read_buff(input, nonce_header.as_mut_slice())?;

        let mut nonce_content = vec![0u8; nonce_content_len as usize];
        read_buff(input, nonce_content.as_mut_slice())?;

        Ok(FileHeader { main_header: main_header, repository_id: repo_id, encryption_type: enc_type, header_length: header_len, nonce_header: nonce_header, nonce_content: nonce_content })
    }
    fn byte_len(&self) -> usize {
        self.main_header.byte_len() + UUID_LENGTH + self.encryption_type.byte_len() + 2 + 4 + self.nonce_header.len() + self.nonce_content.len()
    }
}

impl ByteSerialization for Repository {
    fn to_bytes(&self, vec: &mut Vec<u8>) {
        self.header.to_bytes(vec);
        let hash_len = self.hash.len() as u8;
        vec.write_u8(hash_len);
        vec.write(self.hash.as_slice());
        vec.write(self.name.as_bytes());
    }

    fn from_bytes(input: &mut Cursor<&[u8]>) -> Result<Self, ParseError> {
        let h = RepoHeader::from_bytes(input)?;
        let hash_len = read_u8(input)?;
        let mut buff = vec![0u8; hash_len as usize];
        read_buff(input, &mut buff)?;
        let mut namebuff = Vec::new();
        input.read_to_end(&mut namebuff)?;
        let name = String::from_utf8(namebuff)?;
        Ok(Repository { header: h, hash: buff, name: name, path: None })
    }

    fn byte_len(&self) -> usize {
        unimplemented!()
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

    #[test]
    fn repo_header() {
        let kdf = PasswordHashType::SCrypt { iterations: 1, memory_costs: 2, parallelism: 3 };
        let h = RepoHeader::new(kdf, EncryptionType::RingChachaPoly1305);

        let mut result = Vec::new();
        h.to_bytes(&mut result);

        let main_header_len = 2 + 1 + UUID_LENGTH + 4;
        assert_eq!(main_header_len, h.main_header.byte_len());

        let enc_type_len = 1;
        assert_eq!(enc_type_len, h.encryption_type.byte_len());

        let kdf_len = 1 + 1 + 2 * 4;
        assert_eq!(kdf_len, h.password_hash_type.byte_len());

        let salt_len_field = 1;
        let salt_len = 32;
        assert_eq!(salt_len, h.salt.len());


        let expected_len = main_header_len + (enc_type_len) + (kdf_len) + (salt_len_field) + salt_len;
        assert_eq!(expected_len, h.byte_len());
        assert_eq!(expected_len, result.len());

        let parse_back = RepoHeader::from_bytes(&mut Cursor::new(result.as_slice())).unwrap();
        assert_eq!(h, parse_back);
    }

    #[test]
    fn file_header() {
        let rh = RepoHeader::new_for_test();
        let h = FileHeader::new(&rh);

        let mut result = Vec::new();
        h.to_bytes(&mut result);

        let main_header_len = 2 + 1 + UUID_LENGTH + 4;
        let repo_id = UUID_LENGTH;
        let enc_type_len = 1;
        let nonce1_len = 1;
        let nonce2_len = 1;
        let sizeof_header = 4;
        let expected_len = main_header_len + repo_id + enc_type_len + nonce1_len + nonce2_len + sizeof_header + 2 * 12;

        assert_eq!(expected_len, h.byte_len());
        assert_eq!(expected_len, result.len());

        let parse_back = FileHeader::from_bytes(&mut Cursor::new(result.as_slice())).unwrap();
        assert_eq!(h, parse_back);
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
    let mut buff = [0u8; UUID_LENGTH];
    input.read(&mut buff).map_err(|e| ParseError::IllegalPos(pos))?;
    Uuid::from_bytes(&buff).map_err(|e| ParseError::NoValidUuid(pos))
}