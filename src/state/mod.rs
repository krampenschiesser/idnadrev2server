// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crypt::{CryptoActor,CryptError,AccessToken};
use std::path::PathBuf;
use uuid::Uuid;

pub struct GlobalState {
    crypt_actor: CryptoActor,
}

impl GlobalState {
    pub fn new(folders: Vec<PathBuf>) -> Result<Self,CryptError> {
        let crypt = CryptoActor::new(folders)?;
        Ok(GlobalState { crypt_actor: crypt })
    }

    pub fn crypt(&self) -> &CryptoActor {
        &self.crypt_actor
    }

    pub fn check_token(&self, repo: &Uuid, token: &AccessToken) -> bool {
        self.crypt_actor.check_token(repo,token)
    }
}