// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::io;
use std::path::{PathBuf};
use rocket::response::NamedFile;
use rocket::State;

use super::super::UiDir;

#[get("/")]
pub fn index(ui_dir: State<UiDir>) -> io::Result<NamedFile> {
    info!("{:?}", ui_dir);
    NamedFile::open(ui_dir.0.join("index.html"))
}

#[get("/static/<file..>")]
pub fn files(file: PathBuf, ui_dir: State<UiDir>) -> Option<NamedFile> {
    NamedFile::open(ui_dir.0.join("static").join(file)).ok()
}

#[get("/<any..>", rank=10)]
pub fn any(any: PathBuf, ui_dir: State<UiDir>) -> io::Result<NamedFile> {
    NamedFile::open(ui_dir.0.join("index.html"))
}

