use crypt::{DoubleHashedPw, CipherTextVec};
use failure::Error;
use std::convert::TryFrom;
use uuid::Uuid;

pub type RepositoryId = Uuid;
pub type RepositoryName = String;

pub struct Repository {
    pub name: RepositoryName,
    pub id: RepositoryId,
    pub double_hash_pw: DoubleHashedPw,
    pub file_pw: CipherTextVec,
}

impl<'a> TryFrom<::pb::file::StoredRepositoryV1<'a>> for Repository {
    type Error = ::failure::Error;

    fn try_from(original: ::pb::file::StoredRepositoryV1<'a>) -> Result<Repository, Error> {
        Ok(Repository {
            name: original.name.into(),
            id: Uuid::from_bytes(original.id.as_ref())?,
            double_hash_pw: DoubleHashedPw::from(original.double_hashed_pw.as_ref()),
            file_pw: original.encrypted_file_pw.into(),
        })
    }
}