// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

pub mod actor;
mod structs;
mod util;
mod error;

use self::actor::state::State;
pub use self::error::CryptError;
pub use self::structs::file::{FileHeader, EncryptedFile};
pub use self::structs::repository::{Repository, RepoHeader};
use self::actor::communication::*;
use self::actor::handle;
use actor::{ActorControl, Actor, SenderWrapper, SendSync};
use dto::*;

use std::path::PathBuf;
use std::thread;
use std::sync::mpsc::Sender;
use dto::PlainPw;

pub struct CryptoActor {
    actor_control: ActorControl<CryptCmd, CryptResponse>,
}

pub struct CryptoSender {
    sender: SenderWrapper<CryptCmd, CryptResponse>,
}

pub trait CryptoIfc {
    fn list_repositories(&self) -> Option<Vec<RepositoryDescriptor>>;

    fn open_repository(&self, id: &RepoId, user_name: String, pw: PlainPw ) -> Option<AccessToken>;

    fn create_repository(&self, name: &str, pw: PlainPw , enc_type: EncryptionType) -> Option<RepositoryDto>;

    fn close_repository(&self, id: &RepoId, token: &AccessToken) -> Option<RepoId>;

    fn list_repository_files(&self, id: &RepoId, token: &AccessToken) -> Option<Vec<FileHeaderDescriptor>>;

    fn create_new_file(&self, repo_id: &RepoId, token: &AccessToken, header: String, content: Vec<u8>) -> Option<FileDescriptor>;

    fn get_file_header(&self, repo_id: &RepoId, token: &AccessToken, file_id: &FileId) -> Option<FileHeaderDescriptor>;

    fn get_file(&self, repo_id: &RepoId, token: &AccessToken, file_id: &FileId) -> Option<(FileHeaderDescriptor, Vec<u8>)>;

    fn update_header(&self, repo_id: &RepoId, token: &AccessToken, file_id: &FileId, file_version: u32, header: &str) -> Option<FileHeaderDescriptor>;

    fn update_file(&self, repo_id: &RepoId, token: &AccessToken, file_id: &FileId, file_version: u32, header: &str, content: Vec<u8>) -> Option<FileHeaderDescriptor>;

    fn delete_file(&self, repo_id: &RepoId, token: &AccessToken, file_id: &FileId, file_version: u32) -> Option<FileDescriptor>;

    fn check_token(&self, repo_id: &RepoId, token: &AccessToken) -> bool;
}

impl CryptoActor {
    pub fn new(folders: Vec<PathBuf>) -> Result<Self, CryptError> {
        let mut state = State::new(folders)?;


        let (mut actor, control) = Actor::start(state, handle, CryptCmd::Shutdown);
        let handle = thread::spawn(move || actor.run());

        Ok(CryptoActor { actor_control: control })
    }

    pub fn create_sender(&self) -> CryptoSender {
        CryptoSender { sender: self.actor_control.clone_sender() }
    }
}

impl CryptoIfc for CryptoSender {
    fn list_repositories(&self) -> Option<Vec<RepositoryDescriptor>> {
        list_repositories(&self.sender)
    }

    fn open_repository(&self, id: &RepoId, user_name: String, pw: PlainPw) -> Option<AccessToken> {
        open_repository(&self.sender, id, user_name, pw)
    }

    fn create_repository(&self, name: &str, pw: PlainPw, enc_type: EncryptionType) -> Option<RepositoryDto> {
        create_repository(&self.sender, name, pw, enc_type)
    }

    fn close_repository(&self, id: &RepoId, token: &AccessToken) -> Option<RepoId> {
        close_repository(&self.sender, id, token)
    }

    fn list_repository_files(&self, id: &RepoId, token: &AccessToken) -> Option<Vec<FileHeaderDescriptor>> {
        list_repository_files(&self.sender, id, token)
    }

    fn create_new_file(&self, repo_id: &RepoId, token: &AccessToken, header: String, content: Vec<u8>) -> Option<FileDescriptor> {
        create_new_file(&self.sender, repo_id, token, header, content)
    }

    fn get_file_header(&self, repo_id: &RepoId, token: &AccessToken, file_id: &FileId) -> Option<FileHeaderDescriptor> {
        get_file_header(&self.sender, repo_id, token, file_id)
    }

    fn get_file(&self, repo_id: &RepoId, token: &AccessToken, file_id: &FileId) -> Option<(FileHeaderDescriptor, Vec<u8>)> {
        get_file(&self.sender, repo_id, token, file_id)
    }

    fn update_header(&self, repo_id: &RepoId, token: &AccessToken, file_id: &FileId, file_version: u32, header: &str) -> Option<FileHeaderDescriptor> {
        update_header(&self.sender, repo_id, token, file_id, file_version, header)
    }

