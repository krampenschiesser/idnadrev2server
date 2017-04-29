pub mod actor;
mod structs;
mod util;
mod error;

use self::actor::state::State;
use self::error::CryptError;
use self::actor::communication::*;
use self::actor::function::handle;
use actor::{ActorControl, Actor};
use self::actor::dto::*;

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
    //CreateNewFile { token: Uuid, header: String, content: Vec<u8>, repo: Uuid },
    //UpdateHeader { token: Uuid, header: String, file: FileDescriptor },
    //UpdateFile { token: Uuid, header: String, content: Vec<u8>, file: FileDescriptor },
    //DeleteFile { token: Uuid, file: FileDescriptor },
    //
    //CloseRepository { token: Uuid, id: Uuid },
    //
    //FileAdded(PathBuf),
    //FileChanged(PathBuf),
    //FileDeleted(PathBuf),
    //
    //    Shutdown,

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

    pub fn open_repository(&self, id: &Uuid, pw: Vec<u8>) -> Option<Uuid> {
        let cmd = CryptCmd::OpenRepository { id: id.clone(), pw: pw };

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

    pub fn list_repository_files(&self, id: &Uuid, token: &Uuid) -> Option<Vec<FileHeaderDescriptor>> {
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
    pub fn create_new_file(&self, repo_id: &Uuid, token: &Uuid, header: String, content: Vec<u8>) -> Option<FileDescriptor> {
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
    pub fn get_file_header(&self, repo_id: &Uuid, token: &Uuid, file_id: &Uuid) -> Option<FileHeaderDescriptor> {
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
    pub fn get_file(&self, repo_id: &Uuid, token: &Uuid, file_id: &Uuid) -> Option<(FileHeaderDescriptor, Vec<u8>)> {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use self::super::actor::function::tests::create_temp_repo;

    #[test]
    fn test_start() {
        let (temp, repo, pw) = create_temp_repo();
        let repo_id = repo.get_id();

        let actor = CryptoActor::new(vec![temp.path().to_path_buf()]).unwrap();

        let repos = actor.list_repositories().unwrap();
        assert_eq!(1, repos.len());

        let token = actor.open_repository(&repo_id, "password".as_bytes().to_vec()).unwrap();
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
    }
}