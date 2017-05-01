use crypt::{CryptoActor,CryptError};
use std::path::PathBuf;

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
}