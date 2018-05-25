use uuid::Uuid;

use super::repository::RepositoryId;
use ::pb::file::{EncryptionType,CompressionType};
use ::files::StoredFileName;
use ::crypt::{CipherTextVec,Nonce};

pub type FileId = Uuid;
pub type FileVersion = u32;

#[derive(Debug)]
pub struct RepositoryFile {
    pub id: FileId,
    pub version: FileVersion,
    pub repository_id: RepositoryId,
    pub file_name: StoredFileName,
    pub encryption_type: EncryptionType,
    pub compression_type: CompressionType,
}