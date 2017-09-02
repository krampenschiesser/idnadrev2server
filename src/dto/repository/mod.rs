use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json;
use crypt::{FileHeader, EncryptedFile};
use crypt::{RepoHeader, Repository};
use std::time::Instant;
use std::fmt::{Display, Formatter};
use std::fmt;

use rest_in_rust::*;

mod create_repo;
mod descriptor;
mod id;
mod open_repo;
mod token;

pub use self::id::RepoId;
pub use self::create_repo::CreateRepository;
pub use self::descriptor::RepositoryDescriptor;
pub use self::open_repo::OpenRepository;
pub use self::token::AccessToken;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct RepositoryDto {
    pub id: RepoId,
    pub token: AccessToken,
}

