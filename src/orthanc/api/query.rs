use crate::orthanc::api::response::PostJsonResponse;
use crate::orthanc::models::ModalitiesIdQueryPost200Response as MaybeQueryId;

use super::response::JsonResponseError;

/// The result of a successful query operation, after which you are able to:
///
/// - retrieve answers to the query
/// - retrieve the DICOM data found by the query
///
/// This corresponds to the group of Orthanc API endpoints under
/// [`/queries/{id}`](https://orthanc.uclouvain.be/api/#tag/Networking/paths/~1queries~1{id}/get).
pub struct OrthancQuery {
    pub id: String,
    client: super::client::Client,
}

impl OrthancQuery {
    pub(crate) fn try_new(
        client: &super::client::Client,
        response: PostJsonResponse<MaybeQueryId>,
    ) -> Result<Self, JsonResponseError<MaybeQueryId>> {
        let id = response.and_then(get_id)?;
        let client = client.clone();
        Ok(Self { id, client })
    }
}

fn get_id(value: MaybeQueryId) -> Result<String, (MaybeQueryId, &'static str)> {
    if let Some(id) = value.id {
        match id {
            serde_json::Value::String(id) => Ok(id),
            other => {
                let original = MaybeQueryId {
                    id: Some(other),
                    path: value.path,
                };
                Err((original, "id is not a string"))
            }
        }
    } else {
        let original = MaybeQueryId {
            id: None,
            path: value.path,
        };
        Err((original, "id is None"))
    }
}
