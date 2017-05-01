#![feature(plugin)]
#![feature(custom_attribute)]
#![feature(custom_derive)]
#![plugin(rocket_codegen)]

extern crate uuid;
extern crate rocket;
extern crate rocket_contrib;
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

#[cfg(test)]
extern crate spectral;


mod crypt;
pub mod rest;
mod repository;
mod state;
//mod dummy;
mod actor;

use std::sync::{Arc, RwLock};
use std::sync::mpsc::{channel};
use std::path::{PathBuf, Path};
use rocket::config::{self, ConfigError};
use std::thread;
use repository::service::{RepositoryService, Cmd, Response};
use rest::dto::*;
use rocket::http::Method::*;
use rocket::{Route};
use state::GlobalState;
use crypt::actor::communication::{CryptResponse, CryptCmd};


#[derive(Debug)]
pub struct UiDir(PathBuf);


fn main() {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();

    let r = rocket::ignite();
    let config = config::active().ok_or(ConfigError::NotFound).unwrap();
    let template_dir = PathBuf::from(config.get_str("ui_dir").unwrap());
    let repository_dirs: Vec<PathBuf> = config.get_slice("repository_dirs").unwrap().iter().map(|name| PathBuf::from(name.as_str().unwrap())).collect();

    let state = GlobalState::new(repository_dirs).unwrap();

    r.manage(state)
        //    r.manage(Arc::new(state))
        .manage(UiDir(template_dir))
                .mount("/rest/v1", routes![
                rest::list_repositories,
                rest::create_repository,
        //        rest::open_repository,
        //        rest::close_repository,
        //        rest::get_file,
        //        rest::delete_file,
        //        rest::get_file_header,
        //        rest::save_file_header,
        //        rest::get_file_content,
        //        rest::save_file_content,
                ])
        .mount("/rest/v1", vec![Route::new(Get, "/repo/<id>/?:", rest::list_files)])
        .mount("/rest/v1", vec![Route::new(Get, "/repo/<id>/<type>/?:", rest::list_files_by_type)])
        .mount("/", routes![
            rest::ui::index,
            rest::ui::any,
            rest::ui::files,
            ])
        .launch();
}