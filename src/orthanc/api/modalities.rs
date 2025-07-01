use crate::orthanc::api::response::PostJsonResponse;
use crate::orthanc::bindings;
use crate::orthanc::models::*;

/// Orthanc client for the networking API.
///
/// Ref: https://orthanc.uclouvain.be/api/#tag/Networking
pub struct ModalitiesClient(super::client::Client);

impl ModalitiesClient {
    pub fn new(context: *mut bindings::OrthancPluginContext) -> Self {
        Self(super::client::Client::new(context))
    }

    /// Start a C-MOVE SCU command as a job, in order to drive the execution
    /// of a sequence of C-STORE commands by some remote DICOM modality.
    /// Ref: https://orthanc.uclouvain.be/book/users/rest.html#performing-c-move
    pub fn c_move(
        &self,
        modality: &str,
        request: ModalitiesIdMovePostRequest,
    ) -> PostJsonResponse<ModalitiesIdGetPost200Response> {
        let url = format!("/modalities/{modality}/move");
        PostJsonResponse(self.0.post(url, request))
    }

    /// Request for some DICOM studies to be moved from a PACS to this Orthanc.
    pub fn c_move_studies(
        &self,
        modality: &str,
        study_uids: Vec<String>,
    ) -> PostJsonResponse<ModalitiesIdGetPost200Response> {
        let resources = study_uids
            .into_iter()
            .map(|u| {
                serde_json::Value::Object(
                    [("StudyInstanceUID".to_string(), serde_json::Value::String(u))]
                        .into_iter()
                        .collect(),
                )
            })
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
