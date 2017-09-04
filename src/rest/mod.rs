// Copyright 2017 Christian Löhnert. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

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
//! |GET /repo/`<uuid>`/file/sync         |Retrieves a sync                              |[`Sync`](dto/struct.Sync.html), see [Sync](#sync)|
//!
//! ### Specialized GET methods
//!
//! Since every file has its type given in the header you can request a file of a specific type directly:
//!
//! For example when you have a type = **doc**
//!
//! |Path                                             |Description                          |Returns|
//! |-------------------------------------------------|:------------------------------------|-------|
//! |GET /repo/`<uuid>`/doc/                  |Retrieves all documents in repo              |Same as above|
//! |GET /repo/`<uuid>`/doc/`<uui>`/          |Retrieves the header of a file with type doc |Same as above|
//! |GET /repo/`<uuid>`/doc/`<uui>`/full      |Retrieves a file of type doc with content    |Same as above|
//! |GET /repo/`<uuid>`/doc/sync              |Retrieves a sync                             |[`Sync`](dto/struct.Sync.html), see [Sync](#sync)|
//!
//!
//! ## POST Methods
//!
//! |Path                                            |Description                                   |Body                                       |
//! |------------------------------------------------|:---------------------------------------------|-------------------------------------------|
//! |POST /repo                                |Creates a new repository                            |[`CreateRepository`](dto/struct.CreateRepository.html)|
//! |POST /repo/`<uuid>`                       |Opens an existing repository                        |[`OpenRepository`](dto/struct.OpenRepository.html)|
//! |POST /repo/`<uuid>`/file                  |Creates a file with header and content              |[`File`](dto/struct.File.html)|
//! |POST /repo/`<uuid>`/file/`<uui>`          |Update a file with header and, if set, the content too|[`File`](dto/struct.File.html)|
//!
//! ## DELETE Methods
//!
//! |Path                                       |Description                    |
//! |-------------------------------------------|:------------------------------|
//! |DELETE /repo/`<uuid>`                      |Delete an existing repository  |
//! |DELETE /repo/`<uuid>`/file/`<uui>`         |Delete file in repository      |
//!
//! ## Sync
//!
//! If you have clients with full offline capabilities they have to retrieve only changes made to the repository.
//! A simple way to do this would be to ask for files modified after the max(modification) time in the local storage.
//! However once the storage of the repository is shared this won't work anymore as a file modification might become
//! visible in the next `sync` between the two repositories.
//! The sync method consists of the following:
//!
//! 1. a `modificationStartTime` time to indicate which changes to get, everything after the modification start time
//! 2. a sha1 hash over the id's and versions of all local files before that modification time
//! 3. an optional `modificationEndTime` if given this will indicate to the server to check the hash in that time frame
//!     if the hash matches only a successful sync will be returned, otherwise it will be filled with versions and ids
//!
//! With this method the server can quickly compute the hash on its own and see if nothing changed in the client data
//! This would be the case for simple 1 user systems.
//! The client/server can also cache the sha1 checksum for specific age groups using the optional `modificationEndTime` parameter
//! to peek for changes.
//!
//! If the hash matches a sync reply contains the ID's and versions that have a modifcation time after the given modfication start time.
//! If the hash does not match it will return a sync with hash_matches flag set to false.
//!
//! If it is a peek request with the optional `modificationEndTime` parameter given, it will either return a successful sync
//! or the ID's and versions in that range.
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
//! 1. It searches in the name of a file
//! 2. It searches in the tags of a file
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

use crypt::CryptoIfc;
use dto::*;

//#[cfg(debug_assertions)]
//pub mod cors;

pub mod ui;

use search::SearchParam;
use state::GlobalState;
use crypt::CryptoActor;
use dto::AccessToken;
use serde_json::to_string;

use rest_in_rust::*;
use http::StatusCode;
use http::header;
//
//#[error(404)]
//pub fn not_found<'a>(req: &'a Request) -> Response {
//    if req.method() == Method::Options {
//        Response::build().status(Status::Ok).finalize()
//    } else {
//        Response::build().status(Status::Ok).finalize()
//    }
//}

//#[get("/<any..>", rank = 5)]
//pub fn any<'a>(any: PathBuf) -> Response<'a> {
//    Response::build().status(Status::NotFound).raw_header("Access-Control-Allow-Origin", "http://localhost:3000").finalize()
//}


