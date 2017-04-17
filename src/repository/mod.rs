#![feature(custom_attribute)]
extern crate chrono;
extern crate serde;
extern crate serde_json;

mod crypt;
pub mod service;

use uuid::Uuid;
use std::collections::HashMap;
use chrono::{DateTime, UTC};
use self::crypt::{FileHeader, RepoHeader};
use std::fmt::Display;
use std::fmt;

#[derive(Serialize, Deserialize, Debug)]
pub struct Repository {
    //    #[serde(skip_serializing,skip_deserializing)]
    //    header: RepoHeader,
    pub id: Uuid,
    pub days_to_keep: u16,
    pub name: String,
    files: HashMap<Uuid, RepositoryFile>,
    contents: HashMap<Uuid, Vec<u8>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum FileType {
    Thought,
    Task,
    Document,
    Image,
    Other,
    Text,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RepositoryFile {
    //    #[serde(skip_serializing,skip_deserializing)]
    //    header: FileHeader,
    id: Uuid,
    repository_id: Uuid,
    version: u32,
    name: String,

    created: DateTime<UTC>,
    updated: DateTime<UTC>,
    deleted: Option<DateTime<UTC>>,

    file_type: FileType,
    tags: Vec<String>,
    details: Option<serde_json::Value>,

    content: Option<Vec<u8>>,
}

impl Repository {
    pub fn with_name(name: &str) -> Self {
        Repository { id: Uuid::new_v4(), name: name.into(), files: HashMap::new(), days_to_keep: 7, contents: HashMap::new() }
    }

    pub fn add_file(&mut self, mut file: RepositoryFile) -> &mut Self {
        if file.content.is_some() {
            let content: Vec<u8> = file.content.unwrap();
            self.contents.insert(file.id, content);
        }
        file.content = None;
        self.files.insert(file.id, file);
        self
    }

    pub fn add_files(&mut self, files: Vec<RepositoryFile>) {
        for file in files {
            self.add_file(file);
        }
    }

    pub fn get_file(&self, id: &Uuid) -> Option<RepositoryFile> {
        let mut file = self.files.get(id).cloned();
        match file {
            Some(mut f) => {
                let content = self.get_file_content(id);
                match content {
                    Some(c) => {
                        f.set_content(c.into());
                        Some(f)
                    }
                    None => Some(f)
                }
            }
            None => None
        }
    }

    pub fn get_file_header(&self, id: &Uuid) -> Option<RepositoryFile> {
        self.files.get(id).cloned()
    }
    pub fn get_file_content(&self, id: &Uuid) -> Option<Vec<u8>> {
        self.contents.get(id).cloned()
    }

    pub fn get_files(&self) -> Vec<RepositoryFile> {
        self.files.values().map(|f| f.clone()).collect()
    }
}

impl RepositoryFile {
    pub fn with_name(repository_id: Uuid, name: &str) -> Self {
        let now = UTC::now();
        let id = Uuid::new_v4();
        RepositoryFile {
            repository_id: repository_id,
            id: id,
            name: name.into(),
            version: 0,
            created: now,
            updated: now,
            deleted: Option::None,
            file_type: FileType::Other,
            tags: Vec::new(),
            details: Option::None,
            content: Option::None
        }
    }

    pub fn set_name(&mut self, name: &str) -> &mut Self {
        self.name = name.into();
        self
    }

    pub fn set_version(&mut self, version: u32) -> &mut Self {
        self.version = version;
        self
    }

    pub fn update(&mut self, updated: &DateTime<UTC>) -> &mut Self {
        self.updated = updated.clone();
        self
    }

    pub fn delete(&mut self, deleted: &DateTime<UTC>) -> &mut Self {
        self.deleted = Option::Some(deleted.clone());
        self
    }

    pub fn restore(&mut self) -> &mut Self {
        self.deleted = Option::None;
        self
    }

    pub fn set_file_type(&mut self, file_type: FileType) -> &mut Self {
        self.file_type = file_type;
        self
    }

    pub fn add_tag(&mut self, tag: &str) -> &mut Self {
        self.tags.push(tag.into());
        self
    }

    pub fn set_details(&mut self, details: serde_json::Value) -> &mut Self {
        self.details = Option::Some(details);
        self
    }

    pub fn set_content_string(&mut self, content: &str) -> &mut Self {
        let v = content.as_bytes().to_vec();
        self.content = Option::Some(v);
        self
    }

    pub fn set_content(&mut self, content: Vec<u8>) -> &mut Self {
        self.content = Option::Some(content);
        self
    }
}

impl Display for RepositoryFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}]{}->{}", self.file_type, self.name, self.id)
    }
}

impl Display for Repository {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}->{}", self.name, self.id)
    }
}

impl Display for FileType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = format!("{:?}", self);
        write!(f, "{}", s.to_uppercase())
    }
}

impl FileType {
    pub fn is_text(&self) -> bool {
        match self {
            Thought => true,
            Task => true,
            Document =>true,
            Text => true,
            _ => false,
        }
    }
}