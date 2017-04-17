use uuid::Uuid;
use state::{RepositoryState};
use repository::{RepositoryFile, Repository, FileType};

fn create_repo_file(repo_id: &Uuid, name: &str, content: &str) -> RepositoryFile {
    let mut f = RepositoryFile::with_name(repo_id.clone(), "hello");
    f.set_content_string("world");
    f
}

pub fn new_dummy_data() -> RepositoryState {
    let mut repo1 = Repository::with_name("Repository 1");

    let mut f1 = create_repo_file(&repo1.id, "Hallo", "Welt");
    f1.add_tag("tag1");
    f1.set_file_type(FileType::Thought);
    let mut f2 = create_repo_file(&repo1.id, "Thought", "Urks");
    f2.set_file_type(FileType::Thought);
    let mut f3 = create_repo_file(&repo1.id, "Markup with markdown", "# markdown");
    f3.set_file_type(FileType::Thought);
    let mut f4 = create_repo_file(&repo1.id, "Party", "**Sauerland**");
    f4.set_file_type(FileType::Thought);
    repo1.add_files(vec![f1, f2, f3, f4]);

    let mut repo2 = Repository::with_name("Repository 2");
    let mut f1 = create_repo_file(&repo2.id, "Test task","Content");
    f1.set_file_type(FileType::Document);
    repo2.add_file(f1);

    let mut state = RepositoryState::new();
    state.add_repos(vec![repo1, repo2]);
    state
}