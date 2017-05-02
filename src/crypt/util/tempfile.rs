// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::path::PathBuf;
use std::fs::remove_file;
use uuid::Uuid;
use std::ops::Drop;
use std;
use super::io::path_to_str;

pub struct TempFile {
    pub path: PathBuf,
    pub moved: bool,
}

impl TempFile {
    pub fn new() -> Self {
        let tempdir = std::env::temp_dir();
        let name = format!("{}", Uuid::new_v4().simple());
        TempFile::new_in_path(tempdir.join(name))
    }

    pub fn new_in_path(path: PathBuf) -> Self {
        TempFile { path: path, moved: false }
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        if !self.moved {
            match remove_file(self.path.clone()) {
                Err(d) => error!("Could not close temp file {}: {}", path_to_str(&self.path), d),
                _ => (),
            }
        }
    }
}