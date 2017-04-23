use super::super::actor::{Actor, ActorControl};
use super::{RepoHeader, Repository, EncryptedFile};
use uuid::Uuid;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use super::io::{ScanResult, scan};
use super::crypt::{PlainPw, HashedPw};
use super::error::*;
use std::time::Instant;
use chrono::Duration;
use std::ops::Sub;

#[derive(Debug, PartialEq, Eq, Clone)]
struct AccessToken {
    last_access: Instant,
    id: Uuid,
}

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
    CreateNewFile { token: Uuid, header: String, content: Vec<u8>, repo: RepoHeader },
    UpdateHeader { token: Uuid, header: String, file: FileDescriptor },
    UpdateContent { token: Uuid, content: Vec<u8>, file: FileDescriptor },
    DeleteFile { token: Uuid, file: FileDescriptor },

    OpenRepository { id: Uuid, pw: Vec<u8> },
    CloseRepository { token: Uuid, id: Uuid },

    FileAdded(PathBuf),
    FileChanged(PathBuf),
    FileDeleted(PathBuf),

    ListRepositories,
    ListFiles { token: Uuid, repository: Uuid },
}

#[derive(Debug, PartialEq, Eq)]
pub enum CryptResponse {
    FileCreated(FileDescriptor),
    FileDeleted(FileDescriptor),

    File(FileHeader),
    FileContent(FileHeader, Vec<u8>),
    Files(Vec<FileHeader>),

    Repositories(RepositoryDescriptor),

    RepositoryOpened { token: Uuid, id: Uuid },
    RepositoryOpenFailed { id: Uuid },
    RepositoryIsClosed { id: Uuid },

    OptimisticLockError { file: FileDescriptor, file_version: u32 },
    NoSuchFile(FileDescriptor),
    AccessDenied,
    Error(String),
}

fn handle(cmd: CryptCmd, state: &mut State) -> Result<CryptResponse, String> {
    match &cmd {
        &CryptCmd::OpenRepository { ref id, ref pw } => open_repository(id, pw.as_slice(), state),
        &CryptCmd::CloseRepository { ref id, ref token } => {
            if state.check_token(token, id) {
                close_repository(id, state);
                Err("dooo".to_string())
            } else {
                invalid_token("Trying to close a repository", token)
            }
        }
        _ => Err("dooo".to_string())
    }
}

fn invalid_token(msg: &str, token: &Uuid) -> Result<CryptResponse, String> {
    let ret = format!("No valid access token {}: {}", token, msg);
    warn!("{}", ret);
    Ok(CryptResponse::Error(ret))
}


struct RepositoryState {
    files: HashMap<Uuid, EncryptedFile>,
    error_files: Vec<(PathBuf, String)>,
    key: HashedPw,
    repo: Repository,
    tokens: HashMap<Uuid, AccessToken>,
}

struct State {
    nonces: HashSet<Vec<u8>>,
    repositories: HashMap<Uuid, RepositoryState>,

    folders: Vec<PathBuf>,
    scan_result: ScanResult,
}

impl State {
    fn new(folders: Vec<PathBuf>) -> Result<Self, CryptError> {
        let result = scan(&folders)?;
        Ok(State { nonces: HashSet::new(), repositories: HashMap::new(), folders: Vec::new(), scan_result: result })
    }

    fn get_repository(&self, id: &Uuid) -> Option<&RepositoryState> {
        self.repositories.get(id)
    }
    fn get_repository_mut(&mut self, id: &Uuid) -> Option<&mut RepositoryState> {
        self.repositories.get_mut(id)
    }

    fn has_repository(&self, id: &Uuid) -> bool {
        self.repositories.contains_key(id)
    }

    fn add_repository(&mut self, id: &Uuid, repostate: RepositoryState) {
        self.repositories.insert(id.clone(), repostate);
    }

    fn check_token(&mut self, token: &Uuid, id: &Uuid) -> bool {
        let o = self.get_repository_mut(id);
        match o {
            Some(repo) => repo.check_token(token),
            None => false
        }
    }

    fn generate_token(&mut self, id: &Uuid) -> Option<Uuid> {
        let mut o = self.repositories.get_mut(id);
        match o {
            None => None,
            Some(ref mut r) => Some(r.generate_token())
        }
    }
}

impl AccessToken {
    fn new() -> Self {
        let id = Uuid::new_v4();
        AccessToken { id: id, last_access: Instant::now() }
    }

    fn touch(&mut self) {
        self.last_access = Instant::now();
    }
}


impl RepositoryState {
    fn new(repo: Repository, key: HashedPw) -> Self {
        RepositoryState { key: key, repo: repo, files: HashMap::new(), error_files: Vec::new(), tokens: HashMap::new() }
    }

