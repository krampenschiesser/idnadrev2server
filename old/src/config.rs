use std::path::Path;
use std::io::{Read, Result as IoResult};
use std::fs::File;


#[derive(Deserialize)]
pub struct Config {
    pub ui_dir: String,
    pub repository_dirs: Vec<String>,
}

pub fn read_config(path: &Path) -> IoResult<Config> {
    if !path.exists() {
        return Err(::std::io::Error::new(::std::io::ErrorKind::Other, format!("File {:?} does not exist ", path)));
    }
    let mut f = File::open(path)?;
    let mut s = String::new();
    f.read_to_string(&mut s)?;

    ::toml::from_str(s.as_str()).map_err(|e| ::std::io::Error::new(::std::io::ErrorKind::Other, format!("Could not parse {:?}: {} ", path, e)))
}