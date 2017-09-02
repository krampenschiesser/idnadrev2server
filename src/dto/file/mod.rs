use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde_json;
use crypt::{FileHeader, EncryptedFile};
use crypt::{RepoHeader, Repository};
use std::time::Instant;
use std::fmt::{Display, Formatter};
use std::fmt;

use rest_in_rust::*;

mod descriptor;
mod file;
mod id;
mod reduced_file;

pub use self::file::File;
pub use self::id::FileId;
pub use self::descriptor::{FileHeaderDescriptor, FileDescriptor};





