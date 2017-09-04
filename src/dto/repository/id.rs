use uuid::Uuid;
use std::fmt::{Display};
use std::fmt;

use rest_in_rust::*;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Hash, PartialOrd, Eq)]
pub struct RepoId(Uuid);

impl RepoId {
    pub fn to_uuid(&self) -> Uuid {
        self.0.clone()
    }
}

impl Display for RepoId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.simple())
    }
}

impl FromRequest for RepoId {
    fn from_req(req: &mut Request) -> Result<Self, HttpError> {
        use std::str::FromStr;

        let res = match req.param("repo_id") {
            Some(id) => Ok(id),
            None => Err(HttpError::bad_request("Missing route parameter 'repo' id"))
        }?;

        let res = Uuid::from_str(res);
        match res {
            Ok(id) => Ok(RepoId(id)),
            Err(_) => Err(HttpError::bad_request("Could not parse give repo id"))
        }
    }
}

impl From<Uuid> for RepoId {
    fn from(id: Uuid) -> Self {
        let r = RepoId(id);
        r
    }
}

impl AsRef<Uuid> for RepoId {
    fn as_ref(&self) -> &Uuid {
        &self.0
    }
}

impl RepoId {
    pub fn new(id: Uuid) -> Self {
        RepoId(id)
    }
}