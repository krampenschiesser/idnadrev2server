use ::pb::sync;
use uuid::Uuid;

pub type FileId = Uuid;
pub type Hash = Vec<u8>;
pub type FileVersion = u32;


pub struct HashBucket {
    hash: Hash,
    divisions: Vec<Subdivision>,
}


pub struct SingleFileSync {
    id: FileId,
    version: FileVersion,
    hash: Hash,
}


pub struct Subdivision {
    division: u32,
    modulo: u32,
    remainder: u32,
}



#[cfg(test)]
mod test {
    use super::*;
}