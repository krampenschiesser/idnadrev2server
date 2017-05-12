// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::io;
use std::path::{PathBuf, Path};
use rocket::response::NamedFile;
use rocket::{Response, State};
use rocket::response::Redirect;
use rocket::http::{Status, ContentType};
use std::io::Cursor;

use state::UiState;

#[get("/")]
pub fn index(state: State<UiState>) -> io::Result<NamedFile> {
    info!("{:?}", state.ui_dir);
    NamedFile::open(state.ui_dir.join("index.html"))
}

#[get("/manifest.appcache")]
pub fn manifest(state: State<UiState>) -> Response {
    let hash = state.compute_hash();
    let status = if hash.is_ok() { Status::Ok } else { Status::InternalServerError };

    let body = if hash.is_ok() { hash.unwrap() } else { format!("{}", hash.err().unwrap()) };

    Response::build()
        .sized_body(Cursor::new(body))
        .raw_header("Content-Type", "text/cache-manifest")
        .status(status)
        .finalize()
}

#[get("/static/<file..>", rank = 9)]
pub fn files(file: PathBuf, state: State<UiState>) -> Option<NamedFile> {
    NamedFile::open(state.ui_dir.join("static").join(file)).ok()
}

#[get("/asset-manifest.json")]
pub fn asset_mainfest(state: State<UiState>) -> Option<NamedFile> {
    NamedFile::open(state.ui_dir.join("asset-manifest.json")).ok()
}

#[get("/favicon.ico")]
pub fn favicon(state: State<UiState>) -> Option<NamedFile> {
    NamedFile::open(state.ui_dir.join("favicon.ico")).ok()
}

#[get("/index.html")]
pub fn index_html(state: State<UiState>) -> Option<NamedFile> {
    NamedFile::open(state.ui_dir.join("index.html")).ok()
}

#[get("/<any..>", rank = 10)]
//pub fn any(any: PathBuf, ui_dir: State<UiDir>) -> Option<NamedFile> {
//    NamedFile::open(ui_dir.0.join("index.html")).ok()
//}
pub fn any(any: PathBuf, state: State<UiState>) -> Redirect {
    Redirect::to("/")
}
