pub struct ScanResult {
    repositories: Vec<Repository>,
    files: HashMap<Uuid, (FileHeader, PathBuf)>,
    invalid: Vec<(CryptError, PathBuf)>,
    watcher: RecommendedWatcher,
    file_change_receiver: Receiver<DebouncedEvent>,
}
#[derive(Debug)]
pub enum CheckRes {
    Repo(RepoHeader, PathBuf),
    File(FileHeader, PathBuf),
    Error(CryptError, PathBuf),
}

impl CheckRes {
    fn get_path(&self) -> PathBuf {
        match *self {
            CheckRes::Repo(_, ref p) | CheckRes::File(_, ref p) | CheckRes::Error(_, ref p) => p.clone()
        }
    }
}

impl ScanResult {
    fn new(watcher: RecommendedWatcher, file_change_receiver: Receiver<DebouncedEvent>) -> Self {
        ScanResult { repositories: Vec::new(), files: HashMap::new(), invalid: Vec::new(), watcher: watcher, file_change_receiver: file_change_receiver }
    }

    pub fn get_repository(&self, id: &Uuid) -> Option<Repository> {
        let result = self.repositories.iter().find(|repo| {
            repo.get_id() == *id
        });
        match result {
            Some(repo) => Some(repo.clone()),
            None => None,
        }
    }

    pub fn get_repositories(&self) -> &Vec<Repository> {
        &self.repositories
    }

    pub fn get_files_for_repo(&self, repo_id: &Uuid) -> Vec<(FileHeader, PathBuf)> {
        self.files.values().filter(|ref t| t.0.get_repository_id() == *repo_id).map(|e| e.clone()).collect()
    }

    pub fn update_file(&mut self, header: &FileHeader, path: &PathBuf) {
        let file_id = header.get_id();
        let version = header.get_version();

        let should_insert = match self.files.get(&file_id) {
            None => true,
            Some(present) => {
                let old_version = present.0.get_version();
                if old_version < version {
                    true
                } else {
                    error!("File in scanresult is newer (v={}) than the one added on fs(v={}). Path: {}", old_version, version, path_to_str(path));
                    false
                }
            }
        };
        if should_insert {
            self.files.insert(file_id.clone(), (header.clone(), path.clone()));
        }
    }
}