    fn update_file(&self, repo_id: &RepoId, token: &AccessToken, file_id: &FileId, file_version: u32, header: &str, content: Vec<u8>) -> Option<FileHeaderDescriptor> {
        update_file(&self.sender, repo_id, token, file_id, file_version, header, content)
    }

    fn delete_file(&self, repo_id: &RepoId, token: &AccessToken, file_id: &FileId, file_version: u32) -> Option<FileDescriptor> {
        delete_file(&self.sender, repo_id, token, file_id, file_version)
    }

    fn check_token(&self, repo_id: &RepoId, token: &AccessToken) -> bool {
        check_token(&self.sender, repo_id, token)
    }
}

impl CryptoIfc for CryptoActor {
    fn list_repositories(&self) -> Option<Vec<RepositoryDescriptor>> {
        list_repositories(&self.actor_control)
    }

    fn open_repository(&self, id: &RepoId, user_name: String, pw: PlainPw) -> Option<AccessToken> {
        open_repository(&self.actor_control, id, user_name, pw)
    }

    fn create_repository(&self, name: &str, pw: PlainPw, enc_type: EncryptionType) -> Option<RepositoryDto> {
        create_repository(&self.actor_control, name, pw, enc_type)
    }

    fn close_repository(&self, id: &RepoId, token: &AccessToken) -> Option<RepoId> {
        close_repository(&self.actor_control, id, token)
    }

    fn list_repository_files(&self, id: &RepoId, token: &AccessToken) -> Option<Vec<FileHeaderDescriptor>> {
        list_repository_files(&self.actor_control, id, token)
    }

    fn create_new_file(&self, repo_id: &RepoId, token: &AccessToken, header: String, content: Vec<u8>) -> Option<FileDescriptor> {
        create_new_file(&self.actor_control, repo_id, token, header, content)
    }

    fn get_file_header(&self, repo_id: &RepoId, token: &AccessToken, file_id: &FileId) -> Option<FileHeaderDescriptor> {
        get_file_header(&self.actor_control, repo_id, token, file_id)
    }

    fn get_file(&self, repo_id: &RepoId, token: &AccessToken, file_id: &FileId) -> Option<(FileHeaderDescriptor, Vec<u8>)> {
        get_file(&self.actor_control, repo_id, token, file_id)
    }

    fn update_header(&self, repo_id: &RepoId, token: &AccessToken, file_id: &FileId, file_version: u32, header: &str) -> Option<FileHeaderDescriptor> {
        update_header(&self.actor_control, repo_id, token, file_id, file_version, header)
    }

    fn update_file(&self, repo_id: &RepoId, token: &AccessToken, file_id: &FileId, file_version: u32, header: &str, content: Vec<u8>) -> Option<FileHeaderDescriptor> {
        update_file(&self.actor_control, repo_id, token, file_id, file_version, header, content)
    }

    fn delete_file(&self, repo_id: &RepoId, token: &AccessToken, file_id: &FileId, file_version: u32) -> Option<FileDescriptor> {
        delete_file(&self.actor_control, repo_id, token, file_id, file_version)
    }

    fn check_token(&self, repo_id: &RepoId, token: &AccessToken) -> bool {
        check_token(&self.actor_control, repo_id, token)
    }
}


fn send_unwrap<T: SendSync<CryptCmd, CryptResponse>>(send: &T, cmd: CryptCmd) -> Option<CryptResponse> {
    let response = send.send_sync(cmd.clone());
    match response {
        Err(msg) => {
            error!("Got error from cryptactor: {}", msg);
            None
        }
        Ok(o) => {
            Some(o)
        }
    }
}

fn list_repositories<T: SendSync<CryptCmd, CryptResponse>>(send: &T) -> Option<Vec<RepositoryDescriptor>> {
    let cmd = CryptCmd::ListRepositories;

    if let Some(response) = send_unwrap(send, cmd) {
        match response {
            CryptResponse::Repositories(vec) => Some(vec),
            o => {
                error!("Could not list repositories: {}", o);
                None
            }
        }
    } else {
        None
    }
}

fn open_repository<T: SendSync<CryptCmd, CryptResponse>>(send: &T, id: &RepoId, user_name: String, pw: PlainPw) -> Option<AccessToken> {
    let cmd = CryptCmd::OpenRepository { id: id.clone(), user_name: user_name, pw: pw };

    if let Some(response) = send_unwrap(send, cmd) {
        match response {
            CryptResponse::RepositoryOpened { token, id } => Some(token),
            o => {
                error!("Got wrong response while opening repository {}: {}", id, o);
                None
            }
        }
    } else {
        None
    }
}

