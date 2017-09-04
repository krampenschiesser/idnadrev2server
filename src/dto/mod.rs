// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
mod enc_types;
mod file;
mod repository;
mod search;

pub use self::enc_types::{PlainPw, EncryptionType, PasswordHashType};
pub use self::file::{FileId, File, FileDescriptor, FileHeaderDescriptor};
pub use self::repository::{RepoId, RepositoryDescriptor, RepositoryDto, CreateRepository, AccessToken, OpenRepository};
pub use self::search::{Page, Synchronization, SynchronizationFileDescriptor};

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

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