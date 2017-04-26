use super::{EncryptionType, MainHeader};
use super::crypto::HashedPw;
use super::super::error::CryptError;
use super::super::util::tempfile::TempFile;
use std::path::PathBuf;
use std::io::{Read, Write, Cursor};
use uuid::Uuid;

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct FileHeader {
    main_header: MainHeader,
    repository_id: Uuid,
    encryption_type: EncryptionType,
    //nonce header length
    //nonce content length
    header_length: u32,
    nonce_header: Vec<u8>,
    nonce_content: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct EncryptedFile {
    encryption_header: FileHeader,
    header: String,
    content: Option<Vec<u8>>,
    path: Option<PathBuf>,
}


impl FileHeader {
    pub fn new(repository: &RepoHeader) -> Self {
        let mh = MainHeader::new(FileVersion::FileV1);
        let enc_type = repository.encryption_type.clone();
        let nc = random_vec(enc_type.nonce_len());
        let nh = random_vec(enc_type.nonce_len());
        FileHeader { main_header: mh, repository_id: repository.main_header.id, encryption_type: enc_type, nonce_content: nc, nonce_header: nh, header_length: 0 }
    }

    pub fn get_id(&self) -> Uuid {
        self.main_header.id.clone()
    }
    pub fn get_repository_id(&self) -> Uuid {
        self.repository_id.clone()
    }

    pub fn set_header_length(&mut self, length: u32) {
        self.header_length = length;
    }

    pub fn get_additional_data(&self) -> Vec<u8> {
        let mut v = Vec::new();
        self.main_header.to_bytes(&mut v);
        v
    }

    pub fn get_version(&self) -> u32 {
        self.main_header.version
    }
}

impl EncryptedFile {
    pub fn new(enc_header: FileHeader, header: &str) -> Self {
        EncryptedFile { path: None, content: None, encryption_header: enc_header, header: header.into() }
    }
    pub fn with_content(enc_header: FileHeader, header: &str, content: &[u8]) -> Self {
        let mut f = EncryptedFile::new(enc_header, header);
        f.content = Some(content.to_vec());
        f
    }

    pub fn set_path(&mut self, path: &PathBuf) {
        self.path = Some(path.clone());
    }

    pub fn set_content(&mut self, content: &[u8]) {
        self.content = Some(content.to_vec());
    }

    pub fn get_id(&self) -> Uuid {
        self.encryption_header.get_id()
    }

    pub fn set_header(&mut self, header: &str) {
        self.header = header.to_string();
    }

    pub fn increment_version(&mut self) {
        self.encryption_header.main_header.version += 1;
    }

    pub fn get_version(&self) -> u32 {
        self.encryption_header.get_version()
    }

    pub fn get_path(&self) -> Option<PathBuf> {
        self.path.clone()
    }
    pub fn load_head(header: &FileHeader, key: &HashedPw, path: &PathBuf) -> Result<Self, CryptError> {
        let f = File::open(path.clone())?;
        let mut f = f.take(header.byte_len() as u64 + header.header_length as u64);
        let mut v = Vec::new();
        f.read_to_end(&mut v)?;
        let mut c = Cursor::new(v.as_slice());

        let additional = header.get_additional_data();
        c.set_position(header.byte_len() as u64);

        let mut buff = vec![0u8; header.header_length as usize];
        c.read_exact(buff.as_mut_slice())?;

        let plaintext = crypt::decrypt(&header.encryption_type, &header.nonce_header, key, buff.as_slice(), additional.as_slice())?;

        let plaintext = String::from_utf8(plaintext)?;

        let result = EncryptedFile { encryption_header: header.clone(), path: Some(path.clone()), content: None, header: plaintext };
        Ok(result)
    }

    pub fn load_content(header: &FileHeader, key: &HashedPw, path: &PathBuf) -> Result<Vec<u8>, CryptError> {
        let mut f = File::open(path.clone())?;
        let mut v = Vec::new();
        f.read_to_end(&mut v)?;
        let mut c = Cursor::new(v.as_slice());

        let additional = header.get_additional_data();
        c.set_position(header.byte_len() as u64 + header.header_length as u64);

        let mut buff = Vec::new();
        c.read_to_end(&mut buff)?;

        let plaintext = crypt::decrypt(&header.encryption_type, &header.nonce_content, key, buff.as_slice(), additional.as_slice())?;
        Ok(plaintext)
    }

    pub fn save(&mut self, key: &HashedPw) -> Result<(), CryptError> {
        let path = self.path.as_ref().ok_or(CryptError::NoFilePath)?;
        if path.exists() {
            return Err(CryptError::IOError(format!("File {} already exists", path_to_str(path))));
        }
        let content = self.content.as_ref().ok_or(CryptError::NoFileContent)?;

        let additional = self.encryption_header.get_additional_data();

        let ref mut header = self.encryption_header;

        let encryptedheadertext = crypt::encrypt(&header.encryption_type, header.nonce_header.as_slice(), key, self.header.as_bytes(), additional.as_slice())?;
        header.set_header_length(encryptedheadertext.len() as u32);
        let encryptedcontent = crypt::encrypt(&header.encryption_type, header.nonce_content.as_slice(), key, content, additional.as_slice())?;

        let mut header_bytes = Vec::new();
        header.to_bytes(&mut header_bytes);

        let mut temp = TempFile::new();
        {
            let mut tempfile = File::create(temp.path.clone())?;
            tempfile.write(header_bytes.as_slice())?;
            tempfile.write(encryptedheadertext.as_slice())?;
            tempfile.write(encryptedcontent.as_slice())?;
            tempfile.sync_all()?;
        }

        rename(temp.path.clone(), path)?;
        temp.moved = true;

        Ok(())
    }
    pub fn update_header(&mut self, key: &HashedPw) -> Result<(), CryptError> {
        let original_enc_header = self.encryption_header.clone();
        self.increment_version();
        let path = self.path.as_ref().ok_or(CryptError::NoFilePath)?;
        if !path.exists() {
            return Err(CryptError::FileDoesNotExist(path_to_str(path)));
        }
        let header_on_filesystem = read_file_header(&path)?;
        if header_on_filesystem.get_version() >= self.get_version() {
            return Err(CryptError::OptimisticLockError(header_on_filesystem.get_version()));
        }

        let content = EncryptedFile::load_content(&original_enc_header, key, &path)?;
        let additional = self.encryption_header.get_additional_data();
        let ref mut header = self.encryption_header;

        let encryptedheadertext = crypt::encrypt(&header.encryption_type, header.nonce_header.as_slice(), key, self.header.as_bytes(), additional.as_slice())?;
        header.set_header_length(encryptedheadertext.len() as u32);
        let encryptedcontent = crypt::encrypt(&header.encryption_type, header.nonce_content.as_slice(), key, content.as_slice(), additional.as_slice())?;

        let mut header_bytes = Vec::new();
        header.to_bytes(&mut header_bytes);

        let mut temp = TempFile::new();
        {
            let mut tempfile = File::create(temp.path.clone())?;
            tempfile.write(header_bytes.as_slice())?;
            tempfile.write(encryptedheadertext.as_slice())?;
            tempfile.write(encryptedcontent.as_slice())?;
            tempfile.sync_all()?;
        }

        rename(temp.path.clone(), path)?;
        temp.moved = true;
        Ok(())
    }

    pub fn get_header(&self) -> &String {
        &self.header
    }
}
