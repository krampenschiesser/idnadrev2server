// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use crypt::{CryptoActor, CryptoSender, CryptError};
use std::path::{PathBuf, Path};
use uuid::Uuid;
use search::SearchCache;
use crypt::CryptoIfc;
use std::sync::RwLock;
use dto::AccessToken;

pub struct GlobalState {
    crypt_actor: CryptoActor,
    search_cache: SearchCache,

}

pub struct UiState {
    pub ui_dir: PathBuf,
    hash: RwLock<Hash>,
}

struct Hash { value: Option<String> }

impl Hash {
    fn new() -> Self {
        Hash { value: None }
    }
}

impl UiState {
    pub fn new(ui_dir: PathBuf) -> Self {
        UiState { ui_dir: ui_dir, hash: RwLock::new(Hash::new()) }
    }

    pub fn compute_hash(&self) -> ::std::io::Result<String> {
        {
            let ref o = self.hash.read().unwrap();
            if o.value.is_some() {
                let str = o.value.clone().unwrap();
                return Ok(str)
            }
        }

        let hash: ::std::io::Result<String> = hash_dir(&self.ui_dir);
        match hash {
            Ok(hash) => {
                let ref mut o = self.hash.write().unwrap();
                o.value = Some(hash.clone());
                Ok(hash)
            }
            Err(e) => Err(e)
        }
    }
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
    pub fn search_cache(&self) -> &SearchCache {
        &self.search_cache
    }

    pub fn check_token(&self, repo: &Uuid, token: &AccessToken) -> bool {
        self.crypt_actor.check_token(repo, token)
    }
}

use iron::typemap::Key;

impl Key for GlobalState {
    type Value = GlobalState;
}


fn read_dir(root: &Path) -> ::std::io::Result<Vec<PathBuf>> {
    let dir = root.read_dir()?;

    let mut v = Vec::new();

    for entry in dir {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let mut sub = read_dir(&entry.path().to_path_buf())?;
            v.append(&mut sub);
        } else {
            v.push(entry.path().to_path_buf());
        }
    }
    Ok(v)
}

fn hash(paths: &[PathBuf]) -> ::std::io::Result<String> {
    use std::fs::File;
    use std::io::Read;

    let mut sha1 = ::sha1::Sha1::new();
    for path in paths {
        let mut buf = Vec::new();
        File::open(path)?.read_to_end(&mut buf)?;
        sha1.update(buf.as_slice());
    }
    Ok(sha1.digest().to_string())
}

fn hash_dir(root: &PathBuf) -> ::std::io::Result<String> {
    let paths = read_dir(root)?;

    let hash = hash(&paths)?;
    let mut names = Vec::new();
    for path in paths {
        names.push(path_to_string(root, &path))
    }
    let mut retval = String::new();
    retval.push_str("CACHE MANIFEST\n");
    retval.push_str("#");
    retval.push_str(hash.as_str());
    retval.push_str("\nCACHE:\n");
    for name in names {
        retval.push_str(name.as_str());
        retval.push_str("\n");
    }
    retval.push_str("\nFALLBACK:\n");
    retval.push_str("/ /index.html\n");

    retval.push_str("\nNETWORK:\n*\n");

    Ok(retval)
}

fn path_to_string(root: &PathBuf, path: &PathBuf) -> String {
    let mut str = String::new();
    let prefix = root.as_os_str().to_string_lossy();

    for p in path {
        let p = p.to_string_lossy().into_owned();
        if !prefix.contains(p.as_str()) {
            str.push_str(format!("/{}", p).as_str());
        }
    }
    str
}

#[cfg(test)]
mod tests {
    use super::hash_dir;
    use tempdir::TempDir;
    use std::fs::{File, create_dir};
    use spectral::prelude::*;

    #[test]
    fn test_hash_dir() {
        let temp = TempDir::new("hashing").unwrap();
        let root = temp.path().to_path_buf();

        let subdir = root.join("subdir");
        create_dir(&subdir);

        let file1 = root.join("file1.txt");
        let file2 = subdir.join("file2.txt");

        File::create(file1);
        File::create(file2);

        let hash = hash_dir(&root).unwrap();
        println!("{}", hash);
        assert_that(&hash).contains("/subdir/file2.txt");
        assert_that(&hash).contains("/file1.txt");
    }
}