fn create_repository<T: SendSync<CryptCmd, CryptResponse>>(send: &T, name: &str, pw: PlainPw, enc_type: EncryptionType) -> Option<RepositoryDto> {
    #[cfg(debug_assertions)]
    let scrypt = PasswordHashType::SCrypt { iterations: 4, memory_costs: 4, parallelism: 1 };
    #[cfg(not(debug_assertions))]
    let scrypt = PasswordHashType::SCrypt { iterations: 16, memory_costs: 8, parallelism: 1 };

    let cmd = CryptCmd::CreateRepository { name: name.to_string(), pw: pw, folder_id: None, encryption: EncryptionType::RingChachaPoly1305, kdf: scrypt };

    if let Some(response) = send_unwrap(send, cmd) {
        match response {
            CryptResponse::RepositoryCreated { token, id } => Some(RepositoryDto { token: token, id: id }),
            o => {
                error!("Got wrong response while creating repository {}: {}", name, o);
                None
            }
        }
    } else {
        None
    }
}

fn close_repository<T: SendSync<CryptCmd, CryptResponse>>(send: &T, id: &RepoId, token: &AccessToken) -> Option<RepoId> {
    let cmd = CryptCmd::CloseRepository { id: id.clone(), token: token.clone() };

    if let Some(response) = send_unwrap(send, cmd) {
        match response {
            CryptResponse::RepositoryIsClosed { id } => Some(id),
            o => {
                error!("Got wrong response while closing repository {}: {}", id, o);
                None
            }
        }
    } else {
        None
    }
}

fn list_repository_files<T: SendSync<CryptCmd, CryptResponse>>(send: &T, id: &RepoId, token: &AccessToken) -> Option<Vec<FileHeaderDescriptor>> {
    let cmd = CryptCmd::ListFiles { id: id.clone(), token: token.clone() };

    if let Some(response) = send_unwrap(send, cmd) {
        match response {
            CryptResponse::Files(vec) => Some(vec),
            o => {
                error!("Got wrong response while listing repository files {}: {}", id, o);
                None
            }
        }
    } else {
        None
    }
}

fn create_new_file<T: SendSync<CryptCmd, CryptResponse>>(send: &T, repo_id: &RepoId, token: &AccessToken, header: String, content: Vec<u8>) -> Option<FileDescriptor> {
    let cmd = CryptCmd::CreateNewFile { token: token.clone(), header: header, repo: repo_id.clone(), content: content };

    if let Some(response) = send_unwrap(send, cmd) {
        match response {
            CryptResponse::FileCreated(descriptor) => Some(descriptor),
            o => {
                error!("Got wrong response while trying to create a new file in {}: {}", repo_id, o);
                None
            }
        }
    } else {
        None
    }
}

fn get_file_header<T: SendSync<CryptCmd, CryptResponse>>(send: &T, repo_id: &RepoId, token: &AccessToken, file_id: &FileId) -> Option<FileHeaderDescriptor> {
    let cmd = CryptCmd::GetFileHeader { token: token.clone(), file: FileDescriptor { repo: repo_id.clone(), id: file_id.clone(), version: 0 } };

    if let Some(response) = send_unwrap(send, cmd) {
        match response {
            CryptResponse::File(descriptor) => Some(descriptor),
            o => {
                error!("Got wrong response while trying to get file header {}: {}", file_id, o);
                None
            }
        }
    } else {
        None
    }
}

fn get_file<T: SendSync<CryptCmd, CryptResponse>>(send: &T, repo_id: &RepoId, token: &AccessToken, file_id: &FileId) -> Option<(FileHeaderDescriptor, Vec<u8>)> {
    let cmd = CryptCmd::GetFile { token: token.clone(), file: FileDescriptor { repo: repo_id.clone(), id: file_id.clone(), version: 0 } };

    if let Some(response) = send_unwrap(send, cmd) {
        match response {
            CryptResponse::FileContent(descriptor, content) => Some((descriptor, content)),
            o => {
                error!("Got wrong response while trying to get file {}: {}", file_id, o);
                None
            }
        }
    } else {
        None
    }
}

fn update_header<T: SendSync<CryptCmd, CryptResponse>>(send: &T, repo_id: &RepoId, token: &AccessToken, file_id: &FileId, file_version: u32, header: &str) -> Option<FileHeaderDescriptor> {
    let desc = FileDescriptor { repo: repo_id.clone(), id: file_id.clone(), version: file_version };
    let cmd = CryptCmd::UpdateHeader { token: token.clone(), file: desc, header: header.to_string() };

    if let Some(response) = send_unwrap(send, cmd) {
        match response {
            CryptResponse::File(descriptor) => Some(descriptor),
            o => {
                error!("Got wrong response while trying to update file header {}: {}", file_id, o);
                None
            }
        }
    } else {
        None
    }
}

