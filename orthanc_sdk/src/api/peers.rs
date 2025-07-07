use crate::{bindings, openapi::PeersIdStorePostRequest as StoreRequest};
use orthanc_api::{IdAndPath, JobId};
use serde::de::DeserializeOwned;

use super::{PostJsonResponse, client::BaseClient};

/// Orthanc client for the peers API.
///
/// Ref: <https://orthanc.uclouvain.be/api/#tag/Networking/paths/~1peers~1{id}/get>
pub struct PeersClient(BaseClient);

impl PeersClient {
    pub fn new(context: *mut bindings::OrthancPluginContext) -> Self {
        Self(BaseClient::new(context))
    }

    /// List Orthanc peers.
    pub fn list(&self) -> Vec<String> {
        let response = self.0.get("/peers".to_string());
        response.unwrap()
    }

    /// Send DICOM resources stored locally to some remote Orthanc peer.
    pub fn store_request<P: std::fmt::Display, T: DeserializeOwned>(
        &self,
        peer: P,
        request: StoreRequest,
    ) -> PostJsonResponse<T> {
        let url = format!("/peers/{peer}/store");
        self.0.post(url, request)
    }

    /// Enqueue a job to send DICOM resources stored locally to some remote Orthanc peer.
    pub fn store<P: std::fmt::Display>(
        &self,
        peer: P,
        mut request: StoreRequest,
    ) -> PostJsonResponse<IdAndPath<JobId>> {
        request.asynchronous = Some(true);
        self.store_request(peer, request)
    }
}
