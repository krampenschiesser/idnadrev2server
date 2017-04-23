use super::super::actor::{Actor, ActorControl};
use super::{RepoHeader, Repository, EncryptedFile};
use uuid::Uuid;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use super::io::{ScanResult, scan, Error};
use super::crypt::{PlainPw, HashedPw};

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

struct RepositoryState {
    files: HashMap<Uuid, EncryptedFile>,
    key: HashedPw,
    repo: Repository,
}

struct State {
    nonces: HashSet<Vec<u8>>,
    repositories: HashMap<Uuid, RepositoryState>,

    folders: Vec<PathBuf>,
    scan_result: ScanResult,
}

impl State {
    fn new(folders: Vec<PathBuf>) -> Result<Self, Error> {
        let result = scan(&folders)?;
        Ok(State { nonces: HashSet::new(), repositories: HashMap::new(), folders: Vec::new(), scan_result: result })
    }

    fn get_repository(&self, id: &Uuid) -> Option<&RepositoryState> {
        self.repositories.get(id)
    }
}


impl RepositoryState {
    pub fn new(repo: Repository, key: HashedPw) -> Self {
        RepositoryState { key: key, repo: repo, files: HashMap::new() }
    }
}

fn handle(cmd: CryptCmd, state: &mut State) -> Result<CryptResponse, String> {
    match &cmd {
        &CryptCmd::OpenRepository { ref id, ref pw } => open_repository(id, pw.as_slice(), state),
        _ => Err("dooo".to_string())
    }
}

fn open_repository(id: &Uuid, pw: &[u8], state: &mut State) -> Result<CryptResponse, String> {
    let pw = PlainPw::new(pw);
    let existing = state.repositories.get(&id);

    if existing.is_some() {
        let existing = existing.unwrap();
        let hashed = existing.repo.hash_key(pw);

        if hashed == existing.key {
            Ok(CryptResponse::RepositoryOpened { id: id.clone() })
        }else {
            Ok(CryptResponse::RepositoryOpenFailed { id: id.clone() })
        }
    } else {
        let option = state.scan_result.get_repository(id);
        match option {
            Some(repo) => {
                let hashed_key = repo.hash_key(pw);
                if repo.check_hashed_key(&hashed_key) {
                    let repostate = create_repository_state(hashed_key, repo, &state.scan_result);
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

fn create_repository_state(pw: HashedPw, repo: Repository, scan_result: &ScanResult) -> RepositoryState {
    let to_load = scan_result.get_files_for_repo(&repo.get_id());
    //    for (header, path) in to_load {
    //        EncryptedFile::load_head(header, )
    //    }
    RepositoryState::new(repo,pw)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crypt::{Repository, RepoHeader};
    use crypt;
    use tempdir::TempDir;
    use std::fs::File;
    use crypt::serialize::ByteSerialization;
    use std::io::Write;
    use super::super::crypt::{PlainPw,HashedPw};

    fn create_temp_repo() -> (TempDir, Repository, HashedPw) {
        let tempdir = TempDir::new("temp_repo").unwrap();
        let header = RepoHeader::new_for_test();
        let pw = PlainPw::new("password".as_bytes());
        let repo = crypt::Repository::new("Hallo Repo".into(), pw.clone(), header);
        let pw_hash = repo.hash_key(pw);

        let file_header = crypt::FileHeader::new(&repo.header);
        let mut file = crypt::EncryptedFile::new(file_header, "test header");
        {
            let mut dir = tempdir.path();
            let mut buff = Vec::new();
            repo.to_bytes(&mut buff);
            let mut f = File::create(dir.join("repo")).unwrap();
            f.write(buff.as_slice());

            file.set_path(&dir.join("file"));
            file.set_content("hallo content".as_bytes());
            file.save(&pw_hash).unwrap();
        }
        (tempdir, repo, pw_hash)
    }

    use std::mem::size_of;

    #[test]
    fn bla() {
        let v = size_of::<usize>() * 8;
        println!("{}", v);
    }

    #[test]
    fn open_repo() {
        let (temp, repo, pw) = create_temp_repo();
        let dir = temp.path().into();
        let pw = "password".as_bytes();
        let pw_wrong = "hello".as_bytes();

        let id = repo.get_id();
        let mut state = crypt::actor::State::new(vec![dir]).unwrap();
        let response = open_repository(&id, pw, &mut state).unwrap();
        assert_eq!(CryptResponse::RepositoryOpened { id: id }, response);

        let response = open_repository(&id, pw_wrong, &mut state).unwrap();
        assert_eq!(CryptResponse::RepositoryOpenFailed { id: id }, response);

        let state = state;
        assert_eq!(1, state.repositories.len());
        let ref repostate = state.get_repository(&id).unwrap();
        assert_eq!(1, repostate.files.len());
        let (id, file) = repostate.files.iter().next().unwrap();
        assert_eq!("test header", file.get_header().as_str());
    }
}