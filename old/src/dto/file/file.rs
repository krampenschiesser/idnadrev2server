use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json;
use std::fmt::{Display};
use std::fmt;

use rest_in_rust::*;

use dto::RepoId;
use super::FileId;
use super::FileHeaderDescriptor;
use super::reduced_file::ReducedFile;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct File {
    pub repository: RepoId,
    pub id: FileId,
    pub version: u32,
    pub name: String,

    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub deleted: Option<DateTime<Utc>>,

    pub file_type: String,
    pub tags: Vec<String>,
    pub details: Option<serde_json::Value>,

    pub content: Option<Vec<u8>>,

}
impl File {
    pub fn from_descriptor(desc: &FileHeaderDescriptor) -> Result<Self, String> {
        let reduced = ReducedFile::from_descriptor(desc)?;
        let f = File {
            repository: desc.descriptor.repo,
            id: desc.descriptor.id,
            version: desc.descriptor.version,
            name: reduced.name,

            created: reduced.created,
            updated: reduced.updated,
            deleted: reduced.deleted,

            file_type: reduced.file_type,
            tags: reduced.tags,
            details: reduced.details,
            content: None
        };
        Ok(f)
    }

    pub fn new(repo: &RepoId, name: &str, file_type: &str, content: Option<Vec<u8>>) -> Self {
        let now = Utc::now();
        File {
            repository: repo.clone(),
            id: FileId::from(Uuid::new_v4()),
            version: 0,
            name: name.to_string(),

            created: now,
            updated: now,
            deleted: None,

            file_type: file_type.to_string(),
            tags: Vec::new(),
            details: None,
            content: content
        }
    }

    pub fn to_json(&self) -> Result<String, ::serde_json::error::Error> {
        use serde_json::to_string;

        let reduced = ReducedFile::new(self);
        to_string(&reduced)
    }

    pub fn split_header_content(self) -> (Option<Vec<u8>>, Result<String, ::serde_json::error::Error>) {
        let result = self.to_json();
        let o = self.content;
        (o, result)
    }
}
impl Display for File {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let del = if self.deleted.is_some() { " deleted," } else { "" };
        let tags = self.tags.join(", ");
        write!(f, "File {} [name='{}', tags='{}',{} id={}]", self.file_type, self.name, tags, del, self.id)
    }
}

impl FromRequest for File {
    fn from_req(req: &mut Request) -> Result<Self, HttpError> {
        req.body().to_json()
    }
}
