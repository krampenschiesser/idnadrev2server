use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json;
use crypt::{FileHeader, EncryptedFile};
use crypt::{RepoHeader, Repository};
use std::time::Instant;
use std::fmt::{Display, Formatter};
use std::fmt;

use rest_in_rust::*;

#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
pub struct AccessToken {
    pub id: Uuid,
}

impl Display for AccessToken{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.id.simple())
    }
}


impl From<Uuid> for AccessToken {
    fn from(id: Uuid) -> Self {
        let r = AccessToken{id};
        r
    }
}

impl AsRef<Uuid> for AccessToken{
    fn as_ref(&self) -> &Uuid {
        &self.id
    }
}

impl FromRequest for AccessToken {
    fn from_req(req: &mut Request) -> Result<Self, HttpError> {
        match req.header_str("token") {
            Some(token_str) => {
                match Uuid::parse_str(token_str) {
                    Ok(id) => Ok(AccessToken { id }),
                    Err(_) => Err(HttpError::bad_request(format!("Could not parse Uuid {}", token_str)))
                }
            }
            None => Err(HttpError::bad_request("No token set in header"))
        }
    }
}

impl AccessToken {
    pub fn new() -> Self {
        AccessToken { id: Uuid::new_v4() }
    }

    pub fn to_header(&self) -> (::http::header::HeaderName, ::http::header::HeaderValue) {
        use std::str::FromStr;
        use http::header::HeaderName;
        use http::header::HeaderValue;

        let name = HeaderName::from_str("token").unwrap();
        let string = format!("{}", self.id.simple());
        let value = HeaderValue::from_str(string.as_ref()).unwrap();
        (name, value)
    }
}