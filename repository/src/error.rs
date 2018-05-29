use ::repository::RepositoryId;

#[derive(Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "Repository not found {}", _0)]
    RepositoryNotFound(RepositoryId),
    #[fail(display = "Unknown error occurred")]
    Other,
}