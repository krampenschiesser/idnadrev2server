
mod state;
pub mod communication;
pub mod dto;
mod function;

fn handle(cmd: CryptCmd, state: &mut State) -> Result<CryptResponse, String> {
    match &cmd {
        &CryptCmd::OpenRepository { ref id, ref pw } => open_repository(id, pw.as_slice(), state),
        &CryptCmd::CloseRepository { ref id, ref token } => close_repository(id, token, state),
        &CryptCmd::ListFiles { ref id, ref token } => list_files(id, token, state),
        &CryptCmd::ListRepositories => list_repositories(state),
        &CryptCmd::CreateNewFile { ref token, ref header, ref content, ref repo } => create_new_file(token, header, content, repo, state),
        &CryptCmd::FileAdded(ref path) => file_added(path, state),
        &CryptCmd::FileChanged(ref path) => file_changed(path, state),
        &CryptCmd::FileDeleted(ref path) => file_deleted(path, state),
        _ => Err("dooo".to_string())
    }
}
