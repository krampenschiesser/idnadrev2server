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
#![feature(drop_types_in_const)]
#![feature(const_fn)]

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
extern crate toml;

#[cfg(test)]
extern crate spectral;

mod dto;
mod search;
mod crypt;
pub mod rest;
mod state;
//mod dummy;
mod actor;
mod config;

use std::path::{Path, PathBuf};
use state::{GlobalState, UiState};


#[derive(Debug)]
pub struct UiDir(PathBuf);

use rest_in_rust::*;

fn main() {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();

    //    #[cfg(debug_assertions)]
    //    {
    //        router.post("/repo", rest::cors::create_repository, "cors_create_repo");
    //        router.post("/repo/:repo_id", rest::cors::open_repo_ping, "cors_open_repo");
    //        router.post("/repo/:repo_id/:file_id", rest::cors::create_file, "cors_create_file");
    //    }

    let config = match config::read_config(Path::new("idnadrev.toml")) {
        Ok(c) => c,
        Err(e) => {
            error!("Could not read configuration file: '{}'", e);
            return;
        }
    };


    let state = GlobalState::new(Vec::new()).unwrap();
    let ui_state = UiState::new(Path::new(config.ui_dir.as_str()).to_owned());

    let mut router = Router::new();
    router.get("/rest/v1/repo", rest::list_repositories);
    router.post("/rest/v1/repo", rest::create_repository);
    router.post("/rest/v1/repo/:repo_id", rest::open_repository);
    router.get("/rest/v1/repo/:repo_id/file", rest::create_file);
    router.get("/rest/v1/repo/:repo_id", rest::list_files);
    router.get("/rest/v1/repo/:repo_id/", rest::list_files);
    router.get("/rest/v1/repo/:repo_id/:type", rest::list_files);
    router.get("/rest/v1/repo/:repo_id/:type/", rest::list_files);


    router.static_path_cached("/", path(&ui_state.ui_dir, "index.html"), ChangeDetection::FileInfoChange, EvictionPolicy::Never);
    router.static_path_cached("/index.html", path(&ui_state.ui_dir, "index.html"), ChangeDetection::FileInfoChange, EvictionPolicy::Never);
    router.get("/manifest.appcache", rest::ui::manifest);
    router.static_path_cached("/asset-manifest.json", path(&ui_state.ui_dir, "asset-manifest.json"), ChangeDetection::FileInfoChange, EvictionPolicy::Never);
    router.static_path_cached("/favicon.ico", path(&ui_state.ui_dir, "favicon.ico"), ChangeDetection::FileInfoChange, EvictionPolicy::Never);
    router.static_path_cached("/static", path(&ui_state.ui_dir, "static"), ChangeDetection::FileInfoChange, EvictionPolicy::Never);
    router.static_path_cached("/static/css", path(&ui_state.ui_dir, "static/css"), ChangeDetection::FileInfoChange, EvictionPolicy::Never);
    router.static_path_cached("/static/js", path(&ui_state.ui_dir, "static/js"), ChangeDetection::FileInfoChange, EvictionPolicy::Never);
    router.get("/*any", rest::ui::any);

    let addr = "127.0.0.1:8000".parse().unwrap();
    let s = Server::new(addr, router);
    s.add_state(state);
    s.add_state(ui_state);
    s.start_http();
}

fn path(ui_dir: &PathBuf, file: &str) -> PathBuf {
    let mut file_path = ui_dir.clone();
    file_path.push(file);
    file_path
}
