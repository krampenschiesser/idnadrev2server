use ::error::*;
use ::files::FileSource;
use ::pb::file::*;
use crypt::{CipherText, Plaintext, CipherTextVec, DoubleHashedPw};
use failure::Error;
use quick_protobuf::{BytesReader, MessageRead};
pub use self::repository::*;

pub mod repository;
pub mod file;

pub fn open_repository(source: impl FileSource, id: RepositoryId, pw: &Plaintext) -> Result<Repository, Error> {
    let repository_file_names = source.list_repositories()?;

    let callback = |repo: StoredRepositoryV1| -> Result<Repository, Error> {
        let pw = get_password(&repo, pw)?;
        Ok(Repository {
            id: RepositoryId::from_bytes(repo.id.as_ref())?,
            file_pw: repo.encrypted_file_pw.to_vec(),
            double_hash_pw: DoubleHashedPw::from(repo.double_hashed_pw.as_ref()),
            name: repo.name.into(),
        })
    };

    let mut repository = None;

    for n in repository_file_names.iter() {
        let content = source.get_file_content(n)?;
        let mut reader = BytesReader::from_bytes(content.as_ref());
        let res = StoredFileWrapper::from_reader(&mut reader, content.as_ref());
        if let Ok(wrapper) = res {
            if wrapper.type_pb == FileType::RepositoryV1 {
                let mut reader = BytesReader::from_bytes(wrapper.content.as_ref());

                let res = StoredRepositoryV1::from_reader(&mut reader, wrapper.content.as_ref());

                if let Ok(repo) = res {
                    if repo.id.as_ref() == id.as_bytes() {
                        repository = Some(callback(repo)?);
                    }
                }
            }
        }
    }

    if let Some(repo) = repository {
        Ok(repo)
    } else {
        Err(Error::from(ErrorKind::RepositoryNotFound(id)))
    }
}


fn get_password<'a, 'b>(repo: &'a StoredRepositoryV1<'a>, pw: &'b Plaintext) -> Result<CipherTextVec, Error> {
    use chacha20_poly1305_aead::decrypt;
    use crypt::Hasher;

    //get hash of pw with repo salt and hash type
    //get auth tag from end of repo.encrypted_file_pw
    //use repo.id+salt as aad

    let hash = repo.hash_type.hash_pw(pw,repo.salt.as_ref());

    Err(Error::from(ErrorKind::Other))
}