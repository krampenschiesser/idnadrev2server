use super::{EncryptionType, PasswordHashType, MainHeader};
use super::crypto::{HashedPw, DoubleHashedPw, PlainPw};
use super::super::error::CryptError;
use std::path::PathBuf;
use std::io::{Read, Write, Cursor};
use uuid::Uuid;


#[derive(Debug, Eq, PartialEq, Clone)]
pub struct RepoHeader {
    pub main_header: MainHeader,
    pub encryption_type: EncryptionType,
    pub password_hash_type: PasswordHashType,
    pub salt: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Repository {
    header: RepoHeader,
    hash: DoubleHashedPw,
    name: String,
    path: Option<PathBuf>,
}

impl RepoHeader {
    #[cfg(test)]
    pub fn new_for_test() -> Self {
        let it = 1;
        let mem = 1;
        let cpu = 1;
        let kdf = PasswordHashType::SCrypt { iterations: it, memory_costs: mem, parallelism: cpu };
        RepoHeader::new(kdf, EncryptionType::RingChachaPoly1305)
    }
    pub fn new(kdf: PasswordHashType, enc_type: EncryptionType) -> Self {
        let salt = random_vec(kdf.salt_len());
        let mh = MainHeader::new(FileVersion::RepositoryV1);
        RepoHeader { main_header: mh, encryption_type: enc_type, password_hash_type: kdf, salt: salt }
    }

    pub fn get_id(&self) -> Uuid {
        self.main_header.id.clone()
    }
}

impl Repository {
    pub fn new(name: &str, pw: PlainPw, header: RepoHeader) -> Self {
        let checksum = {
            let v = Repository::hash_key_ext(&header.encryption_type, &header.password_hash_type, pw);
            Repository::hash_pw_ext(&header.encryption_type, &header.password_hash_type, &v)
        };
        Repository { header: header, hash: checksum, name: name.into(), path: None }
    }

    pub fn get_id(&self) -> Uuid {
        self.header.get_id()
    }

    pub fn get_name(&self) -> String {
        self.name.clone()
    }

    pub fn get_folder(&self) -> Option<PathBuf> {
        match self.path {
            Some(ref p) => p.parent().map(|p| p.to_path_buf()),
            None => None
        }
    }

    pub fn hash_key(&self, pw_plain: PlainPw) -> HashedPw {
        Repository::hash_key_ext(&self.header.encryption_type, &self.header.password_hash_type, pw_plain)
    }

    pub fn hash_pw(&self, pw: &HashedPw) -> DoubleHashedPw {
        Repository::hash_pw_ext(&self.header.encryption_type, &self.header.password_hash_type, pw)
    }

    pub fn hash_key_ext(enc_type: &EncryptionType, hash_type: &PasswordHashType, pw_plain: PlainPw) -> HashedPw {
        HashedPw::new(pw_plain, enc_type, hash_type)
    }

    pub fn hash_pw_ext(enc_type: &EncryptionType, hash_type: &PasswordHashType, pw: &HashedPw) -> DoubleHashedPw {
        DoubleHashedPw::new(pw, enc_type, hash_type)
    }

    pub fn check_plain_pw(&self, pw_plain: PlainPw) -> bool {
        let single = self.hash_key(pw_plain);
        let double = self.hash_pw(&single);

        double == self.hash
    }

    pub fn check_hashed_key(&self, pw: &HashedPw) -> bool {
        let double = self.hash_pw(&pw);
        double == self.hash
    }

    pub fn get_path(&self) -> Option<PathBuf> {
        self.path.clone()
    }
    pub fn load(path: PathBuf) -> Result<Self, CryptError> {
        let mut f = File::open(path.clone())?;
        let mut v = Vec::new();
        f.read_to_end(&mut v)?;

        let mut c = Cursor::new(v.as_slice());
        let mut repo = Repository::from_bytes(&mut c)?;
        repo.path = Some(path);
        Ok(repo)
    }
}