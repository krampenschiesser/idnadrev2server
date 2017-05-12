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

use super::super::UiDir;

#[get("/")]
pub fn index(ui_dir: State<UiDir>) -> io::Result<NamedFile> {
    info!("{:?}", ui_dir);
    NamedFile::open(ui_dir.0.join("index.html"))
}

#[get("/manifest.appcache")]
pub fn manifest(ui_dir: State<UiDir>) -> Response {
    let dir = &ui_dir.0;

    let hash: ::std::io::Result<String> = hash_dir(&dir);

    let status = if hash.is_ok() { Status::Ok } else { Status::InternalServerError };

    let body = if hash.is_ok() { hash.unwrap() } else { format!("{}", hash.err().unwrap()) };

    Response::build()
        .sized_body(Cursor::new(body))
        .raw_header("Content-Type", "text/cache-manifest")
        .status(status)
        .finalize()
}

#[get("/static/<file..>", rank = 9)]
pub fn files(file: PathBuf, ui_dir: State<UiDir>) -> Option<NamedFile> {
    NamedFile::open(ui_dir.0.join("static").join(file)).ok()
}

#[get("/asset-manifest.json")]
pub fn asset_mainfest(ui_dir: State<UiDir>) -> Option<NamedFile> {
    NamedFile::open(ui_dir.0.join("asset-manifest.json")).ok()
}

#[get("/favicon.ico")]
pub fn favicon(ui_dir: State<UiDir>) -> Option<NamedFile> {
    NamedFile::open(ui_dir.0.join("favicon.ico")).ok()
}

#[get("/index.html")]
pub fn index_html(ui_dir: State<UiDir>) -> Option<NamedFile> {
    NamedFile::open(ui_dir.0.join("index.html")).ok()
}

#[get("/<any..>", rank = 10)]
//pub fn any(any: PathBuf, ui_dir: State<UiDir>) -> Option<NamedFile> {
//    NamedFile::open(ui_dir.0.join("index.html")).ok()
//}
pub fn any(any: PathBuf, ui_dir: State<UiDir>) -> Redirect {
    Redirect::to("/")
}

fn read_dir(root: &Path) -> ::std::io::Result<Vec<PathBuf>> {
    let dir = root.read_dir()?;

    let mut v = Vec::new();

    for entry in dir {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let mut sub = read_dir(&entry.path().to_path_buf())?;
            v.append(&mut sub);
        } else {
            v.push(entry.path().to_path_buf());
        }
    }
    Ok(v)
}

fn hash(paths: &[PathBuf]) -> ::std::io::Result<String> {
    use std::fs::File;
    use std::io::Read;

    let mut sha1 = ::sha1::Sha1::new();
    for path in paths {
        let mut buf = Vec::new();
        File::open(path)?.read_to_end(&mut buf)?;
        sha1.update(buf.as_slice());
    }
    Ok(sha1.digest().to_string())
}

fn hash_dir(root: &PathBuf) -> ::std::io::Result<String> {
    let paths = read_dir(root)?;

    let hash = hash(&paths)?;
    let mut names = Vec::new();
    for path in paths {
        names.push(path_to_string(root, &path))
    }
    let mut retval = String::new();
    retval.push_str("CACHE MANIFEST\n");
    retval.push_str("#");
    retval.push_str(hash.as_str());
    retval.push_str("\nCACHE:\n");
    for name in names {
        retval.push_str(name.as_str());
        retval.push_str("\n");
    }
    retval.push_str("\nFALLBACK:\n");
    retval.push_str("/ /index.html\n");

    retval.push_str("\nNETWORK:\n*\n");

    Ok(retval)
}

fn path_to_string(root: &PathBuf, path: &PathBuf) -> String {
    let mut str = String::new();
    let prefix = root.as_os_str().to_string_lossy();

    for p in path {
        let p = p.to_string_lossy().into_owned();
        if !prefix.contains(p.as_str()) {
            str.push_str(format!("/{}", p).as_str());
        }
    }
    str
}

#[cfg(test)]
mod tests {
    use super::hash_dir;
    use tempdir::TempDir;
    use std::fs::{File, create_dir};
    use spectral::prelude::*;

    #[test]
    fn test_hash_dir() {
        let temp = TempDir::new("hashing").unwrap();
        let root = temp.path().to_path_buf();

        let subdir = root.join("subdir");
        create_dir(&subdir);

        let file1 = root.join("file1.txt");
        let file2 = subdir.join("file2.txt");

        File::create(file1);
        File::create(file2);

        let hash = hash_dir(&root).unwrap();
        println!("{}", hash);
        assert_that(&hash).contains("/subdir/file2.txt");
        assert_that(&hash).contains("/file1.txt");
    }
}