// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![feature(plugin)]
#![feature(custom_attribute)]
#![feature(custom_derive)]


extern crate uuid;
extern crate chrono;
extern crate serde;
#[macro_use]
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
extern crate byteorder;
extern crate rand;
extern crate ring;
extern crate ring_pwhash;
extern crate base64;
extern crate tempdir;
extern crate notify;
#[macro_use]
extern crate log;
extern crate log4rs;
extern crate distance;
extern crate thread_local;
extern crate rayon;
extern crate sha1;
extern crate rest_in_rust;
extern crate http;

#[cfg(test)]
extern crate spectral;
#[cfg(test)]
extern crate reqwest;

mod dto;
mod search;
mod crypt;
pub mod rest;
mod state;
//mod dummy;
mod actor;

use std::sync::{Arc, RwLock};
use std::sync::mpsc::channel;
use std::path::{PathBuf, Path};
use std::thread;
use dto::*;
use state::GlobalState;
use crypt::actor::communication::{CryptResponse, CryptCmd};


#[derive(Debug)]
pub struct UiDir(PathBuf);

use rest_in_rust::prelude::*;

fn main() {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();

    let mut router = Router::new();
    router.get("/rest/v1/", rest::list_repositories);
    router.post("/rest/v1/repo", rest::create_repository);
    router.post("/rest/v1/repo/:repo_id", rest::open_repository);
    router.get("/rest/v1/repo/:repo_id/file", rest::create_file);
    router.get("/rest/v1/repo/:repo_id", rest::list_files);
    router.get("/rest/v1/repo/:repo_id/", rest::list_files);
    router.get("/rest/v1/repo/:repo_id/:type", rest::list_files);
    router.get("/rest/v1/repo/:repo_id/:type/", rest::list_files);

    router.get("/", rest::ui::index);
    router.get("/index.html", rest::ui::index);
    router.get("/manifest.appcache", rest::ui::manifest);
    router.get("/asset-manifest.json", rest::ui::asset_mainfest);
    router.get("/favicon.ico", rest::ui::favicon);
    router.get("/static/*file_name", rest::ui::files);
    router.get("/*any", rest::ui::any);

    //    #[cfg(debug_assertions)]
    //    {
    //        router.post("/repo", rest::cors::create_repository, "cors_create_repo");
    //        router.post("/repo/:repo_id", rest::cors::open_repo_ping, "cors_open_repo");
    //        router.post("/repo/:repo_id/:file_id", rest::cors::create_file, "cors_create_file");
    //    }


    let state = GlobalState::new(Vec::new()).unwrap();
    let uidir = UiDir(PathBuf::new());

    let addr = "127.0.0.1:8091".parse().unwrap();
    let s = Server::new(addr, router);
    s.add_state(state);
    s.add_state(uidir);
    s.start_http_blocking().unwrap();
}