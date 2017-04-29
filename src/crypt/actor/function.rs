use super::communication::{CryptCmd,CryptResponse};
use super::dto::{FileHeaderDescriptor,FileDescriptor,RepositoryDescriptor};
use super::state::State;
use super::super::structs::FileVersion;
use super::super::structs::crypto::{PlainPw,HashedPw};
use super::super::structs::repository::{Repository};
use super::super::structs::file::{EncryptedFile,FileHeader};
use super::state::scanresult::ScanResult;
use super::state::repositorystate::RepositoryState;
use super::super::util::io::{path_to_str,read_file_header,read_repo_header};
use super::super::error::{CryptError,ParseError};

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
    let repos: Vec<RepositoryDescriptor> = state.get_repositories().iter().map(|r| RepositoryDescriptor::new(r)).collect();
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
        Ok(path) => handle(CryptCmd::FileChanged(path), state),
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
                let header = state.get_repository(&repo_id).unwrap().get_file(&id).unwrap().get_header().to_string();
                let descriptor = FileHeaderDescriptor { header: header, descriptor: descriptor };
                Ok(CryptResponse::File(descriptor))
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
    unimplemented!()
}

fn invalid_token(msg: &str, token: &Uuid) -> Result<CryptResponse, String> {
    Ok(invalid_token_response_only(msg, token))
}

fn invalid_token_response_only(msg: &str, token: &Uuid) -> CryptResponse {
    let ret = format!("No valid access token {}: {}", token, msg);
    warn!("{}", ret);
    CryptResponse::InvalidToken(ret)
}