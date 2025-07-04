use super::answers::Answers;
use super::client::BaseClient;
use super::response::{JsonResponseError, PostJsonResponse};

use crate::openapi::{
    ModalitiesIdQueryPost200Response as MaybeQueryId,
    QueriesIdAnswersIndexRetrievePostRequest as RetrieveRequest,
};
use orthanc_api::{IdAndPath, JobId, QueryId};

/// The result of a successful query operation, after which you are able to:
///
/// - retrieve answers to the query
/// - retrieve the DICOM data found by the query
///
/// This corresponds to the group of Orthanc API endpoints under
/// [`/queries/{id}`](https://orthanc.uclouvain.be/api/#tag/Networking/paths/~1queries~1{id}/get).
pub struct Query {
    pub id: QueryId,
    path: String,
    client: BaseClient,
}

impl Query {
    pub fn try_new(
        client: &BaseClient,
        response: PostJsonResponse<MaybeQueryId>,
    ) -> Result<Self, JsonResponseError<MaybeQueryId>> {
        let (id, path) = response.and_then(must_get)?;
        let client = client.clone();
        Ok(Self { id, path, client })
    }

    /// Get the answers to this query.
    pub fn answers(&self) -> Result<Answers, JsonResponseError<Vec<compact_str::CompactString>>> {
        let url = format!("{}/answers", &self.path);
        let answers = self.client.get(url).data()?;
        Ok(Answers::new(
            self.client.clone(),
            self.path.clone(),
            answers,
        ))
    }

    /// Retrieve all the answers associated with this query/retrieve operation.
    ///
    /// Corresponds with [`/queries/{id}}/retrieve`](https://orthanc.uclouvain.be/api/#tag/Networking/paths/~1queries~1{id}~1retrieve/post).
    pub fn retrieve_raw<'a, T: serde::Deserialize<'a>>(
        &self,
        request: RetrieveRequest,
    ) -> PostJsonResponse<T> {
        let uri = format!("{}/retrieve", &self.path);
        self.client.post(uri, request)
    }

    /// Retrieve all the answers associated with this query/retrieve operation
    /// in an asynchronous job.
    pub fn request_retrieve_job(&self) -> PostJsonResponse<IdAndPath<JobId>> {
        let request = RetrieveRequest {
            asynchronous: Some(true),
            ..Default::default()
        };
        self.retrieve_raw(request)
    }
}

/// Get `id` and `path` as required strings.
fn must_get(value: MaybeQueryId) -> Result<(QueryId, String), (MaybeQueryId, &'static str)> {
    // needlessly ugly and efficient implementation
    if let Some(id) = value.id {
        match id {
            serde_json::Value::String(id) => {
                if let Some(path) = value.path {
                    match path {
                        serde_json::Value::String(path) => Ok((QueryId::new(id), path)),
                        other => {
                            let original = MaybeQueryId {
                                id: Some(serde_json::Value::String(id)),
                                path: Some(other),
                            };
                            Err((original, "id is not a string"))
                        }
                    }
                } else {
                    let original = MaybeQueryId {
                        id: Some(serde_json::Value::String(id)),
                        path: None,
                    };
                    Err((original, "path is None"))
                }
            }
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
