//! # Module containing the REST Interface
//!
//! Current version of the rest interface is v1. This might change in the future if changes are nessessary.
//!
//! The main idea to retrieve content is via files and repositories.
//! First you need to open a repository, as return you get a token that you will use for further access.
//! Then you can list its files and access them.
//!
//! ## GET Methods
//!
//! |Path                                 |Description                                   |Returns             |
//! |-------------------------------------|:---------------------------------------------|--------------------|
//! |GET /repo                            |List all repositories managed by this instance|Vec of all [`Repository`](dto/struct.Repository.html)|
//! |GET /repo/`<uuid>`/file              |Lists all files in a repository               |[`Page`](dto/struct.Page.html) of [`File`](dto/struct.File.html), see [Paging](#searching-and-paging) below|
//! |GET /repo/`<uuid>`/file/`<uui>`      |Retrieves a file header                       |[`File`](dto/struct.File.html), only header filled|
//! |GET /repo/`<uuid>`/file/`<uui>`/full |Retrieves a file with content                 |[`File`](dto/struct.File.html), content and header filled|
//!
//! ### Specialized GET methods
//!
//! Since every file has its type given in the header you can request a file of a specific type directly:
//!
//! For example when you have a type = **doc**
//!
//! |Path                                             |Description                      |Returns|
//! |-------------------------------------------------|:--------------------------------|-------|
//! |GET /repo/`<uuid>`/doc/                  |Retrieves all documents in repo  |Same as above|
//! |GET /repo/`<uuid>`/doc/`<uui>`/          |Retrieves the header of a file with type doc |Same as above|
//! |GET /repo/`<uuid>`/doc/`<uui>`/full      |Retrieves a file of type doc with content    |Same as above|
//!
//!
//! ## POST Methods
//!
//! |Path                                            |Description                                   |Body                                       |
//! |------------------------------------------------|:---------------------------------------------|-------------------------------------------|
//! |POST /repo                                |Creates a new repository                            |[`CreateRepository`](dto/struct.CreateRepository.html)|
//! |POST /repo/`<uuid>`                       |Opens an existing repository                        |[`OpenRepository`](dto/struct.OpenRepository.html)|
//! |POST /repo/`<uuid>`/file/`<uui>`          |Update a file with header and, if set, the content too|[`File`](dto/struct.File.html)|
//!
//! ## DELETE Methods
//!
//! |Path                                       |Description                    |
//! |-------------------------------------------|:------------------------------|
//! |DELETE /repo/`<uuid>`                      |Delete an existing repository  |
//! |DELETE /repo/`<uuid>`/file/`<uui>`         |Delete file in repository      |
//!
//! ## Searching and paging
//!
//! Searching and paging is only allowed for the `/repo/<uuid>/file` url.
//! The default sorting for files is descending on their last modification time.
//! However when you start filtering this does not apply anymore and the sorting depends on the used filter.
//!
//! The default amount of results returned(page size) are 25.
//! The paging is configurable via the following 2 parameters:
//!
//! * offset=0
//! * limit=25
//!
//! For getting 30 documents between index 50(inclusive) and 80(exclusive):
//! > /repo/4711/doc/4242/?offset=50&limit=30
//!
//! ### ?any=text or
//!
//! This searches for text in all fields.
//!
//! 1. It searches in the tags of a file
//! 2. It searches in the name of a file
//! 3. Now if not enough results were found it will search in the content of the file for matches
//!
//! So files with the text in the name will be returned first by paging, then files with text as a tag.
//!
//! ### Filters
//!
//! Filters apply only to json key/value pairs in the header of a file.
//! Curently there are 3 types of filters:
//!
//! 1. Text filters
//! 2. Date filters
//! 3. Number filters [x] not yet implemented
//!
//! #### Text filters
//!
//! For example if we have a property _"location": "home"_ you could use the following query:
//! > /repo/4711/meeting/4242/?location=home
//!
//! * eq = Equal
//! * ne = Not equal
//! * nl = null/emtpy
//! * nn = not null/ not emtpy
//! * fc = fuzzy contains, default when nothing is given
//! * ct = contains
//! * nc = not contains
//!
//! #### Date filters
//!
//! Dates are in [RFC 3339](https://www.ietf.org/rfc/rfc3339.txt)(strict ISO 8601) format, stored and returned in UTC.
//! The search pattern is the following `date:[Operator][UTCDate]`
//!
//! > /repo/4711/meeting/4242/?updated=date:gt:2016-04-28T16:24:32+01:00
//!
//! The following operators exist:
//!
//! * eq = Equal, default used when no operator is given
//! * ne = Not equal
//! * nl = null/emtpy
//! * nn = not null/ not emtpy
//! * gt = Greater than
//! * lt = Less than
//! * ge = Greater than or equal to
//! * le = Less than or equal to
//!

use rocket_contrib::UUID;
use state::{RepositoryState, RepoNamesDTO};
use repository::{RepositoryFile, Repository};
use rocket::State;
use rocket::response::Stream;
use rocket_contrib::{JSON};
use std::io::Cursor;
use std::sync::{RwLock, Arc};
use self::dto::CreateRepository;
use uuid::Uuid;

pub mod ui;
pub mod dto;
mod searchparam;
mod search;

