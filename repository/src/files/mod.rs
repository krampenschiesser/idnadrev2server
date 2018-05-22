use failure::Error;

pub type FileName = String;

pub trait FileSource {
    fn list_repositories(&self) -> Result<Vec<FileName>, Error>;

    fn list_files(&self) -> Result<Vec<FileName>, Error>;

    fn get_file_content(&self, name: &str) -> Result<Vec<u8>, Error>;

    fn store_file(&mut self, file_name: &str, data: &[u8]) -> Result<(), Error>;
}