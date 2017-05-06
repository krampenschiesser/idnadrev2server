// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

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
    pub id: Uuid,
    pub name: String,
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

#[derive(Serialize, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ReducedFile<'a> {
    pub name: &'a String,

    pub created: &'a DateTime<UTC>,
    pub updated: &'a DateTime<UTC>,
    pub deleted: &'a Option<DateTime<UTC>>,

    pub file_type: &'a String,
    pub tags: &'a Vec<String>,
    pub details: &'a Option<serde_json::Value>,
}

impl<'a> ReducedFile<'a> {
    pub fn new(file: &'a File) -> Self {
        ReducedFile {
            name: &file.name,

            created: &file.created,
            updated: &file.updated,
            deleted: &file.deleted,

            file_type: &file.file_type,
            tags: &file.tags,
            details: &file.details,
        }
    }
}

impl File {
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

use ::rocket::Request;
use ::rocket::request::Outcome as Return;
use ::rocket::Outcome;
use ::rocket::http::Status;
use super::super::crypt::AccessToken;

impl<'a, 'r> ::rocket::request::FromRequest<'a, 'r> for AccessToken {
    type Error = String;

    fn from_request(request: &'a Request<'r>) -> Return<Self, Self::Error> {
        let mut token = request.headers().get("token");
        if let Some(token) = token.next() {
            let res = Uuid::parse_str(token);
            match res {
                Ok(uid) => Outcome::Success(AccessToken { id: uid }),
                Err(e) => Outcome::Failure((Status::BadRequest, format!("{}", e)))
            }
        } else {
            Outcome::Failure((Status::Unauthorized, "No token given".to_string()))
        }
    }
}

impl Page {
    pub fn empty() -> Self {
        Page {
            offset: 0,
            files: Vec::new(),
            next: None,
            previous: None,
            start: 0,
            total: None,
        }
    }
}