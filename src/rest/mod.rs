extern crate uuid;
extern crate rocket_contrib;

use uuid::Uuid;
use rocket_contrib::UUID;
use state::{RepositoryState, RepoNamesDTO};
use repository::{RepositoryFile, Repository};
use rocket::State;
use rocket::response::Stream;
use rocket_contrib::{JSON, Value};
use std::collections::HashMap;
use std::io::Cursor;
use std::clone::Clone;
use std::sync::{RwLock,Arc};
use std::thread;

pub mod ui;

#[derive(Debug)]
pub struct LockingError;

#[get("/repository")]
pub fn list_repositories(state: State<Arc<RwLock<RepositoryState>>>) -> Result<JSON<Vec<RepoNamesDTO>>, LockingError> {
    let s = state.read().map_err(|p| LockingError {})?;
    Ok(JSON(s.get_repo_names()))
}

#[post("/repository/<name>")]
pub fn create_repository(name: &str, state: State<Arc<RwLock<RepositoryState>>>) -> Result<String, LockingError> {
    let mut s = state.write().map_err(|p| LockingError {})?;
    let repo = Repository::with_name(name);
    let retval = repo.id.simple().to_string();
    s.add_repo(repo);
    Ok(retval)
}

#[post("/repository/<repository_id>/open")]
pub fn open_repository(repository_id: UUID) -> String {
    format!("{}", repository_id.into_inner().simple())
}

#[get("/repository/<repository_id>")]
pub fn list_files(repository_id: UUID, state: State<Arc<RwLock<RepositoryState>>>) -> Result<Option<JSON<Vec<RepositoryFile>>>, LockingError> {
    let s = state.read().map_err(|p| LockingError {})?;
    let option: Option<&Repository> = s.get_repository(&repository_id.into_inner());
    Ok(option.map(|r| JSON(r.get_files())))
}

#[delete("/repository/<repository_id>/<file_id>")]
pub fn delete_file(repository_id: UUID, file_id: UUID) {
}

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

    #[test]
    fn test_create_repo() {
        let lock = Arc::new(RwLock::new(RepositoryState::new()));
        let rocket = rocket::ignite()
            .manage(lock.clone())
            .mount("/", routes![super::create_repository]);

        let mut req = MockRequest::new(Post, "/repository/HalloWelt");
        let mut response = req.dispatch_with(&rocket);
        assert_eq!(response.status(), Status::Ok);

        let body_string = response.body().and_then(|b| b.into_string()).unwrap();
        let uuid = Uuid::parse_str(body_string.as_str()).unwrap();

        {
            let state = lock.read().unwrap();
            assert!(state.get_repository(&uuid).is_some());
        }
    }
}
