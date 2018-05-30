use ::error::*;
use ::files::FileSource;
use ::pb::file::*;
use crypt::{CipherText, CipherTextVec, DoubleHashedPw, Plaintext, PlaintextVec};
use failure::Error;
use quick_protobuf::{BytesReader, MessageRead};
pub use self::repository::*;

pub mod repository;
pub mod file;

pub fn open_repository(source: &impl FileSource, id: RepositoryId, pw: &Plaintext) -> Result<Repository, Error> {
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


fn get_password<'a, 'b>(repo: &'a StoredRepositoryV1<'a>, pw: &'b Plaintext) -> Result<PlaintextVec, Error> {
    use chacha20_poly1305_aead::decrypt;
    use crypt::{Hasher, AuthTagProvider, DeEncrypter};
    use std::slice::SliceConcatExt;

    let hashed_pw = repo.hash_type.hash_pw(pw, repo.salt.as_ref());
    let (data, authtag) = repo.enc_type.get_auth_tag(repo.encrypted_file_pw.as_ref())?;
    let aad: &[u8] = &[repo.id.as_ref(), repo.salt.as_ref()].concat();

    let decrypted = repo.enc_type.decrypt(&hashed_pw, repo.nonce.as_ref(), aad, authtag, data)?;

    Ok(decrypted)
}


#[cfg(test)]
mod test {
    use super::*;
    use uuid::Uuid;


    struct TestFileSource;

    impl FileSource for TestFileSource {
        fn list_repositories(&self) -> Result<Vec<String>, Error> {
            Ok(vec!["testrepo".into()])
        }

        fn list_files(&self) -> Result<Vec<String>, Error> {
            unimplemented!()
        }

        fn get_file_content(&self, name: &str) -> Result<Vec<u8>, Error> {
            use quick_protobuf::Writer;

            let message = get_repo();
            let mut out = Vec::new();
            {
                let mut writer = Writer::new(&mut out);
                writer.write_message(&message)?;
            }

            Ok(out)
        }

        fn peek_file_content(&self, name: &str, len: usize) -> Result<Vec<u8>, Error> {
            unimplemented!()
        }

        fn store_file(&mut self, file_name: &str, data: &[u8]) -> Result<(), Error> {
            unimplemented!()
        }
    }

    fn get_uuid() -> Uuid {
        let d4 = [12, 3, 9, 56, 54, 43, 8, 9];

        Uuid::from_fields(42, 12, 5, &d4).unwrap()
    }


    fn get_repo<'a>() -> StoredRepositoryV1<'a> {
        use std::borrow::Cow;
        use crypt::{Hasher, DeEncrypter};
        use std::slice::SliceConcatExt;

        let id = get_uuid();
        let nonce = [9u8; 12];
        let hash_pw = PasswordHashType::Argon2i.hash_pw(b"hallo welt", &nonce);
        let double_hash_pw = PasswordHashType::Argon2i.double_hash_pw(b"hallo welt", &nonce);
        let aad: &[u8] = &[id.as_bytes().as_ref(), nonce.as_ref()].concat();
        let (mut encrypted_pw, mut tag) = EncryptionType::ChachaPoly1305.encrypt(&hash_pw, &nonce, aad, b"real password").unwrap();

        encrypted_pw.append(&mut tag);

        StoredRepositoryV1 {
            nonce: Cow::from(nonce.to_vec()),
            enc_type: EncryptionType::ChachaPoly1305,
            hash_type: PasswordHashType::Argon2i,
            salt: Cow::from(nonce.to_vec()),
            name: Cow::from("test repo"),
            id: Cow::from(id.as_bytes().to_vec()),
            version: 1,
            double_hashed_pw: Cow::from(double_hash_pw.content),
            encrypted_file_pw: Cow::from(encrypted_pw),
        }
    }

    #[test]
    fn test_open_repo() {
        let file_source = TestFileSource {};
        let repo = open_repository(&file_source, get_uuid(), b"hallo welt").unwrap();
    }
}