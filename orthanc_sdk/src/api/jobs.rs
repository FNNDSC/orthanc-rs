use super::response::RestResponse;
use super::types::{JobId, JobInfo};
use crate::bindings;

/// Orthanc client for the jobs API.
///
/// Ref: <https://orthanc.uclouvain.be/api/#tag/Jobs>
pub struct JobsClient(super::client::BaseClient);

impl JobsClient {
    pub fn new(context: *mut bindings::OrthancPluginContext) -> Self {
        Self(super::client::BaseClient::new(context))
    }

    /// Get an Orthanc job.
    pub fn get(&self, id: JobId) -> RestResponse<JobInfo> {
        let uri = format!("/jobs/{id}");
        self.0.get(uri)
    }
}