use rocket::{Route, Data, Outcome, Request};
use rocket::handler;
use rocket::http::Status;
use self::searchparam::SearchParam;

pub fn list_files<'r>(request: &'r Request, data: Data) -> handler::Outcome<'r> {
    let search = if let Some(query) = request.uri().query() {
        match SearchParam::from_query_param(query) {
            Err(e) => {
                error!("{:?}",e);
                return Outcome::failure(Status::BadRequest);
            },
            Ok(param) => param,
        }
    } else {
        SearchParam::new()
    };
    Outcome::of(JSON(search))
}

pub fn list_files_by_type<'r>(request: &'r Request, data: Data) -> handler::Outcome<'r> {
    Outcome::of("huhu")
}

#[derive(Debug)]
pub struct LockingError;

#[get("/repository")]
pub fn list_repositories(state: State<Arc<RwLock<RepositoryState>>>) -> Result<JSON<Vec<RepoNamesDTO>>, LockingError> {
    let s = state.read().map_err(|p| LockingError {})?;
    Ok(JSON(s.get_repo_names()))
}

#[post("/repository", data = "<create_repo>")]
pub fn create_repository(create_repo: JSON<CreateRepository>) -> Option<String> {
    info!("Received : {:?}", create_repo);
    Some(format!("{:?}", create_repo))
}

#[post("/repository/<repository_id>/open")]
pub fn open_repository(repository_id: UUID) -> String {
    format!("{}", repository_id.into_inner().simple())
}

//#[get("/repository/<repository_id>")]
//pub fn list_files(repository_id: UUID, state: State<Arc<RwLock<RepositoryState>>>) -> Result<Option<JSON<Vec<RepositoryFile>>>, LockingError> {
//    let s = state.read().map_err(|p| LockingError {})?;
//    let option: Option<&Repository> = s.get_repository(&repository_id.into_inner());
//    Ok(option.map(|r| JSON(r.get_files())))
//}

#[delete("/repository/<repository_id>/<file_id>")]
pub fn delete_file(repository_id: UUID, file_id: UUID) {}

#[get("/repository/<repository_id>/<file_id>")]
pub fn get_file(repository_id: UUID, file_id: UUID, state: State<Arc<RwLock<RepositoryState>>>) -> Result<Option<JSON<RepositoryFile>>, LockingError> {
    get_file_header(repository_id, file_id, state)
}

#[get("/repository/<repository_id>/<file_id>/head")]
pub fn get_file_header(repository_id: UUID, file_id: UUID, state: State<Arc<RwLock<RepositoryState>>>) -> Result<Option<JSON<RepositoryFile>>, LockingError> {
    let s = state.read().map_err(|p| LockingError {})?;
    let repo_option: Option<&Repository> = s.get_repository(&repository_id.into_inner());
    let file_option = repo_option.map_or(None, |r| r.get_file_header(&file_id));
    Ok(file_option.map(|f| JSON(f)))
}

#[post("/repository/<repository_id>/<file_id>/head")]
pub fn save_file_header(repository_id: UUID, file_id: UUID) {}

#[get("/repository/<repository_id>/<file_id>/content")]
pub fn get_file_content(repository_id: UUID, file_id: UUID, state: State<Arc<RwLock<RepositoryState>>>) -> Result<Option<Stream<Cursor<Vec<u8>>>>, LockingError> {
    let s = state.read().map_err(|p| LockingError {})?;
    let repo_option: Option<&Repository> = s.get_repository(&repository_id.into_inner());
    let content_option: Option<Vec<u8>> = repo_option.map_or(None, |r| r.get_file_content(&file_id));
    Ok(content_option.map(|c| Stream::from(Cursor::new(c))))
}

#[post("/repository/<repository_id>/<file_id>/content")]
pub fn save_file_content(repository_id: UUID, file_id: UUID, state: State<Arc<RwLock<RepositoryState>>>) {}


#[cfg(test)]
mod test {
    use rocket;
    use rocket::testing::MockRequest;
    use rocket::http::Status;
    use rocket::http::Method::*;
    use std::sync::{Arc, RwLock};
    use state::RepositoryState;
    use uuid::Uuid;
    use super::dto::*;
    use serde_json;
    use rocket::http::ContentType;

    #[test]
    fn test_create_repo() {
        let lock = Arc::new(RwLock::new(RepositoryState::new()));
        let rocket = rocket::ignite()
            .manage(lock.clone())
            .mount("/", routes![super::create_repository]);

        let create_repo = CreateRepository { password: "bla".as_bytes().to_vec(), encryption: EncryptionType::ChaCha, user_name: "user".to_string(), name: "My Repository".to_string() };
        let json = serde_json::to_string(&create_repo).unwrap();

        let mut req = MockRequest::new(Post, "/repository").body(json);
        req.add_header(ContentType::JSON);

        let mut response = req.dispatch_with(&rocket);
        assert_eq!(response.status(), Status::Ok);

        //        fixme reimplemt actual method
        //        let body_string = response.body().and_then(|b| b.into_string()).unwrap();
        //        let uuid = Uuid::parse_str(body_string.as_str()).unwrap();
        //
        //        {
        //            let state = lock.read().unwrap();
        //            assert!(state.get_repository(&uuid).is_some());
        //        }
    }
}
