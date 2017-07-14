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

extern crate iron;
extern crate router;
//extern crate urlencoded;
extern crate persistent;
extern crate mount;
extern crate bodyparser;


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
#[macro_use]
extern crate mime;
extern crate hyper;
extern crate log4rs;
extern crate distance;
extern crate thread_local;
extern crate rayon;
extern crate sha1;
extern crate unicase;

#[cfg(test)]
extern crate spectral;
#[cfg(test)]
extern crate reqwest;

mod ironext;
mod dto;
mod search;
mod crypt;
pub mod rest;
mod state;
//mod dummy;
mod actor;

use std::sync::{Arc, RwLock};
use std::sync::mpsc::{channel};
use std::path::{PathBuf, Path};
use std::thread;
use dto::*;
use state::GlobalState;
use crypt::actor::communication::{CryptResponse, CryptCmd};


#[derive(Debug)]
pub struct UiDir(PathBuf);

use iron::prelude::*;
use router::Router;
use mount::Mount;
use persistent::Read;

fn main() {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();

    let mut mount = Mount::new();
    let mut repo_router = Router::new();

    repo_router.get("/", rest::list_repositories, "list_repositories");
    repo_router.post("/repo", rest::create_repository, "create_repo");
    repo_router.post("/repo/:repo_id", rest::open_repository, "open_repo");
    repo_router.get("/repo/:repo_id/file", rest::create_file, "create_repo_files");
    repo_router.get("/repo/:repo_id", rest::list_files, "list_all_repo_files");
    repo_router.get("/repo/:repo_id/", rest::list_files, "search_all_repo_files");
    repo_router.get("/repo/:repo_id/:type", rest::list_files, "list_all_repo_files_by_type");
    repo_router.get("/repo/:repo_id/:type/", rest::list_files, "search_all_repo_files_by_type");


    #[cfg(debug_assertions)]
    {
        repo_router.post("/repo", rest::cors::create_repository, "cors_create_repo");
        repo_router.post("/repo/:repo_id", rest::cors::open_repo_ping, "cors_open_repo");
        repo_router.post("/repo/:repo_id/:file_id", rest::cors::create_file, "cors_create_file");
    }

    let mut ui_router = Router::new();
    ui_router.get("/", rest::ui::index, "index");
    ui_router.get("/index.html", rest::ui::index, "index.html");
    ui_router.get("/manifest.appcache", rest::ui::manifest, "manifest");
    ui_router.get("/asset-manifest.json", rest::ui::asset_mainfest, "asset-manifest");
    ui_router.get("/favicon.ico", rest::ui::favicon, "favicon");
    ui_router.get("/static/*file_name", rest::ui::files, "files");
    ui_router.get("/*any", rest::ui::any, "redirect_any");


    mount.mount("/rest/v1/repo", repo_router);
    mount.mount("/", ui_router);


    //    let r = rocket::ignite();
    //    let config = config::active().ok_or(ConfigError::NotFound).unwrap();
    //let template_dir = PathBuf::from(config.get_str("ui_dir").unwrap());
    //let repository_dirs: Vec<PathBuf> = config.get_slice("repository_dirs").unwrap().iter().map(|name| PathBuf::from(name.as_str().unwrap())).co
    //llect();
    let state = GlobalState::new(Vec::new()).unwrap();
    let uidir = UiDir(PathBuf::new());

    let mut chain = Chain::new(mount);
    chain.link(Read::<GlobalState>::both(state));
    let iron = Iron::new(chain).http("localhost:8000").unwrap();
}