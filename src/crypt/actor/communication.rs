use super::dto::{FileDescriptor,FileHeaderDescriptor,RepositoryDescriptor};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, PartialEq, Eq)]
pub enum CryptCmd {
    CreateNewFile { token: Uuid, header: String, content: Vec<u8>, repo: Uuid },
    UpdateHeader { token: Uuid, header: String, file: FileDescriptor },
    UpdateFile { token: Uuid, header: String, content: Vec<u8>, file: FileDescriptor },
    DeleteFile { token: Uuid, file: FileDescriptor },

    OpenRepository { id: Uuid, pw: Vec<u8> },
    CloseRepository { token: Uuid, id: Uuid },
    ListRepositories,
    ListFiles { token: Uuid, id: Uuid },

    FileAdded(PathBuf),
    FileChanged(PathBuf),
    FileDeleted(PathBuf),
}

#[derive(Debug, PartialEq, Eq)]
pub enum CryptResponse {
    FileCreated(FileDescriptor),
    FileChanged(FileDescriptor),
    FileDeleted(FileDescriptor),

    File(FileHeaderDescriptor),
    FileContent(FileHeaderDescriptor, Vec<u8>),
    Files(Vec<FileHeaderDescriptor>),

    Repositories(Vec<RepositoryDescriptor>),

    RepositoryOpened { token: Uuid, id: Uuid },
    RepositoryOpenFailed { id: Uuid },
    RepositoryIsClosed { id: Uuid },
    NoSuchRepository { id: Uuid },

    OptimisticLockError { file: FileDescriptor, file_version: u32 },
    NoSuchFile(FileDescriptor),
    AccessDenied,
    InvalidToken(String),
    Error(String),

    UnrecognizedFile(String),
}