use std::io;
use std::path::{PathBuf};
use rocket::response::NamedFile;
use rocket::State;

use super::super::UiDir;

#[get("/")]
pub fn index(ui_dir: State<UiDir>) -> io::Result<NamedFile> {
    println!("{:?}", ui_dir);
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