    fn generate_token(&mut self) -> Uuid {
        let token = AccessToken::new();
        let retval = token.id.clone();
        self.tokens.insert(token.id.clone(), token);
        retval
    }

    fn check_token(&mut self, token: &Uuid) -> bool {
        let mut o = self.tokens.get_mut(token);
        match o {
            None => false,
            Some(ref mut t) => {
                let elapsed = t.last_access.elapsed();
                let elapsed = match Duration::from_std(elapsed) {
                    Ok(e) => e,
                    Err(_) => Duration::days(1),
                };
                if elapsed.num_minutes() > 20 {
                    false
                } else {
                    t.touch();
                    true
                }
            }
        }
    }

    #[cfg(test)]
    fn set_token_time(&mut self, token: &Uuid, time: Instant) {
        let mut t = self.tokens.get_mut(token).unwrap();
        t.last_access = time;
    }
    #[cfg(test)]
    fn get_token_time(&self, token: &Uuid) -> Instant {
        let t = self.tokens.get(token).unwrap();
        t.last_access
    }
}

fn open_repository(id: &Uuid, pw: &[u8], state: &mut State) -> Result<CryptResponse, String> {
    let pw = PlainPw::new(pw);

    if state.has_repository(id) {
        let mut existing = state.get_repository_mut(id).unwrap();
        let hashed = existing.repo.hash_key(pw);

        if hashed == existing.key {
            let token = existing.generate_token();
            Ok(CryptResponse::RepositoryOpened { token: token, id: id.clone() })
        } else {
            Ok(CryptResponse::RepositoryOpenFailed { id: id.clone() })
        }
    } else {
        let option = state.scan_result.get_repository(id);
        match option {
            Some(repo) => {
                let hashed_key = repo.hash_key(pw);
                if repo.check_hashed_key(&hashed_key) {
                    let mut repostate = create_repository_state(hashed_key, repo, &state.scan_result);
                    let token = repostate.generate_token();
                    state.add_repository(&id, repostate);

                    Ok(CryptResponse::RepositoryOpened { id: id.clone(), token: token })
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
    let mut repo_state = RepositoryState::new(repo, pw);

    for (header, path) in to_load {
        match EncryptedFile::load_head(&header, &repo_state.key, &path) {
            Ok(f) => {
                repo_state.files.insert(f.get_id(), f);
            }
            Err(e) => {
                repo_state.error_files.push((path, format!("{}", e)));
            }
        }
    }
    repo_state
}

fn close_repository(id: &Uuid, state: &mut State) -> Result<CryptResponse, String> {
    Err("".into())
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
    use super::super::crypt::{PlainPw, HashedPw};
    use spectral::prelude::*;
    use std::time::Instant;
    use chrono::Duration;

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

    #[test]
    fn open_repo() {
        let (temp, repo, pw) = create_temp_repo();
        let dir = temp.path().into();
        let pw = "password".as_bytes();
        let pw_wrong = "hello".as_bytes();

        let id = repo.get_id();
        let mut state = crypt::actor::State::new(vec![dir]).unwrap();
        let response = open_repository(&id, pw, &mut state).unwrap();

        match response {
            CryptResponse::RepositoryOpened { token, id: resp_id } => {
                assert_eq!(id, resp_id);
            }
            _ => panic!("No valid response")
        }

        let response = open_repository(&id, pw_wrong, &mut state).unwrap();
        assert_eq!(CryptResponse::RepositoryOpenFailed { id: id }, response);

        let state = state;
        assert_eq!(1, state.repositories.len());
        let ref repostate = state.get_repository(&id).unwrap();
        assert_eq!(1, repostate.files.len());
        let (id, file) = repostate.files.iter().next().unwrap();
        assert_eq!("test header", file.get_header().as_str());
    }

    #[test]
    fn token() {
        let header = RepoHeader::new_for_test();
        let repo = Repository::new("test", "hello".into(), header);
        let pw = repo.hash_key("hello".into());
        let mut state = RepositoryState::new(repo, pw);
        let token = state.generate_token();

        assert_eq!(true, state.check_token(&token));
        let long_ago = Instant::now().sub(Duration::minutes(21).to_std().unwrap());
        state.set_token_time(&token, long_ago);
        assert_eq!(false, state.check_token(&token));

        let token = state.generate_token();
        assert_eq!(false, state.check_token(&Uuid::new_v4()));

        let long_ago = Instant::now().sub(Duration::minutes(5).to_std().unwrap());
        state.set_token_time(&token, long_ago);
        state.check_token(&token);

        assert_that(&state.get_token_time(&token).elapsed().as_secs()).is_less_than(&10);
    }
}