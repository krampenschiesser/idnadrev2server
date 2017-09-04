use crypt::{FileHeader, EncryptedFile};


use dto::RepoId;
use super::id::FileId;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct FileDescriptor {
    pub repo: RepoId,
    pub id: FileId,
    pub version: u32,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct FileHeaderDescriptor {
    pub descriptor: FileDescriptor,
    pub header: String,
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
