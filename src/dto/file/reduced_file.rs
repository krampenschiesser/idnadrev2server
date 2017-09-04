use chrono::{DateTime, Utc};
use serde_json;

use super::file::File;
use super::descriptor::FileHeaderDescriptor;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ReducedFile {
    pub name: String,

    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub deleted: Option<DateTime<Utc>>,

    pub file_type: String,
    pub tags: Vec<String>,
    pub details: Option<serde_json::Value>,
}

impl ReducedFile {
    pub fn new(file: &File) -> Self {
        ReducedFile {
            name: file.name.clone(),

            created: file.created,
            updated: file.updated,
            deleted: file.deleted,

            file_type: file.file_type.clone(),
            tags: file.tags.clone(),
            details: file.details.clone(),
        }
    }

    pub fn from_descriptor(desc: &FileHeaderDescriptor) -> Result<Self, String> {
        use serde_json::from_str;

        let file = match from_str(desc.header.as_str()) {
            Ok(obj) => obj,
            Err(e) => return Err(format!("{}", e))
        };
        Ok(file)
    }
}
