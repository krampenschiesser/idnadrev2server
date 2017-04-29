use super::communication::{CryptCmd, CryptResponse};
use super::dto::{FileHeaderDescriptor, FileDescriptor, RepositoryDescriptor};
use super::state::State;
use super::super::structs::FileVersion;
use super::super::structs::crypto::{PlainPw, HashedPw};
use super::super::structs::repository::{Repository};
use super::super::structs::file::{EncryptedFile, FileHeader};
use super::state::scanresult::ScanResult;
use super::state::repositorystate::RepositoryState;
use super::super::util::io::{path_to_str, read_file_header, read_repo_header};
use super::super::error::{CryptError, ParseError};

use uuid::Uuid;
use log::LogLevel;
use std::path::PathBuf;

fn handle(cmd: CryptCmd, state: &mut State) -> Result<CryptResponse, String> {
    match &cmd {
        &CryptCmd::OpenRepository { ref id, ref pw } => open_repository(id, pw.as_slice(), state),
        &CryptCmd::CloseRepository { ref id, ref token } => close_repository(id, token, state),
        &CryptCmd::ListFiles { ref id, ref token } => list_files(id, token, state),
        &CryptCmd::ListRepositories => list_repositories(state),
        &CryptCmd::CreateNewFile { ref token, ref header, ref content, ref repo } => create_new_file(token, header, content, repo, state),
        &CryptCmd::FileAdded(ref path) => file_added(path, state),
        &CryptCmd::FileChanged(ref path) => file_changed(path, state),
        &CryptCmd::FileDeleted(ref path) => file_deleted(path, state),
        _ => Err("dooo".to_string())
    }
}


