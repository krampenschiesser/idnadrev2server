use crypt::Repository;
use std::fmt::{Display};
use std::fmt;

use super::id::RepoId;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryDescriptor {
    pub id: RepoId,
    pub name: String,
}

impl Display for RepositoryDescriptor {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Repository [name='{}', id={}]", self.name, self.id, )
    }
}

impl RepositoryDescriptor {
    pub fn new(repo: &Repository) -> Self {
        RepositoryDescriptor { id: repo.get_id(), name: repo.get_name().clone() }
    }
}
