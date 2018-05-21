use uuid::Uuid;
use chrono::{DateTime, Utc};

use dto::File;
use dto::RepoId;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Page {
    pub files: Vec<File>,
    pub total: Option<u32>,
    pub offset: u32,
    pub limit: u32,
    pub next: Option<String>,
    pub previous: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SynchronizationFileDescriptor {
    pub id: Uuid,
    pub version: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct Synchronization {
    pub repository: RepoId,
    pub files: Vec<SynchronizationFileDescriptor>,

    pub modification_start: DateTime<Utc>,
    pub modification_end: Option<DateTime<Utc>>,

    pub hash_matches: bool,
}

impl Page {
    pub fn empty() -> Self {
        Page {
            limit: 0,
            files: Vec::new(),
            next: None,
            previous: None,
            offset: 0,
            total: None,
        }
    }
}

