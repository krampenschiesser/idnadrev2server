// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std;
use std::io;
use std::string::FromUtf8Error;
use std::fmt::{Display, Formatter};
use std::fmt;
use std::error::Error;
use notify;
use super::structs::FileVersion;

#[derive(Debug, Eq, PartialEq)]
pub enum CryptError {
    FileAlreadyExists(String),
    FileDoesNotExist(String),
    IOError(String),
    ParseError(ParseError),
    WatcherCreationError(String),
    RingError(RingError),
    NoFilePath,
    NoFileContent,
    OptimisticLockError(u32),
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum RingError {
    KeyFailure,
    DecryptFailue,
    EncryptFailue,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum ParseError {
    WrongValue(u64, u8),
    IllegalPos(u64),
    InvalidUtf8(String),
    IoError(String),
    NoPrefix,
    NoValidUuid(u64),
    UnknownFileVersion(u8),
    InvalidFileVersion(FileVersion),
}

impl Display for CryptError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            CryptError::FileAlreadyExists(ref name) => write!(f, "File {} already exists.", name),
            CryptError::FileDoesNotExist(ref name) => write!(f, "File {} does not exist.", name),
            CryptError::IOError(ref description) => write!(f, "IO Error happened: {}", description),
            CryptError::ParseError(ref e) => write!(f, "Parsing error occured: {}", e),
            CryptError::WatcherCreationError(ref e) => write!(f, "Could not create file watcher! {}", e),
            CryptError::RingError(ref e) => write!(f, "Error happened during encryption/decryption: {}", e),
            CryptError::NoFilePath => write!(f, "No such file path"),
            CryptError::NoFileContent => write!(f, "No file content"),
            CryptError::OptimisticLockError(file_version) => write!(f, "File has newer version than self: {}", file_version),
        }
    }
}

impl From<io::Error> for CryptError {
    fn from(a: io::Error) -> Self {
        CryptError::IOError(format!("{:?}", a))
    }
}

impl From<FromUtf8Error> for CryptError {
    fn from(e: FromUtf8Error) -> Self {
        CryptError::ParseError(ParseError::InvalidUtf8(e.description().into()))
    }
}

impl From<ParseError> for CryptError {
    fn from(e: ParseError) -> Self {
        CryptError::ParseError(e)
    }
}

impl From<notify::Error> for CryptError {
    fn from(e: notify::Error) -> Self {
        CryptError::WatcherCreationError(e.description().into())
    }
}

impl From<RingError> for CryptError {
    fn from(e: RingError) -> Self {
        CryptError::RingError(e)
    }
}

impl Display for RingError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            RingError::KeyFailure => write!(f, "Something is wrong with the key, maybe length?"),
            RingError::DecryptFailue => write!(f, "Error happened during decryption"),
            RingError::EncryptFailue => write!(f, "Error happened during encryption"),
        }
    }
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            ParseError::WrongValue(ref pos, ref val) => write!(f, "Wrong value '{}' at pos {}", val, pos),
            ParseError::IllegalPos(ref pos) => write!(f, "Illegal position {}", pos),
            ParseError::InvalidUtf8(ref e) => write!(f, "No valid utf8: {}", e),
            ParseError::IoError(ref description) => write!(f, "IO Error happened: {}", description),
            ParseError::NoPrefix => write!(f, "No prefix present"),
            ParseError::NoValidUuid(ref pos) => write!(f, "No valid uuid at {}", pos),
            ParseError::UnknownFileVersion(ref version) => write!(f, "Unknown file version {}", version),
            ParseError::InvalidFileVersion(ref version) => write!(f, "Invalid file version {}", version),
        }
    }
}

impl From<std::string::FromUtf8Error> for ParseError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        ParseError::InvalidUtf8(e.description().into())
    }
}

impl From<std::io::Error> for ParseError {
    fn from(e: std::io::Error) -> Self {
        ParseError::IoError(e.description().into())
    }
}