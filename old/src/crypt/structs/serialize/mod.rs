// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::io::{Cursor, Read};
use uuid::Uuid;
use byteorder::{ReadBytesExt, LittleEndian};
use super::super::error::*;

pub const UUID_LENGTH: usize = 16;


pub trait ByteSerialization: Sized {
    fn to_bytes(&self, vec: &mut Vec<u8>);
    fn from_bytes(input: &mut Cursor<&[u8]>) -> Result<Self, ParseError>;

    fn byte_len(&self) -> usize;
}


#[cfg(test)]
mod tests {
    use super::*;
    use crypt::structs::{MainHeader, FileHeader, RepoHeader, FileVersion};
    use dto::*;

    #[test]
    fn enc_type() {
        let mut vec: Vec<u8> = Vec::new();
        EncryptionType::RingAESGCM.to_bytes(&mut vec);
        assert_eq!(1, vec.len());
        assert_eq!(2, vec[0]);

        assert_eq!(EncryptionType::None, EncryptionType::from_bytes(&mut Cursor::new(&[0])).unwrap());
        assert_eq!(EncryptionType::RingChachaPoly1305, EncryptionType::from_bytes(&mut Cursor::new(&[1])).unwrap());
        assert_eq!(EncryptionType::RingAESGCM, EncryptionType::from_bytes(&mut Cursor::new(&[2])).unwrap());

        assert_eq!(Some(ParseError::WrongValue(0, 42)), EncryptionType::from_bytes(&mut Cursor::new(&[42])).err());
    }

    #[test]
    fn enc_type_and_pw_type() {
        let mut vec: Vec<u8> = Vec::new();
        EncryptionType::RingAESGCM.to_bytes(&mut vec);
        PasswordHashType::None.to_bytes(&mut vec);
        assert_eq!(2, vec.len());
        assert_eq!(2, vec[0]);
        assert_eq!(0, vec[1]);
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
        assert_eq!(expected, result);

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

pub fn read_u8(input: &mut Cursor<&[u8]>) -> Result<u8, ParseError> {
    let pos = input.position();
    input.read_u8().map_err(|e| ParseError::IllegalPos(pos))
}

pub fn read_u16(input: &mut Cursor<&[u8]>) -> Result<u16, ParseError> {
    let pos = input.position();
    input.read_u16::<LittleEndian>().map_err(|e| ParseError::IllegalPos(pos))
}

pub fn read_u32(input: &mut Cursor<&[u8]>) -> Result<u32, ParseError> {
    let pos = input.position();
    input.read_u32::<LittleEndian>().map_err(|e| ParseError::IllegalPos(pos))
}

pub fn read_buff<'a>(input: &mut Cursor<&[u8]>, buff: &'a mut [u8]) -> Result<&'a [u8], ParseError> {
    let pos = input.position();
    input.read(buff).map_err(|e| ParseError::IllegalPos(pos))?;
    Ok(buff)
}

pub fn read_uuid(input: &mut Cursor<&[u8]>) -> Result<Uuid, ParseError> {
    let pos = input.position();
    let mut buff = [0u8; UUID_LENGTH];
    input.read(&mut buff).map_err(|e| ParseError::IllegalPos(pos))?;
    Uuid::from_bytes(&buff).map_err(|e| ParseError::NoValidUuid(pos))
}