pub fn list_files(req: &mut Request) -> Result<Response, HttpError> {
    let search = SearchParam::from_req(req)?;
    let token = AccessToken::from_req(req)?;
    let repo_id = RepoId::from_req(req)?;
    let state = GlobalState::from_req_as_ref(req)?;

    if state.check_token(&repo_id, &token) {
        let page = list_files_internal(search, &repo_id, &token, state);
        match to_string(&page) {
            Ok(str) => Ok(str.into()),
            Err(e) => Err(format!("Could not serialize page: {}", e).into())
        }
    } else {
        Err(HttpError::unauthorized("Token invalid"))
    }
}

fn list_files_internal(search: SearchParam, repo_id: &RepoId, token: &AccessToken, state: &GlobalState) -> Page {
    state.search_cache().search(search, repo_id, token)
}

//#[get("/repo")]
pub fn list_repositories(req: &mut Request) -> Result<Response, HttpError> {
    let state = GlobalState::from_req_as_ref(req)?;

    let c: &CryptoActor = state.crypt();
    let option = c.list_repositories();
    let (body, status) = match option {
        None => ("No result...".to_string(), StatusCode::NOT_FOUND),
        Some(vec) => (to_string(&vec).unwrap(), StatusCode::OK),
    };
    Response::builder()
        .status(status)
        .header_str_value(header::ACCESS_CONTROL_ALLOW_ORIGIN, "http://localhost:3000")?
        .body(body)
        .build()
}

//#[post("/repo", data = "<create_repo>")]
pub fn create_repository(req: &mut Request) -> Result<Response, HttpError> {
    let create_repo = CreateRepository::from_req(req)?;
    let state = GlobalState::from_req_as_ref(req)?;

    info!("#create_repository");
    let c: &CryptoActor = state.crypt();
    let option = c.create_repository(create_repo.name.as_str(), PlainPw::new(create_repo.password.as_ref()), EncryptionType::RingChachaPoly1305);
    let (body, status) = match option {
        None => ("No result...".to_string(), StatusCode::NOT_FOUND),
        Some(res) => (to_string(&res).unwrap(), StatusCode::OK),
    };


    Response::builder()
        .status(status)
        .header_str_value(header::ACCESS_CONTROL_ALLOW_ORIGIN, "http://localhost:3000")?
        .body(body)
        .build()
}

//#[post("/repo/<repo_id>", data = "<open>")]
pub fn open_repository(req: &mut Request) -> Result<Response, HttpError> {
    let open = OpenRepository::from_req(req)?;
    let repo_id = RepoId::from_req(req)?;
    let state = GlobalState::from_req_as_ref(req)?;

    info!("#open_repository");
    let c: &CryptoActor = state.crypt();
    let option = c.open_repository(&repo_id, open.user_name.clone(), PlainPw::new(open.password.as_ref()));
    let (body, status) = match option {
        None => ("No result...".to_string(), StatusCode::NOT_FOUND),
        Some(res) => (to_string(&res).unwrap(), StatusCode::OK),
    };

    Response::builder()
        .status(status)
        .header_str_value(header::ACCESS_CONTROL_ALLOW_ORIGIN, "http://localhost:3000")?
        .body(body)
        .build()
}

//#[post("/repo/<repo_id>/file", data = "<file>")]
pub fn create_file(req: &mut Request) -> Result<Response, HttpError> {
    use std::error::Error;

    let token = AccessToken::from_req(req)?;
    let file = File::from_req(req)?;
    let repo_id = RepoId::from_req(req)?;
    let state = GlobalState::from_req_as_ref(req)?;

    info!("#create_file");
    let file = file;
    let repo_id = file.repository;

    let (content, header) = file.split_header_content();
    let header = match header {
        Ok(h) => h,
        Err(e) => return Err(HttpError::bad_request(e.description()))
    };

    let c: &CryptoActor = state.crypt();
    let option = c.create_new_file(&repo_id, &token, header, content.unwrap_or(Vec::new()));
    let (body, status) = match option {
        None => ("No result...".to_string(), StatusCode::NOT_FOUND),
        Some(res) => (to_string(&res).unwrap(), StatusCode::OK),
    };

    Response::builder()
        .status(status)
        .header_str_value(header::ACCESS_CONTROL_ALLOW_ORIGIN, "http://localhost:3000")?
        .body(body)
        .build()
}