fn open_repository(id: &Uuid, pw: &[u8], state: &mut State) -> Result<CryptResponse, String> {
    let pw = PlainPw::new(pw);

    if state.has_repository(id) {
        let mut existing = state.get_repository_mut(id).unwrap();
        let hashed = existing.get_repo().hash_key(pw);

        if &hashed == existing.get_key() {
            let token = existing.generate_token();
            debug!("Generate new token for already opened repository {}. Token: {}", id, &token);
            Ok(CryptResponse::RepositoryOpened { token: token, id: id.clone() })
        } else {
            Ok(CryptResponse::RepositoryOpenFailed { id: id.clone() })
        }
    } else {
        let option = state.get_scan_result().get_repository(id);
        match option {
            Some(repo) => {
                let hashed_key = repo.hash_key(pw);
                if repo.check_hashed_key(&hashed_key) {
                    let mut repostate = create_repository_state(hashed_key, repo, state.get_scan_result());
                    let token = repostate.generate_token();
                    state.add_repository(&id, repostate);
                    debug!("Opened repository {} with token: {}", id, &token);

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
        match EncryptedFile::load_head(&header, &repo_state.get_key(), &path) {
            Ok(f) => {
                repo_state.add_file(f);
            }
            Err(e) => {
                repo_state.add_error((path, format!("{}", e)));
            }
        }
    }
    repo_state
}

fn close_repository(id: &Uuid, token: &Uuid, state: &mut State) -> Result<CryptResponse, String> {
    if state.check_token(token, id) {
        state.remove_token(id, token);
        Ok(CryptResponse::RepositoryIsClosed { id: id.clone() })
    } else {
        invalid_token("Trying to close a repository", token)
    }
}

fn list_files(id: &Uuid, token: &Uuid, state: &mut State) -> Result<CryptResponse, String> {
    if state.check_token(token, id) {
        let repo = state.get_repository(id).unwrap();//unwrap because check_token returns false on no repo
        let files: Vec<FileHeaderDescriptor> = repo.get_file_headers();

        Ok(CryptResponse::Files(files))
    } else {
        invalid_token("Trying to list files of an unknown repository", token)
    }
}


fn list_repositories(state: &mut State) -> Result<CryptResponse, String> {
    let repos: Vec<RepositoryDescriptor> = state.get_scanned_repositories().iter().map(|r| RepositoryDescriptor::new(r)).collect();
    Ok(CryptResponse::Repositories(repos))
}


fn create_new_file(token: &Uuid, header: &String, content: &Vec<u8>, repo_id: &Uuid, state: &mut State) -> Result<CryptResponse, String> {
    let result = if state.check_token(token, repo_id) {
        let repostate = state.get_repository(repo_id).unwrap();
        let ref repo = repostate.get_repo();
        let fh = FileHeader::new(&repo.get_header());
        let file_id = fh.get_id();
        let mut file = EncryptedFile::new(fh, header);
        file.set_content(content);
        let file_path = repo.get_folder().unwrap().join(format!("{}", file_id.simple()));
        file.set_path(&file_path);
        file.save(repostate.get_key());
        info!("Successfully created new file {} in {}", file_id, path_to_str(&file_path));

        Ok((FileDescriptor::new(&file.get_encryption_header()), file_path))
    } else {
        Err(invalid_token_response_only("Trying to create file with invalid token", token))
    };
    match result {
        Ok((descriptor, path)) => {
            handle(CryptCmd::FileAdded(path), state);
            Ok(CryptResponse::FileCreated(descriptor))
        }
        Err(response) => Ok(response)
    }
}

fn update_file_header(token: &Uuid, file_descriptor: &FileDescriptor, header: &String, state: &mut State) -> Result<CryptResponse, String> {
    let file_id = &file_descriptor.id;
    let repo_id = &file_descriptor.repo;
    let result = if state.check_token(token, repo_id) {
        let mut repostate = state.get_repository_mut(repo_id).unwrap();
        let key = repostate.get_key().clone();
        let mut o = repostate.get_file_mut(file_id);

        let cloned_descriptor: FileDescriptor = file_descriptor.clone();
        match o {
            Some(file) => {
                let current_version = file.get_encryption_header().get_version();
                if current_version <= file_descriptor.version {
                    let mut cloned = file.clone();
                    cloned.set_header(header);
                    match cloned.update_header(&key) {
                        Ok(_) => Ok(file.get_path().unwrap()),
                        Err(e) => {
                            let error = format!("Could not update header of {} : {:?}", cloned.get_id(), e);
                            error!("{}", error);
                            Err(CryptResponse::Error(error.to_string()))
                        }
                    }
                } else {
                    Err(CryptResponse::OptimisticLockError { file: cloned_descriptor, file_version: current_version })
                }
            }
            None => Err(CryptResponse::NoSuchFile(cloned_descriptor))
        }
    } else {
        Err(invalid_token_response_only("Trying to update file with invalid token", token))
    };
    match result {
        Ok(path) => {
            let res = handle(CryptCmd::FileChanged(path), state);
            match res {
                Ok(CryptResponse::FileChanged(descriptor)) => {
                    let header = state.get_repository(&descriptor.repo).unwrap().get_file(&descriptor.id).unwrap().get_header().to_string();
                    let descriptor = FileHeaderDescriptor { header: header, descriptor: descriptor };
                    Ok(CryptResponse::File(descriptor))
                }
                Ok(other) => Ok(other),
                Err(r) => Err(r)
            }
        }
        Err(response) => Ok(response)
    }
}

fn unrecognized_file(msg: String, level: LogLevel) -> Result<CryptResponse, String> {
    log!(level, "{}", msg);
    Ok(CryptResponse::UnrecognizedFile(msg))
}

fn file_added(path: &PathBuf, state: &mut State) -> Result<CryptResponse, String> {
    create_or_update_file(path, state, true)
}

fn create_or_update_file(path: &PathBuf, state: &mut State, create: bool) -> Result<CryptResponse, String> {
    let result = read_file_header(path).unwrap();
    match read_file_header(path) {
        Ok(file_header) => {
            let id = file_header.get_id();
            let repo_id = file_header.get_repository_id();
            let version = file_header.get_version();

            let descriptor = FileDescriptor::new(&file_header);
            state.update_file(file_header, path.clone())?;

            if create {
                Ok(CryptResponse::FileCreated(descriptor))
            } else {
                //                let header = state.get_repository(&repo_id).unwrap().get_file(&id).unwrap().get_header().to_string();
                //                let descriptor = FileHeaderDescriptor { header: header, descriptor: descriptor };
                Ok(CryptResponse::FileChanged(descriptor))
            }
        }
        Err(CryptError::ParseError(ParseError::NoPrefix)) => {
            unrecognized_file(format!("Ignoring {}  because it has no matching prefix.", path_to_str(path)), LogLevel::Debug)
        }
        Err(CryptError::ParseError(ParseError::InvalidFileVersion(file_version))) => {
            if file_version == FileVersion::RepositoryV1 {
                let repo_result = read_repo_header(path);
                match repo_result {
                    _ => unimplemented!()
                }
            } else {
                unrecognized_file(format!("Ignoring {} because it has an unkown file version.", path_to_str(path)), LogLevel::Warn)
            }
        }
        Err(CryptError::ParseError(ParseError::UnknownFileVersion(v))) => {
            unrecognized_file(format!("Ignoring {} because it has an unkown file version {}.", path_to_str(path), v), LogLevel::Warn)
        }
        _ => unrecognized_file(format!("Ignoring {} because of general read error: {:?}", path_to_str(path), result), LogLevel::Error)
    }
}

fn file_changed(path: &PathBuf, state: &mut State) -> Result<CryptResponse, String> {
    create_or_update_file(path, state, false)
}

fn file_deleted(path: &PathBuf, state: &mut State) -> Result<CryptResponse, String> {
    let o = state.get_scan_result().get_file_for_path(path.clone());
    match o {
        Some(header) => {
            let id = header.get_id();
            let repo_id = header.get_repository_id();
            state.remove_file(&repo_id, &id);
            let descriptor = FileDescriptor::new(&header);
            Ok(CryptResponse::FileDeleted(descriptor))
        }
        None => Ok(CryptResponse::UnrecognizedFile(path_to_str(path))),
    }
}

fn invalid_token(msg: &str, token: &Uuid) -> Result<CryptResponse, String> {
    Ok(invalid_token_response_only(msg, token))
}

fn invalid_token_response_only(msg: &str, token: &Uuid) -> CryptResponse {
    let ret = format!("No valid access token {}: {}", token, msg);
    warn!("{}", ret);
    CryptResponse::InvalidToken(ret)
}


#[cfg(test)]
mod tests {
    use super::*;
    use super::super::super::structs::repository::{Repository, RepoHeader};
    use super::super::super::structs::file::{FileHeader, EncryptedFile};
    use super::super::super::structs::crypto::{PlainPw, HashedPw};
    use super::super::super::structs::serialize::ByteSerialization;
    use tempdir::TempDir;
    use std::fs::{File, remove_file};
    use std::io::Write;
    use spectral::prelude::*;
    use std::time::{Instant, Duration};
    use log4rs;
    use super::super::super::util::io::{check_map_path};
    use super::super::state::scanresult::CheckRes;

    fn create_temp_repo() -> (TempDir, Repository, HashedPw) {
        let tempdir = TempDir::new("temp_repo").unwrap();
        let header = RepoHeader::new_for_test();
        let pw = PlainPw::new("password".as_bytes());
        let repo = Repository::new("Hallo Repo".into(), pw.clone(), header);
        let pw_hash = repo.hash_key(pw);

        let file_header = FileHeader::new(repo.get_header());
        let mut file = EncryptedFile::new(file_header, "test header");
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
    fn test_open_repo() {
        let (temp, repo, pw) = create_temp_repo();
        let dir = temp.path().into();
        let pw = "password".as_bytes();
        let pw_wrong = "hello".as_bytes();

        let id = repo.get_id();
        let mut state = State::new(vec![dir]).unwrap();
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
        assert_eq!(1, state.get_repositories().len());
        let ref repostate = state.get_repository(&id).unwrap();
        assert_eq!(1, repostate.get_files().len());
        let (id, file) = repostate.get_files().iter().next().unwrap();
        assert_eq!("test header", file.get_header().as_str());
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn test_token() {
        let header = RepoHeader::new_for_test();
        let repo = Repository::new("test", "hello".into(), header);
        let pw = repo.hash_key("hello".into());
        let mut state = RepositoryState::new(repo, pw);
        let token = state.generate_token();

        assert_eq!(true, state.check_token(&token));
        let mut long_ago = Instant::now() - Duration::from_secs(60 * 21);
        state.set_token_time(&token, long_ago);
        assert_eq!(false, state.check_token(&token));

        let token = state.generate_token();
        assert_eq!(false, state.check_token(&Uuid::new_v4()));

        let mut long_ago = Instant::now() - Duration::from_secs(5);
        state.set_token_time(&token, long_ago);
        state.check_token(&token);

        assert_that(&state.get_token_time(&token).elapsed().as_secs()).is_less_than(&10);
    }

    #[test]
    fn test_close_repo() {
        let (temp, repo, pw) = create_temp_repo();
        let dir = temp.path().into();

        let id = repo.get_id();
        let mut state = State::new(vec![dir]).unwrap();

        let pw = "password".as_bytes();
        let token1 = open_repo_get_token(&id, pw, &mut state);
        let token2 = open_repo_get_token(&id, pw, &mut state);
        assert_ne!(token1, token2);

        let invalid_token = Uuid::new_v4();
        let response = close_repository(&id, &invalid_token, &mut state).unwrap();
        let result = match response {
            CryptResponse::InvalidToken(_) => true,
            _ => false
        };
        assert_eq!(true, result, "Should have an error invalid token, but did not!");

        let response = close_repository(&id, &token1, &mut state).unwrap();
        let result = match response {
            CryptResponse::RepositoryIsClosed { id: res_id } => res_id == id,
            _ => false
        };
        assert_eq!(true, result, "Should have received a close of repo, but did not!");

        assert_eq!(1, state.get_repositories().len());
        let response = close_repository(&id, &token2, &mut state).unwrap();
        assert_eq!(0, state.get_repositories().len());
    }

    fn open_repo_get_token(id: &Uuid, pw: &[u8], state: &mut State) -> Uuid {
        let response = open_repository(id, pw, state);
        match response.unwrap() {
            CryptResponse::RepositoryOpened { token, id } => token.clone(),
            _ => panic!("no result token"),
        }
    }

    #[test]
    fn test_list_files() {
        let (temp, repo, pw) = create_temp_repo();
        let dir = temp.path().into();

        let pw = "password".as_bytes();
        let id = repo.get_id();
        let mut state = State::new(vec![dir]).unwrap();
        let token = open_repo_get_token(&id, pw, &mut state);
        let response = list_files(&id, &token, &mut state).unwrap();
        match response {
            CryptResponse::Files(f) => {
                assert_eq!(1, f.len());
                assert_eq!("test header".to_string(), f[0].header);
            }
            _ => panic!("Got invalid response {:?}", &response)
        }
    }

    #[test]
    fn test_list_repos() {
        let (temp, repo, pw) = create_temp_repo();
        let dir = temp.path().into();

        let pw = "password".as_bytes();
        let id = repo.get_id();
        let mut state = State::new(vec![dir]).unwrap();

        let response = list_repositories(&mut state).unwrap();
        match response {
            CryptResponse::Repositories(v) => {
                assert_eq!(1, v.len());
                assert_eq!("Hallo Repo".to_string(), v[0].name);
            }
            _ => panic!("Got invalid response {:?}", &response)
        }
    }

    #[test]
    fn test_add_file() {
        let (temp, repo, pw) = create_temp_repo();
        let dir = temp.path().to_path_buf();

        let pw = "password".as_bytes();
        let id = repo.get_id();
        let mut state = State::new(vec![dir.clone()]).unwrap();

        let token = open_repo_get_token(&id, &pw, &mut state);

        let response = create_new_file(&token, &"test header 2".to_string(), &"content2".as_bytes().to_vec(), &id, &mut state).unwrap();
        let file_id = match response {
            CryptResponse::FileCreated(d) => {
                assert_eq!(0, d.version);
                assert_eq!(&id, &d.repo);
                d.id
            }
            _ => panic!("Got invalid response {:?}", &response)
        };

        let mut found = false;
        for file in dir.read_dir().unwrap() {
            let result = check_map_path(&file.unwrap().path());
            info!("Found {:?}", &result);
            match result {
                Err(_) => {}
                Ok(c) => match c {
                    CheckRes::File(h, _) => {
                        if h.get_id() == file_id {
                            found = true;
                        }
                    }
                    _ => {}
                }
            }
        }
        if !found {
            panic!("Did not write file!");
        }
        state.get_repository(&repo.get_id()).unwrap().get_files().get(&file_id).unwrap();
    }

    fn create_repo_and_file<'a>() -> (Uuid, Uuid, &'a [u8], Uuid, State, TempDir) {
        let (temp, repo, pw) = create_temp_repo();
        let dir = temp.path().to_path_buf();

        let pw = "password".as_bytes();
        let id = repo.get_id();
        let mut state = State::new(vec![dir.clone()]).unwrap();

        let token = open_repo_get_token(&id, &pw, &mut state);

        let response = create_new_file(&token, &"test header 2".to_string(), &"content2".as_bytes().to_vec(), &id, &mut state).unwrap();
        let file_id = match response {
            CryptResponse::FileCreated(d) => {
                assert_eq!(0, d.version);
                assert_eq!(&id, &d.repo);
                d.id
            }
            _ => panic!("Got invalid response {:?}", &response)
        };
        (token, file_id, pw, id, state, temp)
    }

    #[test]
    fn test_update_header() {
        let (token, file_id, pw_bytes, repo_id, mut state, temp) = create_repo_and_file();

        let descriptor = FileDescriptor { id: file_id, repo: repo_id, version: 0 };
        let result = update_file_header(&token, &descriptor, &"bla".to_string(), &mut state).unwrap();
        match result {
            CryptResponse::File(desc) => {
                assert_eq!("bla", desc.header);
                assert_eq!(1, desc.descriptor.version);
            }
            _ => panic!("Did not update file. Result: {:?}", result),
        }
    }

    #[test]
    fn test_file_added() {
        let (token, file_id, pw_bytes, repo_id, mut state, temp) = create_repo_and_file();

        let path = {
            let encrypted_file = state.get_repository(&repo_id).unwrap().get_file(&file_id).unwrap().clone();
            let p = encrypted_file.get_path().unwrap();

            state.get_repository_mut(&repo_id).unwrap().get_files_mut().clear();
            p
        };

        let result = file_added(&path, &mut state);
        match result {
            Ok(CryptResponse::FileCreated(desc)) => {
                assert_eq!(desc.id, file_id);
                assert_eq!(desc.repo, repo_id);
            }
            Ok(o) => {
                panic!("Received invalid response {:?}", o);
            }
            Err(e) => {
                panic!("Should have added file to repo but got {:?}", e);
            }
        }
    }

    #[test]
    fn test_file_changed() {
        let (token, file_id, pw_bytes, repo_id, mut state, temp) = create_repo_and_file();

        let path = {
            let key = state.get_repository(&repo_id).unwrap().get_key().clone();
            let mut encrypted_file: EncryptedFile = state.get_repository_mut(&repo_id).unwrap().get_file_mut(&file_id).unwrap().clone();
            encrypted_file.set_header("HUHU");
            encrypted_file.update_header(&key);

            let p = encrypted_file.get_path().unwrap();
            p
        };

        let result = file_changed(&path, &mut state);
        match result {
            Ok(CryptResponse::FileChanged(desc)) => {
                assert_eq!(desc.id, file_id);
                assert_eq!(desc.repo, repo_id);
            }
            Ok(o) => {
                panic!("Received invalid response {:?}", o);
            }
            Err(e) => {
                panic!("Should have added file to repo but got {:?}", e);
            }
        }

        let file = state.get_repository_mut(&repo_id).unwrap().get_file_mut(&file_id).unwrap().clone();
        assert_eq!("HUHU", file.get_header().as_str());
    }

    #[test]
    fn test_file_deleted() {
        let (token, file_id, pw_bytes, repo_id, mut state, temp) = create_repo_and_file();
        let path = {
            let encrypted_file: EncryptedFile = state.get_repository_mut(&repo_id).unwrap().get_file_mut(&file_id).unwrap().clone();
            let p = encrypted_file.get_path().unwrap();
            p
        };
        remove_file(&path);

        let result = file_deleted(&path, &mut state);
        match result {
            Ok(CryptResponse::FileDeleted(desc)) => {
                assert_eq!(desc.id, file_id);
                assert_eq!(desc.repo, repo_id);
            }
            Ok(o) => {
                panic!("Received invalid response {:?}", o);
            }
            Err(e) => {
                panic!("Should have added file to repo but got {:?}", e);
            }
        }

        assert!(!state.get_repository_mut(&repo_id).unwrap().get_files().contains_key(&file_id));
        assert!(!state.get_scan_result().has_file(&file_id));
    }

    //fixme add tests for repo added/updated/deleted
}