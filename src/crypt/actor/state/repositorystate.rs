use std::collections::{HashMap};
use std::collections::hash_map::Values;
use super::super::dto::FileHeaderDescriptor;
use super::super::super::structs::file::{EncryptedFile, FileHeader};
use super::super::super::structs::repository::{Repository, RepoHeader};
use super::super::dto::AccessToken;
use super::super::super::structs::crypto::HashedPw;
use super::super::super::error::CryptError;
use uuid::Uuid;
use std::path::PathBuf;
use std::time::{Duration, Instant};


pub struct RepositoryState {
    files: HashMap<Uuid, EncryptedFile>,
    error_files: Vec<(PathBuf, String)>,
    key: HashedPw,
    repo: Repository,
    tokens: HashMap<Uuid, AccessToken>,
}


impl RepositoryState {
    pub fn new(repo: Repository, key: HashedPw) -> Self {
        RepositoryState { key: key, repo: repo, files: HashMap::new(), error_files: Vec::new(), tokens: HashMap::new() }
    }

    pub fn generate_token(&mut self) -> Uuid {
        let token = AccessToken::new();
        let retval = token.get_id();
        self.tokens.insert(token.get_id(), token);
        retval
    }

    pub fn remove_token(&mut self, token: &Uuid) {
        match self.tokens.remove(token) {
            None => warn!("No token {} present.", token),
            Some(t) => debug!("Removed token {}", token),
        }
    }

    pub fn has_tokens(&self) -> bool {
        !self.tokens.is_empty()
    }

    pub fn check_token(&mut self, token: &Uuid) -> bool {
        let mut o = self.tokens.get_mut(token);
        match o {
            None => false,
            Some(ref mut t) => {
                let min = t.get_elapsed_minutes();
                if min > 20 {
                    false
                } else {
                    t.touch();
                    true
                }
            }
        }
    }

    pub fn get_file(&self, id: &Uuid) -> Option<&EncryptedFile> {
        self.files.get(id)
    }
    pub fn get_file_mut(&mut self, id: &Uuid) -> Option<&mut EncryptedFile> {
        self.files.get_mut(id)
    }

    pub fn update_file(&mut self, header: FileHeader, path: PathBuf) -> Result<(), CryptError> {
        let file = EncryptedFile::load_head(&header, &self.key, &path)?;
        let existing_version = self.files.get(&header.get_id()).map_or(0, |f| f.get_version());
        if existing_version <= header.get_version() {
            self.files.insert(header.get_id(), file);
            Ok(())
        } else {
            Err(CryptError::OptimisticLockError(existing_version))
        }
    }


    pub fn get_repo(&self) -> &Repository {
        &self.repo
    }

    pub fn get_key(&self) -> &HashedPw {
        &self.key
    }

    pub fn add_file(&mut self, file: EncryptedFile) {
        self.files.insert(file.get_id(), file);
    }

    pub fn add_error(&mut self, tuple: (PathBuf, String)) {
        self.error_files.push(tuple);
    }

    pub fn get_file_headers(&self) -> Vec<FileHeaderDescriptor> {
        self.files.values().map(|f| FileHeaderDescriptor::new(f)).collect()
    }

    pub fn get_files(&self) -> &HashMap<Uuid, EncryptedFile> {
        &self.files
    }

    pub fn get_files_mut(&mut self) -> &mut HashMap<Uuid, EncryptedFile> {
        &mut self.files
    }

    pub fn remove_file(&mut self, id: &Uuid) {
        self.files.remove(id);
    }

    pub fn get_name(&self) -> String {
        self.repo.get_name()
    }
    #[cfg(test)]
    pub fn set_token_time(&mut self, token: &Uuid, time: Instant) {
        let mut t = self.tokens.get_mut(token).unwrap();
        t.last_access = time;
    }

    #[cfg(test)]
    pub fn get_token_time(&self, token: &Uuid) -> Instant {
        let t = self.tokens.get(token).unwrap();
        t.last_access
    }
}