fn update_file<T: SendSync<CryptCmd, CryptResponse>>(send: &T, repo_id: &RepoId, token: &AccessToken, file_id: &FileId, file_version: u32, header: &str, content: Vec<u8>) -> Option<FileHeaderDescriptor> {
    let desc = FileDescriptor { repo: repo_id.clone(), id: file_id.clone(), version: file_version };
    let cmd = CryptCmd::UpdateFile { token: token.clone(), file: desc, header: header.to_string(), content: content };

    if let Some(response) = send_unwrap(send, cmd) {
        match response {
            CryptResponse::File(descriptor) => Some(descriptor),
            o => {
                error!("Got wrong response while trying to update file {}: {}", file_id, o);
                None
            }
        }
    } else {
        None
    }
}

fn delete_file<T: SendSync<CryptCmd, CryptResponse>>(send: &T, repo_id: &RepoId, token: &AccessToken, file_id: &FileId, file_version: u32) -> Option<FileDescriptor> {
    let desc = FileDescriptor { repo: repo_id.clone(), id: file_id.clone(), version: file_version };
    let cmd = CryptCmd::DeleteFile { token: token.clone(), file: desc };

    if let Some(response) = send_unwrap(send, cmd) {
        match response {
            CryptResponse::FileDeleted(descriptor) => Some(descriptor),
            o => {
                error!("Got wrong response while trying to delete file {}: {}", file_id, o);
                None
            }
        }
    } else {
        None
    }
}

fn check_token<T: SendSync<CryptCmd, CryptResponse>>(send: &T, repo_id: &RepoId, token: &AccessToken) -> bool {
    let cmd = CryptCmd::CheckToken { repo: repo_id.clone(), token: token.clone() };

    if let Some(response) = send_unwrap(send, cmd) {
        match response {
            CryptResponse::TokenValid => true,
            _ => false
        }
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use self::super::actor::function::tests::create_temp_repo;
    use tempdir::TempDir;
    use std::time::Instant;

    #[test]
    fn test_full_process() {
        let (temp, repo, pw) = create_temp_repo();
        let repo_id = repo.get_id();

        let actor = CryptoActor::new(vec![temp.path().to_path_buf()]).unwrap();

        let repos = actor.list_repositories().unwrap();
        assert_eq!(1, repos.len());

        let token = actor.open_repository(&repo_id, "name".to_string(), "password".as_bytes().to_vec()).unwrap();
        let files = actor.list_repository_files(&repo_id, &token).unwrap();
        assert_eq!(1, files.len());

        let new_file = actor.create_new_file(&repo_id, &token, "Hallo Header".to_string(), vec![4, 2]).unwrap();

        let files = actor.list_repository_files(&repo_id, &token).unwrap();
        assert_eq!(2, files.len());

        let header = actor.get_file_header(&repo_id, &token, &new_file.id).unwrap();
        assert_eq!("Hallo Header", header.header);

        let (header, content) = actor.get_file(&repo_id, &token, &new_file.id).unwrap();
        assert_eq!("Hallo Header", header.header);
        assert_eq!(vec![4, 2], content);

        let new_file = actor.update_header(&repo_id, &token, &new_file.id, new_file.version, "New Header").unwrap();
        let header = actor.get_file_header(&repo_id, &token, &new_file.descriptor.id).unwrap();
        assert_eq!("New Header", header.header);

        let new_file = actor.update_file(&repo_id, &token, &new_file.descriptor.id, new_file.descriptor.version, "Other Header", vec![47, 11]).unwrap();
        let (header, content) = actor.get_file(&repo_id, &token, &new_file.descriptor.id).unwrap();
        assert_eq!("Other Header", header.header);
        assert_eq!(vec![47, 11], content);

        actor.delete_file(&repo_id, &token, &new_file.descriptor.id, new_file.descriptor.version).unwrap();
        let files = actor.list_repository_files(&repo_id, &token).unwrap();
        assert_eq!(1, files.len());

        actor.close_repository(&repo_id, &token).unwrap();
    }

    #[test]
    fn test_create_repo_add_file() {
        let temp = TempDir::new("temp_repo").unwrap();

        let actor = CryptoActor::new(vec![temp.path().to_path_buf()]).unwrap();
        let repository = actor.create_repository("another repository", "bla".as_bytes().to_vec(), EncryptionType::RingAESGCM).unwrap();
        let repo_id = repository.id;
        let token = repository.token;

        let new_file = actor.create_new_file(&repo_id, &token, "Hallo Header".to_string(), vec![4, 2]).unwrap();
        actor.close_repository(&repo_id, &token).unwrap();
    }
}