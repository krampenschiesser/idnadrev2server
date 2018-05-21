use rest_in_rust::*;

use dto::EncryptionType;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CreateRepository {
    ///Name of the repository, must be unique
    pub name: String,
    ///Encryption type of the repository, will be used for all files in it
    pub encryption: EncryptionType,
    ///Password bytes
    pub password: Vec<u8>,
    ///User name, currently unused, use whatever you want
    pub user_name: String,
}
impl FromRequest for CreateRepository {
    fn from_req(req: &mut Request) -> Result<Self, HttpError> {
        req.body().to_json()
    }
}
