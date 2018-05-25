use ::pb::sync;
use failure::Error;
use repository::file::RepositoryFile;
use sha1::Sha1;
use uuid::Uuid;

pub type FileId = Uuid;
pub type Hash = Vec<u8>;
pub type FileVersion = u32;


#[derive(Debug)]
pub struct HashBucket<'a> {
    hash: Hash,
    divisions: Vec<Subdivision>,
    files: Vec<&'a RepositoryFile>,
}

#[derive(Debug)]
pub struct SingleFileSync {
    id: FileId,
    version: FileVersion,
    hash: Hash,
}

#[derive(Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Clone, Copy)]
pub struct Subdivision {
    division: u32,
    modulo: u8,
    remainder: u32,
}

impl<'a> HashBucket<'a> {
    fn split(self, max_bucket_size: usize) -> Vec<HashBucket<'a>> {
        if self.files.len() > max_bucket_size {
            let mut split = create_buckets(self.files.as_slice(), max_bucket_size, self.divisions);

            loop {
                let option = split.iter_mut().position(|i| i.files.len() > max_bucket_size);
                if let Some(pos) = option {
                    let bucket = split.remove(pos);
                    let mut vec = bucket.split(max_bucket_size);
                    split.append(&mut vec);
                } else {
                    break;
                }
            }
            split
        } else {
            Vec::with_capacity(0)
        }
    }
}

fn create_buckets<'a, 'b>(files: &'b [&'a RepositoryFile], max_bucket_size: usize, divisions: Vec<Subdivision>) -> Vec<HashBucket<'a>> {
    use std::collections::HashMap;

    let mut buckets = Vec::new();
    let mut map: HashMap<u8, HashBucket<'a>> = HashMap::new();
    let modulo = files.len() / max_bucket_size + 1;

    let current_division = divisions.last().map(|d| Subdivision { division: d.division + 1, remainder: 0, modulo: modulo as u8 }).unwrap_or(Subdivision { division: 1, remainder: 0, modulo: modulo as u8 });


    for file in files.iter() {
        let byte = file.id.as_bytes()[15 - current_division.division as usize];
        let map_index = byte % current_division.modulo;

        map.entry(map_index).or_insert_with(|| {
            let mut divs = divisions.clone();
            let mut div = current_division.clone();
            div.remainder = map_index as u32;
            divs.push(div);
            HashBucket {
                divisions: divs,
                files: Vec::new(),
                hash: Vec::with_capacity(0),
            }
        }).files.push(file);
    }

    for (key, value) in map.into_iter() {
        buckets.push(value);
    }

    buckets
}

#[cfg(test)]
mod test {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_create_buckets() {
        let files: Vec<RepositoryFile> = (0..2000_u16).map(|n| create_random_file()).collect();
        let reference: Vec<&RepositoryFile> = files.iter().collect();

        let buckets = HashBucket {
            hash: Vec::with_capacity(0),
            files: reference,
            divisions: Vec::new(),
        }.split(100);
        println!("Got bucket len {}", buckets.len());
        buckets.iter().for_each(|b| {
            println!("File len {}", b.files.len());
            print!("divisions: {:?}", b.divisions);
            assert!(b.hash.len() > 0);
        });
        assert!(buckets.len() > 1);
    }

    fn create_random_file() -> RepositoryFile {
        use pb::file::{CompressionType, EncryptionType};

        RepositoryFile {
            id: Uuid::new_v4(),
            compression_type: CompressionType::DeflateZip,
            encryption_type: EncryptionType::ChachaPoly1305,
            file_name: "bla".into(),
            repository_id: Uuid::new_v4(),
            version: 1,
        }
    }
}