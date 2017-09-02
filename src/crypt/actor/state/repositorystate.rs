// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::collections::{HashMap};
use std::collections::hash_map::Values;
use dto::FileHeaderDescriptor;
use super::super::super::structs::file::{EncryptedFile, FileHeader};
use super::super::super::structs::repository::{Repository, RepoHeader};
use super::super::super::structs::crypto::HashedPw;
use super::super::super::error::CryptError;
use dto::{FileId,RepoId,AccessToken};
use uuid::Uuid;
use std::path::PathBuf;
use std::time::{Duration, Instant};


#[derive(Debug, PartialEq, Eq, Clone)]
pub struct AccessTokenState {
    #[cfg(test)]
    pub last_access: Instant,
    #[cfg(not(test))]
    last_access: Instant,

    token: AccessToken,
}


pub struct RepositoryState {
    files: HashMap<FileId, EncryptedFile>,
    error_files: Vec<(PathBuf, String)>,
    key: HashedPw,
    repo: Repository,
    tokens: HashMap<AccessToken, AccessTokenState>,
}

impl AccessTokenState {
    pub fn new() -> Self {
        let token = AccessToken::new();
        AccessTokenState { token: token, last_access: Instant::now() }
    }

    pub fn touch(&mut self) {
        self.last_access = Instant::now();
    }

    pub fn get_id(&self) -> Uuid {
        self.token.id.clone()
    }

    pub fn get_token(&self) -> AccessToken {
        self.token.clone()
    }

    pub fn get_elapsed_minutes(&self) -> u64 {
        let secs = self.last_access.elapsed().as_secs();
        secs / 60
    }

    pub fn get_elapsed(&self) -> Duration {
        self.last_access.elapsed()
    }
}

impl RepositoryState {
    pub fn new(repo: Repository, key: HashedPw) -> Self {
        RepositoryState { key: key, repo: repo, files: HashMap::new(), error_files: Vec::new(), tokens: HashMap::new() }
    }

    pub fn generate_token(&mut self) -> AccessToken {
        let token = AccessTokenState::new();
        let retval = token.get_token();
        self.tokens.insert(token.get_token(), token);
        retval
    }

    pub fn remove_token(&mut self, token: &AccessToken) {
        match self.tokens.remove(&token) {
            None => warn!("No token {} present.", token),
            Some(t) => debug!("Removed token {}", token),
        }
    }

    pub fn has_tokens(&self) -> bool {
        !self.tokens.is_empty()
    }

    pub fn check_token(&mut self, token: &AccessToken) -> bool {
        let cloned = self.tokens.clone();
        let mut o = self.tokens.get_mut(&token);
        match o {
            None => {
                debug!("Token not found {}.", &token.id);
                false
            }
            Some(ref mut t) => {
                let min = t.get_elapsed_minutes();
                if min > 20 {
                    debug!("Token timed out: {}", &token.id);
                    false
                } else {
                    t.touch();
                    true
                }
            }
        }
    }

    pub fn get_file(&self, id: &FileId) -> Option<&EncryptedFile> {
        self.files.get(id)
    }
    pub fn get_file_mut(&mut self, id: &FileId) -> Option<&mut EncryptedFile> {
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

    pub fn get_files(&self) -> &HashMap<FileId, EncryptedFile> {
        &self.files
    }

    pub fn get_files_mut(&mut self) -> &mut HashMap<FileId, EncryptedFile> {
        &mut self.files
    }

    pub fn remove_file(&mut self, id: &FileId) {
        self.files.remove(id);
    }

    pub fn get_name(&self) -> String {
        self.repo.get_name()
    }
    #[cfg(test)]
    pub fn set_token_time(&mut self, token: &AccessToken, time: Instant) {
        let mut t = self.tokens.get_mut(&token.id).unwrap();
        t.last_access = time;
    }

    #[cfg(test)]
    pub fn get_token_time(&self, token: &AccessToken) -> Instant {
        let t = self.tokens.get(&token.id).unwrap();
        t.last_access
    }
}
