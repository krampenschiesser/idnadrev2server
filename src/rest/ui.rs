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
use iron::prelude::*;
use iron::status;
use persistent::Read;
use state::UiState;
use router::Router;

//#[get("/")]
pub fn index(req: &mut Request) -> IronResult<Response> {
    read_single_file("index.html", &mut req)
}

fn read_file_string(path: &PathBuf) -> Result<String, ::std::io::Error> {
    use std::fs::File;
    use std::io::{ErrorKind, Error};

    let file = File::open(path);
    let mut val = String::new();
    match file.read_to_string(&mut val) {
        Ok(_) => Ok(val),
        Err(e) => Err(Error::new(ErrorKind::Other, "Could not read file"))
    }
}

pub fn manifest(req: &mut Request) -> IronResult<Response> {
    use iron::headers::ContentType;

    let ui_state = req.get::<Read<UiState>>().unwrap().as_ref();

    let hash = ui_state.compute_hash();
    let status = if hash.is_ok() { status::Ok } else { status::InternalServerError };

    let body = if hash.is_ok() { hash.unwrap() } else { format!("{}", hash.err().unwrap()) };

    let mut response = Response::with((status, body));
    response.headers.set(ContentType(mime!(Text/CacheManifest)));
    Ok(response)
}

//#[get("/static/<file..>", rank = 9)]
pub fn files(req: &mut Request) -> IronResult<Response> {
    let ui_state = req.get::<Read<UiState>>().unwrap().as_ref();
    let ref file = req.extensions.get::<Router>()
        .unwrap().find("file_name").unwrap_or("/");

    let path = ui_state.ui_dir.join("static").join(file);
    let result = read_file_string(path)?;
    Ok(Response::with((status::Ok, result)))
}

fn read_single_file(name: &str, req: &mut Request) -> IronResult<Response> {
    let ui_state = req.get::<Read<UiState>>().unwrap().as_ref();
    let path = ui_state.ui_dir.join(name);
    let result = read_file_string(path)?;
    Ok(Response::with((status::Ok, result)))
}

//#[get("/asset-manifest.json")]
pub fn asset_mainfest(req: &mut Request) -> IronResult<Response> {
    read_single_file("asset-manifest.json", &mut req)
}

//#[get("/favicon.ico")]
pub fn favicon(req: &mut Request) -> IronResult<Response> {
    read_single_file("favicon.ico", &mut req)
}

//#[get("/index.html")]
pub fn index_html(req: &mut Request) -> IronResult<Response> {
    read_single_file("index.html", &mut req)
}

//pub fn any(any: PathBuf, ui_dir: State<UiDir>) -> Option<NamedFile> {
//    NamedFile::open(ui_dir.0.join("index.html")).ok()
//}
pub fn any(_: &mut Request) -> IronResult<Response> {
    use iron::modifiers::Redirect;
    Ok(Response::with((status::Found, Redirect("/"))))
}
