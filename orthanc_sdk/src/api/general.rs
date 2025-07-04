use crate::api::RestResponse;
use crate::api::client::BaseClient;
use crate::bindings::{OrthancPluginContext, OrthancPluginErrorCode};
use orthanc_api::ResourceId;
use serde::de::DeserializeOwned;

/// A client for the Orthanc built-in API.
///
/// Hint: [GeneralClient] should be used to get non-DICOM resources in Orthanc
/// such as jobs and queries. To get DICOM resources, [crate::api::DicomClient]
/// has similar methods but with DICOM-specific features such as specifying
/// `"RequestedDicomTags"`.
#[derive(Copy, Clone)]
pub struct GeneralClient(BaseClient);

impl GeneralClient {
    /// Create a client for the plugin.
    pub fn new(context: *mut OrthancPluginContext) -> Self {
        Self(BaseClient::new(context))
    }

    /// Get an Orthanc resource.
    pub fn get<T: DeserializeOwned, I: ResourceId<Item = T>>(&self, id: I) -> RestResponse<T> {
        self.0.get(id.uri())
    }

    /// Delete an Orthanc resource.
    pub fn delete<I: ResourceId>(&self, id: I) -> OrthancPluginErrorCode {
        self.0.delete(id.uri())
    }
}
