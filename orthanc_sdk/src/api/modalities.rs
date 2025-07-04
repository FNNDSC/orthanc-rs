use super::client::BaseClient;
use super::query::Query;
use super::response::{JsonResponseError, PostJsonResponse};
use crate::bindings;
use crate::openapi::{
    ModalitiesIdGetPost200Response, ModalitiesIdMovePostRequest, ModalitiesIdQueryPost200Response,
    ModalitiesIdQueryPostRequest,
};
use serde_json::json;

/// Orthanc client for the networking API.
///
/// Ref: <https://orthanc.uclouvain.be/api/#tag/Networking>
pub struct ModalitiesClient(BaseClient);

impl ModalitiesClient {
    pub fn new(context: *mut bindings::OrthancPluginContext) -> Self {
        Self(BaseClient::new(context))
    }

    /// List all the DICOM modalities that are known to Orthanc.
    pub fn list_modalities(&self) -> Vec<String> {
        let response = self.0.get("/modalities".to_string());
        response.unwrap()
    }

    /// Trigger C-FIND SCU command against the DICOM modality
    /// (i.e. query PACS for DICOM data).
    pub fn query_raw<M: std::fmt::Display>(
        &self,
        modality: M,
        request: ModalitiesIdQueryPostRequest,
    ) -> PostJsonResponse<ModalitiesIdQueryPost200Response> {
        let url = format!("/modalities/{modality}/query");
        self.0.post(url, request)
    }

    /// Query for a study by AccessionNumber.
    ///
    /// TODO remove this method and add one to search by multiple DICOM tags
    /// using the builder pattern instead.
    pub fn query_study<M: std::fmt::Display>(
        &self,
        modality: M,
        accession_number: String,
    ) -> Result<Query, JsonResponseError<ModalitiesIdQueryPost200Response>> {
        let request = ModalitiesIdQueryPostRequest {
            level: Some("Study".to_string()),
            query: Some(json!({"AccessionNumber": accession_number})),
            ..Default::default()
        };
        let response = self.query_raw(modality, request);
        Query::try_new(&self.0, response)
    }

    /// Start a C-MOVE SCU command as a job, in order to drive the execution
    /// of a sequence of C-STORE commands by some remote DICOM modality.
    ///
    /// Ref: <https://orthanc.uclouvain.be/book/users/rest.html#performing-c-move>
    pub fn c_move<M: std::fmt::Display>(
        &self,
        modality: M,
        request: ModalitiesIdMovePostRequest,
    ) -> PostJsonResponse<ModalitiesIdGetPost200Response> {
        let url = format!("/modalities/{modality}/move");
        self.0.post(url, request)
    }

    /// Request for some DICOM studies to be moved from a PACS to this Orthanc.
    pub fn c_move_studies(
        &self,
        modality: &str,
        study_uids: Vec<String>,
    ) -> PostJsonResponse<ModalitiesIdGetPost200Response> {
        let resources = study_uids
            .into_iter()
            .map(|u| json!({"StudyInstanceUID": u}))
            .collect();
        self.c_move(
            modality,
            ModalitiesIdMovePostRequest {
                asynchronous: Some(true),
                level: Some("Study".to_string()),
                resources: Some(resources),
                ..Default::default()
            },
        )
    }
}
