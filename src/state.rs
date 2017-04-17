use repository::{Repository, RepositoryFile};
use uuid::Uuid;
use std::collections::HashMap;

pub struct RepositoryState {
    pub repositories: HashMap<Uuid, Repository>,
}


#[derive(Serialize, Deserialize)]
pub struct RepoNamesDTO {
    id: Uuid,
    name: String,
}

impl RepositoryState {
    pub fn new() -> Self {
        RepositoryState { repositories: HashMap::new() }
    }

    pub fn add_repos(&mut self, repositories: Vec<Repository>) {
        for repository in repositories {
            self.repositories.insert(repository.id, repository);
        }
    }

    pub fn add_repo(&mut self, repository: Repository) {
        self.repositories.insert(repository.id, repository);
    }

    pub fn get_repo_names(&self) -> Vec<RepoNamesDTO> {
        let v = self.repositories.values();
        v.map(|ref v| RepoNamesDTO { id: v.id, name: v.name.clone() }).collect()
    }

    pub fn get_repository(&self, id: &Uuid) -> Option<&Repository> {
        self.repositories.get(id)
    }
}