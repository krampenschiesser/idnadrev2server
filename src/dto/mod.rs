// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json;
use crypt::{FileHeader, EncryptedFile};
use crypt::{RepoHeader, Repository};
use std::time::Instant;


use std::fmt::{Display, Formatter};
use std::fmt;

pub struct RepoId(Uuid);

pub struct FileId(Uuid);

#[derive(Debug, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
pub struct AccessToken {
    pub id: Uuid,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub enum EncryptionType {
    None,
    RingChachaPoly1305,
    RingAESGCM,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum PasswordHashType {
    None,
    Argon2i { iterations: u16, memory_costs: u16, parallelism: u16 },
    SCrypt { iterations: u8, memory_costs: u32, parallelism: u32 },
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct RepositoryDto {
    pub id: Uuid,
    pub token: AccessToken,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct FileDescriptor {
    pub repo: Uuid,
    pub id: Uuid,
    pub version: u32,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize)]
pub struct FileHeaderDescriptor {
    pub descriptor: FileDescriptor,
    pub header: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RepositoryDescriptor {
    pub id: Uuid,
    pub name: String,
}

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
    pub repository: Uuid,
    pub files: Vec<SynchronizationFileDescriptor>,

    pub modification_start: DateTime<Utc>,
    pub modification_end: Option<DateTime<Utc>>,

    pub hash_matches: bool,
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OpenRepository {
    ///ID of the repository to open
    //    pub id: Uuid,
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

    pub created: DateTime<Utc>,
    pub updated: DateTime<Utc>,
    pub deleted: Option<DateTime<Utc>>,

    pub file_type: String,
    pub tags: Vec<String>,
    pub details: Option<serde_json::Value>,

    pub content: Option<Vec<u8>>,

}

impl Display for RepositoryDescriptor {
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

#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ReducedFile {
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

    pub fn new(repo: &Uuid, name: &str, file_type: &str, content: Option<Vec<u8>>) -> Self {
        let now = Utc::now();
        File {
            repository: repo.clone(),
            id: Uuid::new_v4(),
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

use iron::{Request, IronResult, IronError, status};
use ironext::{FromReq, StringError};

impl FromReq<AccessToken> for AccessToken {
    fn from_req(req: &Request) -> IronResult<Self> {
        if let Some(token_str_bytes) = req.headers.get_raw("token") {}

        match req.headers.get_raw("token") {
            Some(token_str_bytes) => {
                let token_str = ::std::str::from_utf8(&token_str_bytes[0]);
                match token_str {
                    Ok(str) => {
                        match Uuid::parse_str(str) {
                            Ok(id) => Ok(AccessToken { id }),
                            Err(_) => Err(IronError::new(StringError::new(format!("Could not parse Uuid {}", str)), status::BadRequest))
                        }
                    }
                    Err(e) => Err(IronError::new(StringError::new("Invalid utf8 string in token detected"), status::BadRequest))
                }
            }
            None => Err(IronError::new(StringError::new("No token set in header"), status::BadRequest))
        }
    }
}

impl AccessToken {
    pub fn new() -> Self {
        AccessToken { id: Uuid::new_v4() }
    }
}

impl Display for AccessToken {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "Token {}", self.id.simple())
    }
}

impl FileDescriptor {
    pub fn new(header: &FileHeader) -> Self {
        FileDescriptor { repo: header.get_repository_id(), id: header.get_id(), version: header.get_version() }
    }
}

impl FileHeaderDescriptor {
    pub fn new(enc_file: &EncryptedFile) -> Self {
        let ref h = enc_file.get_encryption_header();
        let descriptor = FileDescriptor { repo: h.get_repository_id(), id: h.get_id(), version: h.get_version() };
        FileHeaderDescriptor { header: enc_file.get_header().clone(), descriptor: descriptor }
    }
}

impl RepositoryDescriptor {
    pub fn new(repo: &Repository) -> Self {
        RepositoryDescriptor { id: repo.get_id(), name: repo.get_name().clone() }
    }
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

impl FromReq<RepoId> for RepoId {
    fn from_req(req: &Request) -> IronResult<Self> {
        use router::Router;
        use std::str::FromStr;

        if let Some(router) = req.extensions.get::<Router>() {
            let id_str = router.find("repo_id");
            let res  = match id_str {
                Some(id) => Ok(id),
                None => Err(IronError::new(StringError::new("Missing route parameter 'repo' id"), status::BadRequest))
            }?;

            let res = Uuid::from_str(res);
            match res {
                Ok(id) =>Ok(RepoId(id)),
                Err(_) => Err(IronError::new(StringError::new("Could not parse give repo id"), status::BadRequest))
            }

        } else {
            Err(IronError::new(StringError::new("Iron router not found"), status::InternalServerError))
        }
    }
}

impl From<Uuid> for RepoId {
    fn from(id: Uuid) -> Self {
        let r = RepoId(id);
        r
    }
}

impl From<Uuid> for FileId {
    fn from(id: Uuid) -> Self {
        let r = FileId(id);
        r
    }
}

impl AsRef<Uuid> for RepoId {
    fn as_ref(&self) -> &Uuid {
        &self.0
    }
}

impl AsRef<Uuid> for FileId {
    fn as_ref(&self) -> &Uuid {
        &self.0
    }
}

impl FromReq<CreateRepository> for CreateRepository {
    fn from_req(req: &Request) -> IronResult<CreateRepository> {
        get_json_body(req)
    }
}

impl FromReq<OpenRepository> for OpenRepository {
    fn from_req(req: &Request) -> IronResult<OpenRepository> {
        get_json_body(req)
    }
}

impl FromReq<File> for File {
    fn from_req(req: &Request) -> IronResult<File> {
        get_json_body(req)
    }
}

fn get_json_body<T>(req: &Request) -> IronResult<T>
where T: ::serde::de::DeserializeOwned{
    use std::io::Read;
    use serde_json::from_str;
    use serde_json::Error;

    let mut s = String::new();
    req.body.read_to_string(&mut s);

    let b: Result<T, Error> = from_str(s.as_str());
    match b {
        Ok(cmd) => Ok(cmd),
        Err(_) => Err(IronError::new(StringError::new("Could not parse input as cmd"), status::BadRequest))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_eq() {
        use std::str::FromStr;

        let uuid = Uuid::from_str("1074e93b-e8e7-465e-9fb1-54da4e5c136b").unwrap();
        let token1 = AccessToken { id: uuid };

        let uuid = Uuid::from_str("1074e93b-e8e7-465e-9fb1-54da4e5c136b").unwrap();
        let token2 = AccessToken { id: uuid };

        assert_eq!(token1, token2);
    }
}