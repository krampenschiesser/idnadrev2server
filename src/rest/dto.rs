//! # Interface objects
//!
//!
//!
//!
//!

use uuid::Uuid;
use chrono::{DateTime, UTC};
use std::fmt::Display;
use std::fmt;
use serde_json;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Repository {
    pub name: String,
    pub id: Uuid
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum EncryptionType {
    ChaCha,
    AES,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    files: Vec<File>,
    total: Option<u32>,
    start: u32,
    offset: u32,
    next: Option<String>,
    previous: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateRepository {
    ///Name of the repository, must be unique
    pub name: String,
    ///Encryption type of the repository, will be used for all files in it
    pub encryption: EncryptionType,
    ///Password bytes
    pub password: Vec<u8>,
    ///User name, currently unused, use whatever you want
    pub user_name: String,
}

///
/// Struct to
///
///
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OpenRepository{
    ///ID of the repository to open
    pub id: Uuid,
    ///Password to use for open
    pub password: Vec<u8>,
    ///Username to use for open
    pub user_name: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct File {
    pub repository: Uuid,
    pub id: Uuid,
    pub version: u32,
    pub name: String,

    pub created: DateTime<UTC>,
    pub updated: DateTime<UTC>,
    pub deleted: Option<DateTime<UTC>>,

    pub file_type: String,
    pub tags: Vec<String>,
    pub details: Option<serde_json::Value>,

    pub content: Option<Vec<u8>>,
}

impl Display for Repository {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Repository [name='{}', id={}]", self.name, self.id, )
    }
}

impl Display for File {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let del = if self.deleted.is_some() { " deleted," } else { "" };
        let tags = self.tags.join(", ");
        write!(f, "File {} [name='{}', tags='{}',{} id={}]", self.file_type, self.name, tags, del, self.id.simple())
    }
}
