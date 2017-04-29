use super::{EncryptionType, PasswordHashType, MainHeader, FileVersion};
use super::crypto::{HashedPw, DoubleHashedPw, PlainPw};
use super::super::error::{CryptError, ParseError};
use super::super::util::random_vec;
use super::super::util::tempfile::TempFile;
use super::super::util::io::path_to_str;
use std::path::PathBuf;
use std::fs::{rename,File};
use std::io::{Read, Write, Cursor};
use uuid::Uuid;
use byteorder::{WriteBytesExt, LittleEndian};
use super::serialize::*;


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
    //fixme refactor to pw container with user management
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

    pub fn get_encryption_type(&self) -> &EncryptionType {
        &self.encryption_type
    }
    pub fn get_id(&self) -> Uuid {
        self.main_header.id.clone()
    }

    pub fn get_additional_data(&self) -> Vec<u8> {
        let mut v = Vec::new();
        self.main_header.to_bytes(&mut v);
        v
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

    pub fn get_header(&self) -> &RepoHeader {
        &self.header
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

    pub fn set_path(&mut self, path: &PathBuf) {
        self.path = Some(path.clone())
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

    pub fn save(&self) -> Result<(), CryptError> {
        let path = self.path.as_ref().ok_or(CryptError::NoFilePath)?;
        if path.exists() {
            return Err(CryptError::IOError(format!("Repository {} already exists", path_to_str(path))));
        }
        let mut buff = Vec::new();
        self.to_bytes(&mut buff);

        let mut temp = TempFile::new();
        {
            let mut tempfile = File::create(temp.path.clone())?;
            tempfile.write(buff.as_slice())?;
            tempfile.sync_all()?;
        }
        rename(temp.path.clone(), path)?;
        temp.moved = true;

        Ok(())
    }
}


impl ByteSerialization for RepoHeader {
    fn to_bytes(&self, vec: &mut Vec<u8>) {
        self.main_header.to_bytes(vec);
        self.encryption_type.to_bytes(vec);
        self.password_hash_type.to_bytes(vec);

        let salt_len = self.salt.len() as u8;
        vec.push(salt_len);
        vec.append(&mut self.salt.clone());
    }

    fn from_bytes(input: &mut Cursor<&[u8]>) -> Result<Self, ParseError> {
        let main_header = MainHeader::from_bytes(input)?;
        if main_header.file_version != FileVersion::RepositoryV1 {
            return Err(ParseError::InvalidFileVersion(main_header.file_version));
        }
        let enc_type = EncryptionType::from_bytes(input)?;
        let pwh_type = PasswordHashType::from_bytes(input)?;
        let length = read_u8(input)?;
        let mut buff = vec![0u8; length as usize];
        read_buff(input, &mut buff.as_mut_slice())?;
        Ok(RepoHeader { salt: buff, encryption_type: enc_type, main_header: main_header, password_hash_type: pwh_type })
    }
    fn byte_len(&self) -> usize {
        self.main_header.byte_len() + self.encryption_type.byte_len() + self.password_hash_type.byte_len() + 1 + self.salt.len()
    }
}

impl ByteSerialization for Repository {
    fn to_bytes(&self, vec: &mut Vec<u8>) {
        self.header.to_bytes(vec);
        let hash_len = self.hash.len() as u8;
        vec.write_u8(hash_len);
        vec.write(self.hash.as_slice());
        vec.write(self.name.as_bytes());
    }

    fn from_bytes(input: &mut Cursor<&[u8]>) -> Result<Self, ParseError> {
        let h = RepoHeader::from_bytes(input)?;
        let hash_len = read_u8(input)?;
        let mut buff = vec![0u8; hash_len as usize];
        read_buff(input, &mut buff)?;
        let mut namebuff = Vec::new();
        input.read_to_end(&mut namebuff)?;
        let name = String::from_utf8(namebuff)?;
        Ok(Repository { header: h, hash: DoubleHashedPw::from_bytes(buff), name: name, path: None })
    }

    fn byte_len(&self) -> usize {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_repo_header() {
        let kdf = PasswordHashType::SCrypt { iterations: 1, memory_costs: 2, parallelism: 3 };
        let h = RepoHeader::new(kdf, EncryptionType::RingChachaPoly1305);

        let mut result = Vec::new();
        h.to_bytes(&mut result);

        let main_header_len = 2 + 1 + UUID_LENGTH + 4;
        assert_eq!(main_header_len, h.main_header.byte_len());

        let enc_type_len = 1;
        assert_eq!(enc_type_len, h.encryption_type.byte_len());

        let kdf_len = 1 + 1 + 2 * 4;
        assert_eq!(kdf_len, h.password_hash_type.byte_len());

        let salt_len_field = 1;
        let salt_len = 32;
        assert_eq!(salt_len, h.salt.len());


        let expected_len = main_header_len + (enc_type_len) + (kdf_len) + (salt_len_field) + salt_len;
        assert_eq!(expected_len, h.byte_len());
        assert_eq!(expected_len, result.len());

        let parse_back = RepoHeader::from_bytes(&mut Cursor::new(result.as_slice())).unwrap();
        assert_eq!(h, parse_back);
    }
}