// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use dto::*;
use super::super::util::io::path_to_str;
use std::path::PathBuf;
use uuid::Uuid;

use std::fmt::{Display, Formatter};
use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CryptCmd {
    CreateNewFile { token: AccessToken, header: String, content: Vec<u8>, repo: RepoId},
    UpdateHeader { token: AccessToken, header: String, file: FileDescriptor },
    UpdateFile { token: AccessToken, header: String, content: Vec<u8>, file: FileDescriptor },
    DeleteFile { token: AccessToken, file: FileDescriptor },
    GetFileHeader { token: AccessToken, file: FileDescriptor },
    GetFile { token: AccessToken, file: FileDescriptor },

    CreateRepository { name: String, pw: PlainPw, encryption: EncryptionType, kdf: PasswordHashType, folder_id: Option<u16> },
    OpenRepository { id: RepoId, user_name: String, pw: PlainPw },
    CloseRepository { token: AccessToken, id: RepoId},
    ListRepositories,
    ListFiles { token: AccessToken, id: RepoId},

    CheckToken { repo: RepoId, token: AccessToken },

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
    RepositoryChanged(RepoId),

    File(FileHeaderDescriptor),
    FileContent(FileHeaderDescriptor, Vec<u8>),
    Files(Vec<FileHeaderDescriptor>),

    Repositories(Vec<RepositoryDescriptor>),

    RepositoryOpened { token: AccessToken, id: RepoId},
    RepositoryCreated { token: AccessToken, id: RepoId },
    RepositoryOpenFailed { id: RepoId},
    RepositoryIsClosed { id: RepoId },
    NoSuchRepository { id: RepoId },
    RepositoryAlreadyExists { name: String },

    TokenValid,

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
            CryptCmd::CreateRepository { ref name, .. } => write!(f, "Creating repository {}", name),
            CryptCmd::OpenRepository { ref id, .. } => write!(f, "Open repository {}", id),
            CryptCmd::CloseRepository { ref id, .. } => write!(f, "Close repository {}", id),
            CryptCmd::ListFiles { ref id, .. } => write!(f, "List files in {}", id),
            CryptCmd::ListRepositories => write!(f, "List repositories"),
            CryptCmd::CheckToken { ref repo, .. } => write!(f, "Check token for repo {}", repo),

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
            CryptResponse::FileCreated(ref desc) => write!(f, "CryptResponse::FileCreated: Created file: {}", &desc.id),
            CryptResponse::FileChanged(ref desc) => write!(f, "CryptResponse::FileChanged: Changed file: {}", &desc.id),
            CryptResponse::FileDeleted(ref desc) => write!(f, "CryptResponse::FileDeleted: Deleted file: {}", &desc.id),

            CryptResponse::File(ref fh) => write!(f, "CryptResponse::File: File {}: {}", &fh.descriptor.id, &fh.header),
            CryptResponse::FileContent(ref fh, _) => write!(f, "CryptResponse::FileContent: File {}: {}", &fh.descriptor.id, &fh.header),
            CryptResponse::Files(ref files) => write!(f, "CryptResponse::Files: Listing of {} files", files.len()),

            CryptResponse::Repositories(ref repos) => write!(f, "CryptResponse::Repositories: Listing of {} repos", repos.len()),

            CryptResponse::RepositoryCreated { ref id, .. } => write!(f, "CryptResponse::RepositoryCreated: created repository {}", id),
            CryptResponse::RepositoryOpened { ref id, .. } => write!(f, "CryptResponse::RepositoryOpened: Opened repository {}", id),
            CryptResponse::RepositoryOpenFailed { ref id } => write!(f, "CryptResponse::RepositoryOpenFailed: Failed to open repository {}", id),
            CryptResponse::RepositoryIsClosed { ref id } => write!(f, "CryptResponse::RepositoryIsClosed: Repository is closed {}", id),
            CryptResponse::RepositoryAlreadyExists { ref name } => write!(f, "CryptResponse::RepositoryAlreadyExists: A repository with name {} already exists.", name),
            CryptResponse::NoSuchRepository { ref id } => write!(f, "CryptResponse::NoSuchRepository: Repositroy does not exist {}", id),
            CryptResponse::RepositoryChanged(ref id) => write!(f, "CryptResponse::RepositoryChanged: Repository {} changed.", id),

            CryptResponse::TokenValid => write!(f, "CryptResponse::TokenValid"),

            CryptResponse::OptimisticLockError { ref file, ref file_version } => write!(f, "CryptResponse::OptimisticLockError: File was modified, new_version={} file_version={}, file={}", file_version, &file.version, &file.id),
            CryptResponse::NoSuchFile(ref file) => write!(f, "CryptResponse::NoSuchFile: No file exists {}", file.id),
            CryptResponse::AccessDenied => write!(f, "CryptResponse::AccessDenied: Access denied (playing halflife?)"),
            CryptResponse::InvalidToken(ref t) => write!(f, "CryptResponse::InvalidToken: Invalid token {}", t),
            CryptResponse::UnrecognizedFile(ref reason) => write!(f, "CryptResponse::UnrecognizedFile: Did not recoginze file: {}", reason),
            CryptResponse::Error(ref msg) => write!(f, "CryptResponse::Error: General error: {}", msg),

            CryptResponse::Shutdown => write!(f, "CryptResponse::Shutdown: Shutdown successful"),
        }
    }
}