//
//#[delete("/repository/<repository_id>/<file_id>")]
//pub fn delete_file(repository_id: UUID, file_id: UUID) {}
//
//#[get("/repository/<repository_id>/<file_id>")]
//pub fn get_file(repository_id: UUID, file_id: UUID, state: State<Arc<RwLock<RepositoryState>>>) -> Result<Option<JSON<RepositoryFile>>, LockingError> {
//    get_file_header(repository_id, file_id, state)
//}
//
//#[get("/repository/<repository_id>/<file_id>/head")]
//pub fn get_file_header(repository_id: UUID, file_id: UUID, state: State<Arc<RwLock<RepositoryState>>>) -> Result<Option<JSON<RepositoryFile>>, LockingError> {
//    let s = state.read().map_err(|p| LockingError {})?;
//    let repo_option: Option<&Repository> = s.get_repository(&repository_id.into_inner());
//    let file_option = repo_option.map_or(None, |r| r.get_file_header(&file_id));
//    Ok(file_option.map(|f| JSON(f)))
//}
//
//#[post("/repository/<repository_id>/<file_id>/head")]
//pub fn save_file_header(repository_id: UUID, file_id: UUID) {}
//
//#[get("/repository/<repository_id>/<file_id>/content")]
//pub fn get_file_content(repository_id: UUID, file_id: UUID, state: State<Arc<RwLock<RepositoryState>>>) -> Result<Option<Stream<Cursor<Vec<u8>>>>, LockingError> {
//    let s = state.read().map_err(|p| LockingError {})?;
//    let repo_option: Option<&Repository> = s.get_repository(&repository_id.into_inner());
//    let content_option: Option<Vec<u8>> = repo_option.map_or(None, |r| r.get_file_content(&file_id));
//    Ok(content_option.map(|c| Stream::from(Cursor::new(c))))
//}
//
//#[post("/repository/<repository_id>/<file_id>/content")]
//pub fn save_file_content(repository_id: UUID, file_id: UUID, state: State<Arc<RwLock<RepositoryState>>>) {}


#[cfg(test)]
mod test {
    use dto::*;
    use tempdir::TempDir;
    use state::GlobalState;
    use rest_in_rust::*;
    use http::{StatusCode, Uri, Method};
    use http::request::Builder as RequestBuilder;
    use std::str::FromStr;
    use ::serde_json::to_string;

    use spectral::prelude::*;

    fn setup() -> (TempDir, ServerTester) {
        use ::rest;

        let temp = TempDir::new("rest-test").unwrap();

        let state = GlobalState::new(vec![temp.path().to_path_buf()]).unwrap();

        let mut router = Router::new();
        router.get("/rest/v1/repo", rest::list_repositories);
        router.post("/rest/v1/repo", rest::create_repository);
        router.post("/rest/v1/repo/:repo_id", rest::open_repository);
        router.post("/rest/v1/repo/:repo_id/file", rest::create_file);
        router.get("/rest/v1/repo/:repo_id", rest::list_files);
        router.get("/rest/v1/repo/:repo_id/", rest::list_files);
        router.get("/rest/v1/repo/:repo_id/:type", rest::list_files);
        router.get("/rest/v1/repo/:repo_id/:type/", rest::list_files);


        let addr = "127.0.0.1:8091".parse().unwrap();
        let s = Server::new(addr, router);
        s.add_state(state);
        let tester = s.start_testing();
        (temp, tester)
    }

    fn get_ok<'de, T>(path: &str, server: &ServerTester) -> T
        where T: ::serde::de::DeserializeOwned {
        let uri = Uri::from_str(path).unwrap();
        let request = RequestBuilder::new().uri(uri).method(Method::GET).body(().into()).unwrap();

        let response = server.handle(request);
        assert_eq!(StatusCode::OK, response.status(), "{:?}", response);
        response.body().to_json().unwrap()
    }

    fn get_from_repo<T>(suffix: &str, repo_id: &RepoId, token: &AccessToken, server: &ServerTester) -> T
        where T: ::serde::de::DeserializeOwned {
        let uri = Uri::from_str(format!("/rest/v1/repo/{}{}", repo_id, suffix).as_str()).unwrap();
        let (hv, hn) = token.to_header();
        let request = RequestBuilder::new().header(hv, hn).uri(uri).method(Method::GET).body(().into()).unwrap();

        let response = server.handle(request);
        assert_eq!(StatusCode::OK, response.status(), "{:?}", response);
        response.body().to_json().unwrap()
    }

