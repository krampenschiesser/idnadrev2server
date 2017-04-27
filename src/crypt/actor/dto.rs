use super::super::structs::file::{FileHeader, EncryptedFile};
use super::super::structs::repository::{RepoHeader, Repository};

use std::time::Instant;
use uuid::Uuid;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AccessToken {
    #[cfg(test)]
    pub last_access: Instant,
    #[cfg(not(test))]
    last_access: Instant,

    id: Uuid,
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

#[derive(Debug, PartialEq, Eq)]
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
        let id = Uuid::new_v4();
        AccessToken { id: id, last_access: Instant::now() }
    }

    pub fn touch(&mut self) {
        self.last_access = Instant::now();
    }

    pub fn get_id(&self) -> Uuid {
        self.id.clone()
    }

    pub fn get_elapsed_minutes(&self) -> u64 {
        let secs = self.last_access.elapsed().as_secs();
        secs * 60
    }
}