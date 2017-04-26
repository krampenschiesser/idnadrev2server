
struct RepositoryState {
    files: HashMap<Uuid, EncryptedFile>,
    error_files: Vec<(PathBuf, String)>,
    key: HashedPw,
    repo: Repository,
    tokens: HashMap<Uuid, AccessToken>,
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

    fn remove_token(&mut self, token: &Uuid) {
        match self.tokens.remove(token) {
            None => warn!("No token {} present.", token),
            Some(t) => debug!("Removed token {}", token),
        }
    }

    fn has_tokens(&self) -> bool {
        !self.tokens.is_empty()
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

    pub fn get_file(&self, id: &Uuid) -> Option<&EncryptedFile> {
        self.files.get(id)
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