    fn post_ok<T, B>(path: &str, body: &B, server: &ServerTester) -> T
        where T: ::serde::de::DeserializeOwned,
              B: ::serde::Serialize
    {
        use ::serde_json::to_string;
        use http::header::HeaderValue;
        use http::header;
        let s = to_string(body).unwrap();

        let uri = Uri::from_str(path).unwrap();

        let hv = HeaderValue::from_str("application/json").unwrap();
        let request = RequestBuilder::new().header(header::CONTENT_TYPE, hv).uri(uri).method(Method::POST).body(s.into()).unwrap();

        let response = server.handle(request);
        assert_eq!(StatusCode::OK, response.status(), "{:?}", response);
        response.body().to_json().unwrap()
    }

    fn post_to_repo<T, B>(suffix: &str, repo_id: &RepoId, token: &AccessToken, body: &B, server: &ServerTester) -> T
        where T: ::serde::de::DeserializeOwned,
              B: ::serde::Serialize
    {
        let (key, value) = token.to_header();
        let s = to_string(body).unwrap();
        let uri = Uri::from_str(format!("/rest/v1/repo/{}{}", repo_id, suffix).as_ref()).unwrap();

        let request = RequestBuilder::new().header(key, value).uri(uri).method(Method::POST).body(s.into()).unwrap();
        let response = server.handle(request);
        assert_eq!(StatusCode::OK, response.status(), "{:?}", response);
        response.body().to_json().unwrap()
    }

    fn create_open_repo() -> (TempDir, ServerTester, RepoId, AccessToken) {
        let (temp, server) = setup();
        let vec: Vec<RepositoryDescriptor> = get_ok("/rest/v1/repo", &server);
        assert_that(&vec).is_empty();
        let cmd = CreateRepository { name: "repo".to_string(), user_name: "none".to_string(), encryption: EncryptionType::RingChachaPoly1305, password: vec![1, 2, 3] };
        let response: Option<RepositoryDto> = post_ok("/rest/v1/repo", &cmd, &server);
        let vec: Vec<RepositoryDescriptor> = get_ok("/rest/v1/repo", &server);
        assert_that(&vec).has_length(1);
        let repo_id = vec[0].id;
        let cmd = OpenRepository { id: repo_id, user_name: "none".to_string(), password: vec![1, 2, 3] };
        let response: Option<AccessToken> = post_ok(format!("/rest/v1/repo/{}", repo_id).as_str(), &cmd, &server);
        assert!(response.is_some(), format!("{:?}", response));
        (temp, server, repo_id.clone(), response.unwrap())
    }

    #[test]
    fn good_case_create_open_close() {
        create_open_repo();
    }

    #[test]
    fn create_file_invalid_token() {
        let (temp, server, repo_id, token) = create_open_repo();

        let mut cmd = File::new(&repo_id, "test", "DOCUMENT", Some(vec![1, 2, 3, 4, 5, 6]));
        cmd.tags = vec!["hallo".to_string()];

        let file: Option<FileDescriptor> = post_to_repo("/file", &repo_id, &token, &cmd, &server);

        let other_token = AccessToken::new();

        let uri = Uri::from_str(format!("/rest/v1/repo/{}/document", &repo_id).as_str()).unwrap();
        let (hv, hn) = other_token.to_header();
        let request = RequestBuilder::new().header(hv, hn).uri(uri).method(Method::GET).body(().into()).unwrap();

        let response = server.handle(request);
        assert_eq!(StatusCode::UNAUTHORIZED, response.status(), "{:?}", response);
    }

    #[test]
    fn good_case_create_file_and_list() {
        let (temp, rocket, repo_id, token) = create_open_repo();

        let mut cmd = File::new(&repo_id, "test", "DOCUMENT", Some(vec![1, 2, 3, 4, 5, 6]));
        cmd.tags = vec!["hallo".to_string()];

        let file: Option<FileDescriptor> = post_to_repo("/file", &repo_id, &token, &cmd, &rocket);
        assert!(file.is_some());

        let page: Page = get_from_repo("/document/", &repo_id, &token, &rocket);
        assert_eq!(Some(1), page.total);
        assert_eq!(0, page.offset);
        assert_eq!(1, page.limit);
        assert_eq!(None, page.previous);
        assert_eq!(None, page.next);

        assert_eq!(1, page.files.len());

        let ref file = page.files[0];
        assert_eq!("test", file.name);
        assert_eq!("DOCUMENT", file.file_type);
        assert_eq!(None, file.content);
    }
}
