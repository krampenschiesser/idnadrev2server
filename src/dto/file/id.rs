use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json;
use crypt::{FileHeader, EncryptedFile};
use crypt::{RepoHeader, Repository};
use std::time::Instant;
use std::fmt::{Display, Formatter};
use std::fmt;

use rest_in_rust::*;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Hash, PartialOrd, Eq)]
pub struct FileId(Uuid);

impl Display for FileId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.simple())
    }
}

impl FromRequest for FileId {
    fn from_req(req: &mut Request) -> Result<Self, HttpError> {
        use std::str::FromStr;

        let res = match req.param("file_id") {
            Some(id) => Ok(id),
            None => Err(HttpError::bad_request("Missing route parameter 'repo' id"))
        }?;

        let res = Uuid::from_str(res);
        match res {
            Ok(id) => Ok(FileId(id)),
            Err(_) => Err(HttpError::bad_request("Could not parse give repo id"))
        }
    }
}

impl From<Uuid> for FileId {
    fn from(id: Uuid) -> Self {
        let r = FileId(id);
        r
    }
}

impl AsRef<Uuid> for FileId {
    fn as_ref(&self) -> &Uuid {
        &self.0
    }
}
