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
use std::io::Cursor;
use state::UiState;
use rest_in_rust::*;

//#[get("/")]
pub fn index(req: &mut Request) -> Result<Response, HttpError> {
    read_single_file("index.html", req)
}

fn read_file_string(path: &PathBuf) -> Result<String, ::std::io::Error> {
    use std::fs::File;
    use std::io::{ErrorKind, Error, Read};

    let mut file = File::open(path)?;
    let mut val = String::new();
    match file.read_to_string(&mut val) {
        Ok(_) => Ok(val),
        Err(e) => Err(Error::new(ErrorKind::Other, "Could not read file"))
    }
}

pub fn manifest(req: &mut Request) -> Result<Response, HttpError> {
    use ::http::status;
    use ::http::header;
    let ui_state = UiState::from_req_as_ref(req)?;

    let hash = ui_state.compute_hash();
    let status = if hash.is_ok() { status::OK.into() } else { status::INTERNAL_SERVER_ERROR };

    let body = if hash.is_ok() { hash.unwrap() } else { format!("{}", hash.err().unwrap()) };

    let r = Response::builder()
        .status(status)
        .header_str_value(header::CONTENT_TYPE, "Text/CacheManifest")?
        .body(body)
        .build()?;
    Ok(r)
}

//#[get("/static/<file..>", rank = 9)]
pub fn files(req: &mut Request) -> Result<Response, HttpError> {
    let file: String = {
        req.param("file_name").unwrap_or("/")
    }.into();
    let ui_state = UiState::from_req_as_ref(req)?;

    let path = ui_state.ui_dir.join("static").join(file);
    let result = read_file_string(&path)?;
    Ok(result.into())
}

fn read_single_file(name: &str, req: &mut Request) -> Result<Response, HttpError> {
    let ui_state = UiState::from_req_as_ref(req)?;
    let path = ui_state.ui_dir.join(name);
    let result = read_file_string(&path)?;
    Ok(result.into())
}

//#[get("/asset-manifest.json")]
pub fn asset_mainfest(req: &mut Request) -> Result<Response, HttpError> {
    read_single_file("asset-manifest.json", req)
}

//#[get("/favicon.ico")]
pub fn favicon(req: &mut Request) -> Result<Response, HttpError> {
    read_single_file("favicon.ico", req)
}

//#[get("/index.html")]
pub fn index_html(req: &mut Request) -> Result<Response, HttpError> {
    read_single_file("index.html", req)
}

//pub fn any(any: PathBuf, ui_dir: State<UiDir>) -> Option<NamedFile> {
//    NamedFile::open(ui_dir.0.join("index.html")).ok()
//}
pub fn any(_: &mut Request) -> Result<Response, HttpError> {
    let response = Response::moved_permanent("/")?;
    Ok(response)
}
