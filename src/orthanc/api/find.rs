use crate::orthanc::api::{PostJsonResponse, client::BaseClient};
use crate::orthanc::bindings::OrthancPluginContext;
use crate::orthanc::models::ToolsFindPostRequest;
use serde::Deserialize;

/// A trait for making requests to
/// [`/tools/find`](https://orthanc.uclouvain.be/api/#tag/System/paths/~1tools~1find/post).
///
/// The response from `/tools/find` is customized depending on request values e.g.
/// `{ "Expand": true }`, `{ "ResponseContent": [ "Children" ] }`,
/// `{ "RequestedTags": [ "PatientName", "PatientID" ] }`, etc...
/// This trait associates a request type [Into<ToolsFindPostRequest>] with a response type,
/// providing a statically type-safe method to call `/tools/find`.
pub trait Find<'a>: Into<ToolsFindPostRequest> {
    type Item: Deserialize<'a>;

    fn find(self, context: *mut OrthancPluginContext) -> PostJsonResponse<Vec<Self::Item>> {
        let client = BaseClient::new(context);
        let request = self.into();
        client.post("/tools/find".to_string(), request)
    }
}
