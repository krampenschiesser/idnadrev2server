use super::super::actor::{Actor, ActorControl};
use super::{RepoHeader, Repository};
use uuid::Uuid;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use ring::constant_time::verify_slices_are_equal;
use super::io::{ScanResult, scan, Error};
use std::cell::RefCell;

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
pub struct RepositoryDescriptor {
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

    Repositories(RepositoryDescriptor),

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
    repo: Repository,
}

struct State {
    nonces: HashSet<Vec<u8>>,
    repositories: HashMap<Uuid, RepositoryState>,

    folders: Vec<PathBuf>,
    scan_result: ScanResult,
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
    fn new(folders: Vec<PathBuf>) -> Result<Self, Error> {
        let result = scan(&folders)?;
        Ok(State { nonces: HashSet::new(), repositories: HashMap::new(), folders: Vec::new(), scan_result: result })
    }
}


impl RepositoryState {}

fn handle(cmd: CryptCmd, state: &mut State) -> Result<CryptResponse, String> {
    match &cmd {
        &CryptCmd::OpenRepository { ref id, ref pw } => open_repository(id, pw.as_slice(), state),
        _ => Err("dooo".to_string())
    }
}

fn open_repository(id: &Uuid, pw: &[u8], state: &mut State) -> Result<CryptResponse, String> {
    let existing = state.repositories.get(&id);
    if existing.is_some() {
        let existing = existing.unwrap();
        let ref header = existing.repo.header;
        let ref pwh = header.password_hash_type;
        let hashed = pwh.hash(pw, header.encryption_type.key_len());

        let matching = verify_slices_are_equal(hashed.as_slice(), existing.key.as_slice());
        match matching {
            Ok(_) => Ok(CryptResponse::RepositoryOpened { id: id.clone() }),
            Err(_) => Ok(CryptResponse::RepositoryOpenFailed { id: id.clone() }),
        }
    } else {
        let option = state.scan_result.get_repository(id);
        match option {
            Some(repo) => {
                if repo.check_pw(pw) {
//                    let repostate = create_repository_state(pw, repo, &state.scan_result);
//                    state.repositories.insert(id.clone(), repostate);
                    Ok(CryptResponse::RepositoryOpened { id: id.clone() })
                } else {
                    Ok(CryptResponse::RepositoryOpenFailed { id: id.clone() })
                }
            }
            None => Ok(CryptResponse::RepositoryOpenFailed { id: id.clone() }),
        }
    }
}

//fn create_repository_state(pw: &[u8], repo: Repository, scan_result: &ScanResult) -> RepositoryState {
//    let to_load = scan_result.get_files_for_repo(&repo.get_id());
//
//}

#[cfg(test)]
mod tests {
    use super::*;
    use crypt::{Repository, RepoHeader};
    use crypt;
    use tempdir::TempDir;
    use std::fs::File;
    use crypt::serialize::ByteSerialization;
    use std::io::Write;

    fn create_temp_repo() -> (TempDir, Repository) {
        let tempdir = TempDir::new("temp_repo").unwrap();
        let header = RepoHeader::new_for_test();
        let repo = crypt::Repository::new("Hallo Repo".into(), "password".as_bytes(), header);
        {
            let mut dir = tempdir.path();
            let mut buff = Vec::new();
            repo.to_bytes(&mut buff);
            let mut f = File::create(dir.join("repo")).unwrap();
            f.write(buff.as_slice());
        }
        (tempdir, repo)
    }

    use std::mem::size_of;

    #[test]
    fn bla() {
        let v = size_of::<usize>() * 8;
        println!("{}", v);
    }

    #[test]
    fn open_repo() {
        let (temp, repo) = create_temp_repo();
        let dir = temp.path().into();
        let pw = "password".as_bytes();
        let pw_wrong = "hello".as_bytes();

        let id = repo.get_id();
        let mut state = self::State::new(vec![dir]).unwrap();
        let response = open_repository(&id, pw, &mut state).unwrap();
        assert_eq!(CryptResponse::RepositoryOpened { id: id }, response);

        let response = open_repository(&id, pw_wrong, &mut state).unwrap();
        assert_eq!(CryptResponse::RepositoryOpenFailed { id: id }, response);

        let state = state;
    }
}