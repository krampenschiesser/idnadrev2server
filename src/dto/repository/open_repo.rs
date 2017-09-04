use rest_in_rust::*;
use super::RepoId;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct OpenRepository {
    ///ID of the repository to open
    pub id: RepoId,
    ///Password to use for open
    pub password: Vec<u8>,
    ///Username to use for open
    pub user_name: String,
}

impl FromRequest for OpenRepository {
    fn from_req(req: &mut Request) -> Result<Self, HttpError> {
        req.body().to_json()
    }
}
