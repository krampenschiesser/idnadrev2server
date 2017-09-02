mod filter;
mod searchparam;

pub use self::searchparam::SearchParam;
use self::filter::filter_text;

use crypt::{CryptoSender, CryptoIfc };
use dto::{RepoId,AccessToken, Page, File,FileHeaderDescriptor};

pub struct SearchCache {
    pub crypt_sender: CryptoSender,
}

impl SearchCache {
    pub fn new(crypt_sender: CryptoSender) -> Self {
        SearchCache { crypt_sender }
    }

    pub fn search(&self, param: SearchParam, repo_id: &RepoId, token: &AccessToken) -> Page {
        let files = self.get_files(repo_id, token);
        param.filter(files.as_slice())
    }

    fn get_files(&self, repo_id: &RepoId, token: &AccessToken) -> Vec<File> {
        if let Some(mut v) = self.crypt_sender.list_repository_files(repo_id, token) {
            let mut retval: Vec<File> = v.iter_mut()
                .map(|fhd| File::from_descriptor(fhd))
                .filter(|r| r.is_ok())
                .map(|r| r.unwrap())
                .collect();
            retval
        } else {
            Vec::new()
        }
    }
}

impl SearchParam {
    fn filter_file(&self, file: &File) -> bool {
        use search::searchparam::SearchFilter;
        use self::filter::filter_date_time;
        use self::searchparam::FilterOperator;
        use self::filter::fuzzy_contains;

        if let Some(ref file_type) = self.file_type {
            if &file.file_type != file_type {
                return false
            }
        }

        if let Some(ref title) = self.name {
            if &file.name != title {
                return false
            }
        }
        if let Some(ref filter) = self.created {
            if !filter_date_time(filter, &file.created) {
                return false
            }
        }
        if let Some(ref filter) = self.updated {
            if !filter_date_time(filter, &file.updated) {
                return false
            }
        }
        if let Some(ref filter) = self.deleted {
            if let Some(ref deletion_time) = file.deleted {
                if !filter_date_time(filter, deletion_time) {
                    return false
                }
            }
        }

        if !self.tags.is_empty() {
            let mut found_any = false;
            for tag in &self.tags {
                for other in &file.tags {
                    if fuzzy_contains(tag.as_str(), other.as_str()) {
                        found_any = true;
                        break;
                    }
                }
                if found_any {
                    break;
                }
            }
            if !found_any {
                return false;
            }
        }

        if let Some(ref any) = self.any {
            let contained = fuzzy_contains(any.as_str(), file.name.as_str());
            let contained = contained || {
                for tag in &file.tags {
                    if fuzzy_contains(any.as_str(), tag.as_str()) {
                        return true
                    }
                }
                false
            };
            if !contained {
                return false;
            }
            //let contained = contained || search_in_content(any,file.content); //fixme need to add content search and retrieve....
        }

        if !self.text_filters.is_empty() {
            if let Some(ref details) = file.details {
               if !self.text_filters.iter().all(|filter| filter.test(details)) {
                   return false;
               }
            } else {
                return false;
            }
        }

        if !self.date_filters.is_empty() {
            if let Some(ref details) = file.details {
                if !self.date_filters.iter().all(|filter| filter.test(details)) {
                    return false;
                }
            } else {
                return false;
            }
        }
        true
    }

    fn filter(&self, files: &[File]) -> Page {
        use rayon::prelude::*;
        use std::cmp;

        let mut found: Vec<&File> = files.par_iter()
            .filter(|f| self.filter_file(f)).collect();

        found.sort_by(|f1, f2| f1.updated.cmp(&f2.updated).reverse());


        let mut page = Page::empty();
        page.total = Some(found.len() as u32);
        page.offset = self.offset;

        if self.offset < found.len() as u32 {
            let tail = found.split_off(self.offset as usize);

            let limit = cmp::min(self.limit as usize, tail.len());
            let mut result: Vec<File> = Vec::with_capacity(self.limit as usize);

            for f in tail[0..limit].iter() {
                result.push(f.to_owned().to_owned());
            }

            page.files = result;
            page.limit = limit as u32;
        } else {
            page.limit = 0;
        }
        page
    }
}