
pub mod repositorystate;
pub mod scanresult;


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
    fn get_repositories(&self) -> &Vec<Repository> {
        self.scan_result.get_repositories()
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
            None => {
                info!("No repository found for id {}", id);
                false
            }
        }
    }

    fn generate_token(&mut self, id: &Uuid) -> Option<Uuid> {
        let mut o = self.repositories.get_mut(id);
        match o {
            None => None,
            Some(ref mut r) => Some(r.generate_token())
        }
    }

    fn remove_token(&mut self, id: &Uuid, token: &Uuid) {
        let no_tokens = match self.repositories.get_mut(id) {
            None => false,
            Some(ref mut r) => {
                r.remove_token(token);
                !r.has_tokens()
            }
        };
        if no_tokens {
            info!("All tokens removed, now closing repository {} with id {}", self.get_repository(id).unwrap().repo.name, id);
            self.repositories.remove(id);
        }
    }

    fn update_file(&mut self, file_header: FileHeader, path: PathBuf) -> Result<(), String> {
        let file_id = file_header.get_id();
        let added = self.scan_result.update_file(&file_header, &path);
        let repo_id = file_header.get_repository_id();

        match self.repositories.get_mut(&repo_id) {
            Some(ref mut repo) => {
                let repo_enc_type = repo.repo.header.encryption_type.clone();
                let file_enc_type = file_header.encryption_type.clone();
                if repo_enc_type != file_enc_type {
                    Err(format!("Cannot add file with different encryption type. Repository: {}, file: {}", repo_enc_type, file_enc_type))
                } else {
                    repo.update_file(file_header.clone(), path);
                    Ok(())
                }
            }
            None => {
                Err(format!("Found no repository for {}", repo_id))
            }
        }
    }
}