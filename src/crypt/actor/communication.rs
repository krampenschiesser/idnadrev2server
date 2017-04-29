use super::dto::{FileDescriptor, FileHeaderDescriptor, RepositoryDescriptor};
use super::super::util::io::path_to_str;
use std::path::PathBuf;
use uuid::Uuid;

use std::fmt::{Display, Formatter};
use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CryptCmd {
    CreateNewFile { token: Uuid, header: String, content: Vec<u8>, repo: Uuid },
    UpdateHeader { token: Uuid, header: String, file: FileDescriptor },
    UpdateFile { token: Uuid, header: String, content: Vec<u8>, file: FileDescriptor },
    DeleteFile { token: Uuid, file: FileDescriptor },
    GetFileHeader { token: Uuid, file: FileDescriptor },
    GetFile { token: Uuid, file: FileDescriptor },

    OpenRepository { id: Uuid, pw: Vec<u8> },
    CloseRepository { token: Uuid, id: Uuid },
    ListRepositories,
    ListFiles { token: Uuid, id: Uuid },

    FileAdded(PathBuf),
    FileChanged(PathBuf),
    FileDeleted(PathBuf),

    Shutdown,
}

#[derive(Debug, PartialEq, Eq, Clone)]
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

    Shutdown,
}

impl Display for CryptCmd {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            CryptCmd::OpenRepository { ref id, .. } => write!(f, "Open repository {}", id),
            CryptCmd::CloseRepository { ref id, .. } => write!(f, "Close repository {}", id),
            CryptCmd::ListFiles { ref id, .. } => write!(f, "List files in {}", id),
            CryptCmd::ListRepositories => write!(f, "List repositories"),

            CryptCmd::CreateNewFile { ref header, ref repo, .. } => write!(f, "Create new file with header: {} in repo: {}", header, repo),
            CryptCmd::UpdateHeader { ref file, .. } => write!(f, "Update header of {} version={}", &file.id, &file.version),
            CryptCmd::UpdateFile { ref file, .. } => write!(f, "Updating complete file {} version={}", &file.id, &file.version),
            CryptCmd::DeleteFile { ref file, .. } => write!(f, "Deleting file {}", &file.id),
            CryptCmd::GetFileHeader { ref file, .. } => write!(f, "Get file header {}", &file.id),
            CryptCmd::GetFile { ref file, .. } => write!(f, "Get file {}", &file.id),

            CryptCmd::FileAdded(ref p) => write!(f, "File was added on file system: {}", path_to_str(p)),
            CryptCmd::FileChanged(ref p) => write!(f, "File was changed on file system: {}", path_to_str(p)),
            CryptCmd::FileDeleted(ref p) => write!(f, "File was deleted from file system: {}", path_to_str(p)),

            CryptCmd::Shutdown => write!(f, "Shutdown requested!"),
        }
    }
}

impl Display for CryptResponse {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            CryptResponse::FileCreated(ref desc) => write!(f, "Created file: {}", &desc.id),
            CryptResponse::FileChanged(ref desc) => write!(f, "Changed file: {}", &desc.id),
            CryptResponse::FileDeleted(ref desc) => write!(f, "Deleted file: {}", &desc.id),

            CryptResponse::File(ref fh) => write!(f, "File {}: {}", &fh.descriptor.id, &fh.header),
            CryptResponse::FileContent(ref fh, _) => write!(f, "File {}: {}", &fh.descriptor.id, &fh.header),
            CryptResponse::Files(ref files) => write!(f, "Listing of {} files", files.len()),

            CryptResponse::Repositories(ref repos) => write!(f, "Listing of {} repos", repos.len()),

            CryptResponse::RepositoryOpened { ref id, .. } => write!(f, "Opened repository {}", id),
            CryptResponse::RepositoryOpenFailed { ref id } => write!(f, "Failed to open repository {}", id),
            CryptResponse::RepositoryIsClosed { ref id } => write!(f, "Repository is closed {}", id),
            CryptResponse::NoSuchRepository { ref id } => write!(f, "Repositroy does not exist {}", id),

            CryptResponse::OptimisticLockError { ref file, ref file_version } => write!(f, "File was modified, new_version={} file_version={}, file={}", file_version, &file.version, &file.id),
            CryptResponse::NoSuchFile(ref file) => write!(f, "No file exists {}", file.id),
            CryptResponse::AccessDenied => write!(f, "Access denied (playing halflife?)"),
            CryptResponse::InvalidToken(ref t) => write!(f, "Invalid token {}", t),
            CryptResponse::UnrecognizedFile(ref reason) => write!(f, "Did not recoginze file: {}", reason),
            CryptResponse::Error(ref msg) => write!(f, "General error: {}", msg),

            CryptResponse::Shutdown => write!(f, "Shutdown successful"),
        }
    }
}