use super::super::actor::{Actor, ActorControl};
use super::{RepoHeader};
use uuid::Uuid;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use ring::constant_time::verify_slices_are_equal;

#[derive(Debug, PartialEq, Eq)]
pub struct FileDescriptor {
    repo: Uuid,
    id: Uuid,
    version: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub struct FileHeader {
    descriptor: FileDescriptor,
    header: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Repository {
    id: Uuid,
    name: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CryptCmd {
    CreateNewFile { header: String, content: Vec<u8>, repo: RepoHeader },
    UpdateHeader { header: String, file: FileDescriptor },
    UpdateContent { content: Vec<u8>, file: FileDescriptor },
    DeleteFile(FileDescriptor),

    OpenRepository { id: Uuid, pw: Vec<u8> },
    CloseRepository(Uuid),

    FileAdded(PathBuf),
    FileChanged(PathBuf),
    FileDeleted(PathBuf),

    ListRepositories,
    ListFiles { repository: Uuid },
}

#[derive(Debug, PartialEq, Eq)]
pub enum CryptResponse {
    FileCreated(FileDescriptor),
    FileDeleted(FileDescriptor),

    File(FileHeader),
    FileContent(FileHeader, Vec<u8>),
    Files(Vec<FileHeader>),

    Repositories(Repository),

    RepositoryOpened { id: Uuid },
    RepositoryOpenFailed { id: Uuid },
    RepositoryIsClosed { id: Uuid },

    OptimisticLockError { file: FileDescriptor, file_version: u32 },
    NoSuchFile(FileDescriptor),
}

struct FileState {
    descriptor: FileDescriptor,
    header: String,
    content: Option<Vec<u8>>,
    path: PathBuf,
}

struct RepositoryState {
    files: HashMap<Uuid, FileState>,
    key: Vec<u8>,
    header: RepoHeader,
}

struct State {
    nonces: HashSet<Vec<u8>>,
    repositories: HashMap<Uuid, RepositoryState>,

    folders: Vec<PathBuf>,
}

impl FileState {
    fn size(&self) -> usize {
        let content_size = match self.content {
            None => 0,
            Some(ref v) => v.len(),
        };
        content_size + self.header.as_bytes().len()
    }
}

impl State {
    fn new() -> Self {
        State { nonces: HashSet::new(), repositories: HashMap::new(), folders: Vec::new() }
    }
}


impl RepositoryState {
    //    fn new(header: RepoHeader, pw: &[u8], files: Vec<FileDescriptor>) -> Self {}
}

fn handle(cmd: CryptCmd, state: &mut State) -> Result<CryptResponse, String> {
    match cmd {
        //        CryptCmd::OpenRepository { id, pw } => { open_repository(&id, pw.as_slice(), &mut state) }
        _ => Err("dooo".to_string())
    }
}

fn open_repository(id: &Uuid, pw: &[u8], state: &mut State) -> Result<CryptResponse, String> {
    let existing = state.repositories.get(&id);
    if existing.is_some() {
        let existing = existing.unwrap();
        let ref pwh = existing.header.password_hash_type;
        let hashed = pwh.hash(pw, existing.header.encryption_type.key_len());

        let matching = verify_slices_are_equal(hashed.as_slice(), existing.key.as_slice());
        match matching {
            Ok(_) => Ok(CryptResponse::RepositoryOpened { id: id.clone() }),
            Err(_) => Ok(CryptResponse::RepositoryOpenFailed { id: id.clone() }),
        }
    } else {
        //        let repo_files = super::io::get_repo_files(&state.folders);
        Err("bla".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;


    #[test]
    #[ignore]
    fn open_repo() {
        let pw = "hello".as_bytes();
        let pw_wrong = "hello123".as_bytes();

        let id = Uuid::new_v4();
        let mut state = self::State::new();
        let response = open_repository(&id, pw, &mut state).unwrap();
        assert_eq!(CryptResponse::RepositoryOpened { id: id }, response);

        let response = open_repository(&id, pw_wrong, &mut state).unwrap();
        assert_eq!(CryptResponse::RepositoryOpenFailed { id: id }, response);
    }
}