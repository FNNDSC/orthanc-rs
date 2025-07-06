use super::client::BaseClient;
use crate::api::{PostJsonResponse, RestResponse};
use crate::openapi::ToolsFindPostRequest;
use orthanc_api::{DeleteResponse, DicomResource, DicomResourceId, RequestedTags};
use serde::de::DeserializeOwned;

/// A client for getting DICOM resources (patient, study, series, or instance)
/// from Orthanc's built-in API.
#[derive(Copy, Clone)]
pub struct DicomClient(BaseClient);

impl DicomClient {
    /// Create a client for the plugin.
    pub fn new(context: *mut crate::bindings::OrthancPluginContext) -> Self {
        Self(BaseClient::new(context))
    }

    /// Get a DICOM resource stored by Orthanc.
    pub fn get<
        T: RequestedTags + DeserializeOwned,
        U: DicomResource<T> + DeserializeOwned,
        I: DicomResourceId<U>,
    >(
        &self,
        id: I,
    ) -> RestResponse<U> {
        let requested_tags = T::names().join(";");
        let uri = format!("{}?requested-tags={}", id.uri(), requested_tags);
        self.0.get(uri)
    }

    /// Delete a DICOM resource from Orthanc.
    ///
    /// FIXME: do not use [DicomClient::delete] because it does not work, use
    ///        [crate::api::GeneralClient::delete] instead. Even though the
    ///        Orthanc REST API returns a body with "RemainingAncestor" when
    ///        deleting a patient, study, series, or instance from an HTTP client,
    ///        it seems like the Orthanc built-in API never returns a body when
    ///        DELETE is called from a plugin.
    pub fn delete<A: DeserializeOwned, I: DicomResourceId<Option<()>, Ancestor = A>>(
        &self,
        id: I,
    ) -> RestResponse<DeleteResponse<A>> {
        self.0.delete_with_response(id.uri())
    }

    /// Search DICOM content by calling
    /// [`/tools/find`](https://orthanc.uclouvain.be/api/#tag/System/paths/~1tools~1find/post).
    pub fn find<T: DeserializeOwned, R: Find<Item = T>>(
        &self,
        request: R,
    ) -> PostJsonResponse<Vec<T>> {
        self.0.post("/tools/find".to_string(), request.into())
    }
}

/// Parameters for
/// [`/tools/find`](https://orthanc.uclouvain.be/api/#tag/System/paths/~1tools~1find/post),
/// which is called by [DicomClient::find].
pub trait Find: Into<ToolsFindPostRequest> {
    type Item: DeserializeOwned;
}
