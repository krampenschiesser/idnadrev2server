use ::repository::RepositoryId;

#[derive(Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "Repository not found {}", _0)]
    RepositoryNotFound(RepositoryId),
    #[fail(display = "Data to short. Expected size {} but got {}: {}", expected_size, real_size, msg)]
    DataTooShort{msg: String, expected_size: usize, real_size: usize},

    #[fail(display = "Invalid nonce length. Expected {}, got {}", expected_size, real_size)]
    InvalidNonceLength{expected_size: usize, real_size: usize},
    #[fail(display = "Unknown error occurred")]
    Other,
    #[fail(display = "Invalid Password")]
    InvalidPassword,
}