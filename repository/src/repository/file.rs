use uuid::Uuid;

use super::repository::RepositoryId;
use ::pb::file::{EncryptionType,CompressionType};
use ::files::StoredFileName;
use ::crypt::{CipherTextVec,Nonce};

pub type FileId = Uuid;
pub type FileVersion = u32;

pub struct RepositoryFile {
    id: FileId,
    version: FileVersion,
    repository_id: RepositoryId,
    file_name: StoredFileName,
    encryption_type: EncryptionType,
    compression_type: CompressionType,
}