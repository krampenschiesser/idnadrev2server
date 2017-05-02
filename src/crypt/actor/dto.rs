// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use super::super::structs::file::{FileHeader, EncryptedFile};
use super::super::structs::repository::{RepoHeader, Repository};

use std::time::Instant;
use uuid::Uuid;


use std::fmt::{Display, Formatter};
use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize)]
pub struct AccessToken {
    pub id: Uuid,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum EncTypeDto {
    AES,
    ChaCha
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum PwKdfDto {
    SCrypt { iterations: u8, memory_costs: u32, parallelism: u32 },
    //Argon,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct RepositoryDto {
    pub id: Uuid,
    pub token: AccessToken,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FileDescriptor {
    pub repo: Uuid,
    pub id: Uuid,
    pub version: u32,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FileHeaderDescriptor {
    pub descriptor: FileDescriptor,
    pub header: String,
}

#[derive(Debug, PartialEq, Eq, Clone,Serialize)]
pub struct RepositoryDescriptor {
    pub id: Uuid,
    pub name: String,
}

impl FileDescriptor {
    pub fn new(header: &FileHeader) -> Self {
        FileDescriptor { repo: header.get_repository_id(), id: header.get_id(), version: header.get_version() }
    }
}

impl FileHeaderDescriptor {
    pub fn new(enc_file: &EncryptedFile) -> Self {
        let ref h = enc_file.get_encryption_header();
        let descriptor = FileDescriptor { repo: h.get_repository_id(), id: h.get_id(), version: h.get_version() };
        FileHeaderDescriptor { header: enc_file.get_header().clone(), descriptor: descriptor }
    }
}

impl RepositoryDescriptor {
    pub fn new(repo: &Repository) -> Self {
        RepositoryDescriptor { id: repo.get_id(), name: repo.get_name().clone() }
    }
}

impl AccessToken {
    pub fn new() -> Self {
        AccessToken { id: Uuid::new_v4() }
    }
}

impl Display for AccessToken {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Creating repository {}", self.id.simple())
    }
}