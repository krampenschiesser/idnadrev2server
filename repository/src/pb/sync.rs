//! Automatically generated rust module for 'sync.proto' file

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

#[derive(Debug, Default, PartialEq, Clone)]
pub struct SingleFileSync<'a> {
    pub id: Cow<'a, [u8]>,
    pub version: u32,
    pub hash: Cow<'a, [u8]>,
}

impl<'a> MessageRead<'a> for SingleFileSync<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.id = r.read_bytes(bytes).map(Cow::Borrowed)?,
                Ok(16) => msg.version = r.read_uint32(bytes)?,
                Ok(26) => msg.hash = r.read_bytes(bytes).map(Cow::Borrowed)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for SingleFileSync<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_len((&self.id).len())
        + 1 + sizeof_varint(*(&self.version) as u64)
        + 1 + sizeof_len((&self.hash).len())
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(10, |w| w.write_bytes(&**&self.id))?;
        w.write_with_tag(16, |w| w.write_uint32(*&self.version))?;
        w.write_with_tag(26, |w| w.write_bytes(&**&self.hash))?;
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct SynchronizationBucket<'a> {
    pub file_syncs: Vec<SingleFileSync<'a>>,
}

impl<'a> MessageRead<'a> for SynchronizationBucket<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.file_syncs.push(r.read_message::<SingleFileSync>(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for SynchronizationBucket<'a> {
    fn get_size(&self) -> usize {
        0
        + self.file_syncs.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        for s in &self.file_syncs { w.write_with_tag(10, |w| w.write_message(s))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct HashBucket<'a> {
    pub hash: Cow<'a, [u8]>,
    pub divisions: Vec<Subdivision>,
}

impl<'a> MessageRead<'a> for HashBucket<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.hash = r.read_bytes(bytes).map(Cow::Borrowed)?,
                Ok(18) => msg.divisions.push(r.read_message::<Subdivision>(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for HashBucket<'a> {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_len((&self.hash).len())
        + self.divisions.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(10, |w| w.write_bytes(&**&self.hash))?;
        for s in &self.divisions { w.write_with_tag(18, |w| w.write_message(s))?; }
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Subdivision {
    pub division: u32,
    pub modulo: u32,
    pub remainder: u32,
}

impl<'a> MessageRead<'a> for Subdivision {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.division = r.read_uint32(bytes)?,
                Ok(16) => msg.modulo = r.read_uint32(bytes)?,
                Ok(24) => msg.remainder = r.read_uint32(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl MessageWrite for Subdivision {
    fn get_size(&self) -> usize {
        0
        + 1 + sizeof_varint(*(&self.division) as u64)
        + 1 + sizeof_varint(*(&self.modulo) as u64)
        + 1 + sizeof_varint(*(&self.remainder) as u64)
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        w.write_with_tag(8, |w| w.write_uint32(*&self.division))?;
        w.write_with_tag(16, |w| w.write_uint32(*&self.modulo))?;
        w.write_with_tag(24, |w| w.write_uint32(*&self.remainder))?;
        Ok(())
    }
}

#[derive(Debug, Default, PartialEq, Clone)]
pub struct Synchronization<'a> {
    pub buckets: Vec<HashBucket<'a>>,
}

impl<'a> MessageRead<'a> for Synchronization<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.buckets.push(r.read_message::<HashBucket>(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for Synchronization<'a> {
    fn get_size(&self) -> usize {
        0
        + self.buckets.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
    }

    fn write_message<W: Write>(&self, w: &mut Writer<W>) -> Result<()> {
        for s in &self.buckets { w.write_with_tag(10, |w| w.write_message(s))?; }
        Ok(())
    }
}

