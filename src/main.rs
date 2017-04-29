#![feature(plugin)]
#![feature(custom_attribute)]
#![plugin(rocket_codegen)]

extern crate uuid;
extern crate rocket;
extern crate rocket_contrib;
extern crate chrono;
extern crate serde;
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
#[cfg(test)]
extern crate spectral;


mod crypt;
mod rest;
mod repository;
mod state;
mod dummy;
mod actor;

use std::sync::{Arc, RwLock};
use std::sync::mpsc::{channel};
use std::path::{PathBuf};
use rocket::config::{self, ConfigError};
use std::thread;
use repository::service::{RepositoryService, Cmd, Response};


#[get("/")]
fn index() -> &'static str {
    "Hello, world!"
}

#[derive(Debug)]
pub struct UiDir(PathBuf);


fn tryservice() {
    let (service, mut access) = RepositoryService::new();
    let sender = access.get_sender();

    let t = thread::spawn(move || {
        info!("Before work loop");
        service.work_loop();
        info!("After work loop");
    });
    let mut receivers = Vec::new();
    for i in 0..20 {
        let (s2, r2) = channel();
        sender.send((s2.clone(), Cmd::CreateRepository(format!("Hello #{}", i))));
        receivers.push(r2);
    }
    thread::sleep_ms(1000);
    for r2 in receivers {
        let r: Response = r2.recv().unwrap();
        match r {
            Response::CreatedRepository(id, name) => info!("Created repo {} with id {}", name, id),
            _ => info!("other command"),
        }
    }
    info!("Before stopping");
    access.stop();
    info!("After stopping");
    info!("Before joining");
    t.join();
}

use std::time::Instant;
use ring_pwhash::scrypt::{ScryptParams,scrypt};

fn main() {
    log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap();
    info!("Starting up!");
    trace!("Tracing");
    error!("Error!");
    warn!("Warning");
    tryservice();


    let state = dummy::new_dummy_data();

    let r = rocket::ignite();
    let template_dir = config::active().ok_or(ConfigError::NotFound)
        .map(|config| PathBuf::from(config.get_str("ui_dir").unwrap()))
        .unwrap();


    r.manage(Arc::new(RwLock::new(state)))
        .manage(UiDir(template_dir))
        .mount("/rest/v1", routes![
        index,
        rest::list_repositories,
        rest::create_repository,
        rest::list_files,
        rest::open_repository,
        rest::get_file,
        rest::delete_file,
        rest::get_file_header,
        rest::save_file_header,
        rest::get_file_content,
        rest::save_file_content,
        ])
        .mount("/", routes![
            rest::ui::index,
            rest::ui::any,
            rest::ui::files,
            ])
        .launch();
}