
#[derive(Debug, PartialEq, Eq, Clone)]
struct AccessToken {
    last_access: Instant,
    id: Uuid,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FileDescriptor {
    repo: Uuid,
    id: Uuid,
    version: u32,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct FileHeaderDescriptor {
    descriptor: FileDescriptor,
    header: String,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RepositoryDescriptor {
    id: Uuid,
    name: String,
}

impl FileDescriptor {
    fn new(header: &FileHeader) -> Self {
        FileDescriptor { repo: header.get_repository_id(), id: header.get_id(), version: header.get_version() }
    }
}

impl FileHeaderDescriptor {
    fn new(enc_file: &EncryptedFile) -> Self {
        let ref h = enc_file.encryption_header;
        let descriptor = FileDescriptor { repo: h.get_repository_id(), id: h.get_id(), version: h.get_version() };
        FileHeaderDescriptor { header: enc_file.header.clone(), descriptor: descriptor }
    }
}

impl RepositoryDescriptor {
    fn new(repo: &Repository) -> Self {
        RepositoryDescriptor { id: repo.get_id(), name: repo.name.clone() }
    }
}
impl AccessToken {
    fn new() -> Self {
        let id = Uuid::new_v4();
        AccessToken { id: id, last_access: Instant::now() }
    }

    fn touch(&mut self) {
        self.last_access = Instant::now();
    }
}