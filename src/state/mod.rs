// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crypt::{CryptoActor, CryptoSender, CryptError, AccessToken};
use std::path::PathBuf;
use uuid::Uuid;
use search::SearchCache;
use crypt::CryptoIfc;

pub struct GlobalState {
    crypt_actor: CryptoActor,
    search_cache: SearchCache,
}

impl GlobalState {
    pub fn new(folders: Vec<PathBuf>) -> Result<Self, CryptError> {
        let crypt = CryptoActor::new(folders)?;
        let sender = crypt.create_sender();
        //        let cache = SearchCache::new(&crypt);
        Ok(GlobalState { crypt_actor: crypt, search_cache: SearchCache { crypt_sender: sender } })
    }

    pub fn crypt(&self) -> &CryptoActor {
        &self.crypt_actor
    }
    pub fn search_cache(&self) -> &SearchCache{
        &self.search_cache
    }

    pub fn check_token(&self, repo: &Uuid, token: &AccessToken) -> bool {
        self.crypt_actor.check_token(repo, token)
    }
}