//! Automatically generated rust module for 'file.proto' file

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]
#![allow(unknown_lints)]
#![allow(clippy)]
#![cfg_attr(rustfmt, rustfmt_skip)]


use std::io::Write;
use std::borrow::Cow;
use quick_protobuf::{MessageRead, MessageWrite, BytesReader, Writer, Result};
use quick_protobuf::sizeofs::*;
use super::*;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum FileType {
    RepositoryV1 = 1,
    FileV1 = 2,
}

impl Default for FileType {
    fn default() -> Self {
        FileType::RepositoryV1
    }
}

impl From<i32> for FileType {
    fn from(i: i32) -> Self {
        match i {
            1 => FileType::RepositoryV1,
            2 => FileType::FileV1,
            _ => Self::default(),
        }
    }
}

impl<'a> From<&'a str> for FileType {
    fn from(s: &'a str) -> Self {
        match s {
            "RepositoryV1" => FileType::RepositoryV1,
            "FileV1" => FileType::FileV1,
            _ => Self::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum EncryptionType {
    ChachaPoly1305 = 1,
}

impl Default for EncryptionType {
    fn default() -> Self {
        EncryptionType::ChachaPoly1305
    }
}

impl From<i32> for EncryptionType {
    fn from(i: i32) -> Self {
        match i {
            1 => EncryptionType::ChachaPoly1305,
            _ => Self::default(),
        }
    }
}

impl<'a> From<&'a str> for EncryptionType {
    fn from(s: &'a str) -> Self {
        match s {
            "ChachaPoly1305" => EncryptionType::ChachaPoly1305,
            _ => Self::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PasswordHashType {
    SCrypt = 1,
}

impl Default for PasswordHashType {
    fn default() -> Self {
        PasswordHashType::SCrypt
    }
}

impl From<i32> for PasswordHashType {
    fn from(i: i32) -> Self {
        match i {
            1 => PasswordHashType::SCrypt,
            _ => Self::default(),
        }
    }
}

impl<'a> From<&'a str> for PasswordHashType {
    fn from(s: &'a str) -> Self {
        match s {
            "SCrypt" => PasswordHashType::SCrypt,
            _ => Self::default(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CompressionType {
    DeflateZip = 1,
}

impl Default for CompressionType {
    fn default() -> Self {
        CompressionType::DeflateZip
    }
}

impl From<i32> for CompressionType {
    fn from(i: i32) -> Self {
        match i {
            1 => CompressionType::DeflateZip,
            _ => Self::default(),
        }
    }
}

impl<'a> From<&'a str> for CompressionType {
    fn from(s: &'a str) -> Self {
        match s {
            "DeflateZip" => CompressionType::DeflateZip,
            _ => Self::default(),
        }
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct StoredFileWrapper<'a> {
    pub type_pb: FileType,
    pub content: Cow<'a, [u8]>,
}

impl<'a> MessageRead<'a> for StoredFileWrapper<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.type_pb = r.read_enum(bytes)?,
                Ok(18) => msg.content = r.read_bytes(bytes).map(Cow::Borrowed)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for StoredFileWrapper<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_varint(*(&self.type_pb) as u64)
        + 1 + sizeof_len((&self.content).len())
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(8, |w| w.write_enum(*&self.type_pb as i32))?;
        w.write_with_tag(18, |w| w.write_bytes(&**&self.content))?;
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct StoredRepositoryV1<'a> {
    pub id: Cow<'a, [u8]>,
    pub version: u32,
    pub repository_id: Cow<'a, [u8]>,
    pub enc_type: EncryptionType,
    pub hash_type: PasswordHashType,
    pub salt: Cow<'a, [u8]>,
    pub double_hashed_pw: Cow<'a, [u8]>,
    pub nonce: Cow<'a, [u8]>,
    pub encrypted_file_pw: Cow<'a, [u8]>,
    pub name: Cow<'a, str>,
}

impl<'a> MessageRead<'a> for StoredRepositoryV1<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.id = r.read_bytes(bytes).map(Cow::Borrowed)?,
                Ok(16) => msg.version = r.read_uint32(bytes)?,
                Ok(26) => msg.repository_id = r.read_bytes(bytes).map(Cow::Borrowed)?,
                Ok(32) => msg.enc_type = r.read_enum(bytes)?,
                Ok(40) => msg.hash_type = r.read_enum(bytes)?,
                Ok(50) => msg.salt = r.read_bytes(bytes).map(Cow::Borrowed)?,
                Ok(58) => msg.double_hashed_pw = r.read_bytes(bytes).map(Cow::Borrowed)?,
                Ok(66) => msg.nonce = r.read_bytes(bytes).map(Cow::Borrowed)?,
                Ok(74) => msg.encrypted_file_pw = r.read_bytes(bytes).map(Cow::Borrowed)?,
                Ok(82) => msg.name = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for StoredRepositoryV1<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_len((&self.id).len())
        + 1 + sizeof_varint(*(&self.version) as u64)
        + 1 + sizeof_len((&self.repository_id).len())
        + 1 + sizeof_varint(*(&self.enc_type) as u64)
        + 1 + sizeof_varint(*(&self.hash_type) as u64)
        + 1 + sizeof_len((&self.salt).len())
        + 1 + sizeof_len((&self.double_hashed_pw).len())
        + 1 + sizeof_len((&self.nonce).len())
        + 1 + sizeof_len((&self.encrypted_file_pw).len())
        + 1 + sizeof_len((&self.name).len())
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(10, |w| w.write_bytes(&**&self.id))?;
        w.write_with_tag(16, |w| w.write_uint32(*&self.version))?;
        w.write_with_tag(26, |w| w.write_bytes(&**&self.repository_id))?;
        w.write_with_tag(32, |w| w.write_enum(*&self.enc_type as i32))?;
        w.write_with_tag(40, |w| w.write_enum(*&self.hash_type as i32))?;
        w.write_with_tag(50, |w| w.write_bytes(&**&self.salt))?;
        w.write_with_tag(58, |w| w.write_bytes(&**&self.double_hashed_pw))?;
        w.write_with_tag(66, |w| w.write_bytes(&**&self.nonce))?;
        w.write_with_tag(74, |w| w.write_bytes(&**&self.encrypted_file_pw))?;
        w.write_with_tag(82, |w| w.write_string(&**&self.name))?;
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct StoredFileV1<'a> {
    pub id: Cow<'a, [u8]>,
    pub version: u32,
    pub repository_id: Cow<'a, [u8]>,
    pub encryption_type: EncryptionType,
    pub compression_type: CompressionType,
    pub nonce_header: Cow<'a, [u8]>,
    pub nonce_content: Cow<'a, [u8]>,
    pub encrypted_header: Cow<'a, [u8]>,
    pub encrypted_content: Cow<'a, [u8]>,
}

impl<'a> MessageRead<'a> for StoredFileV1<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.id = r.read_bytes(bytes).map(Cow::Borrowed)?,
                Ok(16) => msg.version = r.read_uint32(bytes)?,
                Ok(26) => msg.repository_id = r.read_bytes(bytes).map(Cow::Borrowed)?,
                Ok(32) => msg.encryption_type = r.read_enum(bytes)?,
                Ok(40) => msg.compression_type = r.read_enum(bytes)?,
                Ok(50) => msg.nonce_header = r.read_bytes(bytes).map(Cow::Borrowed)?,
                Ok(58) => msg.nonce_content = r.read_bytes(bytes).map(Cow::Borrowed)?,
                Ok(66) => msg.encrypted_header = r.read_bytes(bytes).map(Cow::Borrowed)?,
                Ok(74) => msg.encrypted_content = r.read_bytes(bytes).map(Cow::Borrowed)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for StoredFileV1<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_len((&self.id).len())
        + 1 + sizeof_varint(*(&self.version) as u64)
        + 1 + sizeof_len((&self.repository_id).len())
        + 1 + sizeof_varint(*(&self.encryption_type) as u64)
        + 1 + sizeof_varint(*(&self.compression_type) as u64)
        + 1 + sizeof_len((&self.nonce_header).len())
        + 1 + sizeof_len((&self.nonce_content).len())
        + 1 + sizeof_len((&self.encrypted_header).len())
        + 1 + sizeof_len((&self.encrypted_content).len())
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(10, |w| w.write_bytes(&**&self.id))?;
        w.write_with_tag(16, |w| w.write_uint32(*&self.version))?;
        w.write_with_tag(26, |w| w.write_bytes(&**&self.repository_id))?;
        w.write_with_tag(32, |w| w.write_enum(*&self.encryption_type as i32))?;
        w.write_with_tag(40, |w| w.write_enum(*&self.compression_type as i32))?;
        w.write_with_tag(50, |w| w.write_bytes(&**&self.nonce_header))?;
        w.write_with_tag(58, |w| w.write_bytes(&**&self.nonce_content))?;
        w.write_with_tag(66, |w| w.write_bytes(&**&self.encrypted_header))?;
        w.write_with_tag(74, |w| w.write_bytes(&**&self.encrypted_content))?;
        Ok(())
    }
}

