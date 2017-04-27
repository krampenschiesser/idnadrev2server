use std::path::PathBuf;
use std::fs::remove_file;
use uuid::Uuid;
use std::ops::Drop;
use std;
use super::io::path_to_str;

pub struct TempFile {
    pub path: PathBuf,
    pub moved: bool,
}

impl TempFile {
    pub fn new() -> Self {
        let tempdir = std::env::temp_dir();
        let name = format!("{}", Uuid::new_v4().simple());
        TempFile::new_in_path(tempdir.join(name))
    }

    pub fn new_in_path(path: PathBuf) -> Self {
        TempFile { path: path, moved: false }
    }
}

impl Drop for TempFile {
    fn drop(&mut self) {
        if !self.moved {
            match remove_file(self.path.clone()) {
                Err(d) => error!("Could not close temp file {}: {}", path_to_str(&self.path), d),
                _ => (),
            }
        }
    }
}