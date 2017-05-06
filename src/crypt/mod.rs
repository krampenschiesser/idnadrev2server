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
use self::actor::communication::*;
use self::actor::handle;
use actor::{ActorControl, Actor};
pub use self::actor::dto::*;

use std::path::PathBuf;
use std::thread;
use uuid::Uuid;

pub struct CryptoActor {
    actor_control: ActorControl<CryptCmd, CryptResponse>,
}

impl CryptoActor {
    pub fn new(folders: Vec<PathBuf>) -> Result<Self, CryptError> {
        let mut state = State::new(folders)?;


        let (mut actor, control) = Actor::start(state, handle, CryptCmd::Shutdown);
        let handle = thread::spawn(move || actor.run());

        Ok(CryptoActor { actor_control: control })
    }
}

impl CryptoActor {
    fn send_unwrap(&self, cmd: CryptCmd) -> Option<CryptResponse> {
        let response = self.actor_control.send_sync(cmd.clone());
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

    pub fn list_repositories(&self) -> Option<Vec<RepositoryDescriptor>> {
        let cmd = CryptCmd::ListRepositories;

        if let Some(response) = self.send_unwrap(cmd) {
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

    pub fn open_repository(&self, id: &Uuid, user_name: String, pw: Vec<u8>) -> Option<AccessToken> {
        let cmd = CryptCmd::OpenRepository { id: id.clone(), user_name: user_name, pw: pw };

        if let Some(response) = self.send_unwrap(cmd) {
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

    pub fn create_repository(&self, name: &str, pw: Vec<u8>, enc_type: EncTypeDto) -> Option<RepositoryDto> {
        #[cfg(debug_assertions)]
        let scrypt = PwKdfDto::SCrypt { iterations: 4, memory_costs: 4, parallelism: 1 };
        #[cfg(not(debug_assertions))]
        let scrypt = PwKdfDto::SCrypt { iterations: 16, memory_costs: 8, parallelism: 1 };

        let cmd = CryptCmd::CreateRepository { name: name.to_string(), pw: pw, folder_id: None, encryption: EncTypeDto::ChaCha, kdf: scrypt };

        if let Some(response) = self.send_unwrap(cmd) {
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

    pub fn close_repository(&self, id: &Uuid, token: &AccessToken) -> Option<Uuid> {
        let cmd = CryptCmd::CloseRepository { id: id.clone(), token: token.clone() };

        if let Some(response) = self.send_unwrap(cmd) {
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

    pub fn list_repository_files(&self, id: &Uuid, token: &AccessToken) -> Option<Vec<FileHeaderDescriptor>> {
        let cmd = CryptCmd::ListFiles { id: id.clone(), token: token.clone() };

        if let Some(response) = self.send_unwrap(cmd) {
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
    pub fn create_new_file(&self, repo_id: &Uuid, token: &AccessToken, header: String, content: Vec<u8>) -> Option<FileDescriptor> {
        let cmd = CryptCmd::CreateNewFile { token: token.clone(), header: header, repo: repo_id.clone(), content: content };

        if let Some(response) = self.send_unwrap(cmd) {
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
    pub fn get_file_header(&self, repo_id: &Uuid, token: &AccessToken, file_id: &Uuid) -> Option<FileHeaderDescriptor> {
        let cmd = CryptCmd::GetFileHeader { token: token.clone(), file: FileDescriptor { repo: repo_id.clone(), id: file_id.clone(), version: 0 } };

        if let Some(response) = self.send_unwrap(cmd) {
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

    pub fn get_file(&self, repo_id: &Uuid, token: &AccessToken, file_id: &Uuid) -> Option<(FileHeaderDescriptor, Vec<u8>)> {
        let cmd = CryptCmd::GetFile { token: token.clone(), file: FileDescriptor { repo: repo_id.clone(), id: file_id.clone(), version: 0 } };

        if let Some(response) = self.send_unwrap(cmd) {
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

    pub fn update_header(&self, repo_id: &Uuid, token: &AccessToken, file_id: &Uuid, file_version: u32, header: &str) -> Option<FileHeaderDescriptor> {
        let desc = FileDescriptor { repo: repo_id.clone(), id: file_id.clone(), version: file_version };
        let cmd = CryptCmd::UpdateHeader { token: token.clone(), file: desc, header: header.to_string() };

        if let Some(response) = self.send_unwrap(cmd) {
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

    pub fn update_file(&self, repo_id: &Uuid, token: &AccessToken, file_id: &Uuid, file_version: u32, header: &str, content: Vec<u8>) -> Option<FileHeaderDescriptor> {
        let desc = FileDescriptor { repo: repo_id.clone(), id: file_id.clone(), version: file_version };
        let cmd = CryptCmd::UpdateFile { token: token.clone(), file: desc, header: header.to_string(), content: content };

        if let Some(response) = self.send_unwrap(cmd) {
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

    pub fn delete_file(&self, repo_id: &Uuid, token: &AccessToken, file_id: &Uuid, file_version: u32) -> Option<FileDescriptor> {
        let desc = FileDescriptor { repo: repo_id.clone(), id: file_id.clone(), version: file_version };
        let cmd = CryptCmd::DeleteFile { token: token.clone(), file: desc };

        if let Some(response) = self.send_unwrap(cmd) {
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

    pub fn check_token(&self, repo_id: &Uuid, token: &AccessToken) -> bool {
        let cmd = CryptCmd::CheckToken { repo: repo_id.clone(), token: token.clone()};

        if let Some(response) = self.send_unwrap(cmd) {
            match response {
                CryptResponse::TokenValid => true,
                _ => false

            }
        } else {
            false
        }
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
        let repository = actor.create_repository("another repository", "bla".as_bytes().to_vec(), EncTypeDto::AES).unwrap();
        let repo_id = repository.id;
        let token = repository.token;

        let new_file = actor.create_new_file(&repo_id, &token, "Hallo Header".to_string(), vec![4, 2]).unwrap();
        actor.close_repository(&repo_id, &token).unwrap();
    }
}