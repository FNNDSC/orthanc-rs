use crate::orthanc::api::response::RestResponse;
use crate::orthanc::bindings;
use crate::orthanc::helpers::{create_empty_buffer, invoke_service};
use serde::{Deserialize, Serialize};
use std::ffi::CString;

/// Safe-rust wrappers for
/// [`OrthancPluginCallRestApi`](https://orthanc.uclouvain.be/sdk/group__Orthanc.html#gae1588d25e5686eb4d2d6ee7946db2730)
pub struct Client {
    context: *mut bindings::OrthancPluginContext,
}

impl Client {
    /// Create a [Client].
    pub fn new(context: *mut bindings::OrthancPluginContext) -> Self {
        Self { context }
    }

    /// Wrapper for [`OrthancPluginRestApiPost`](https://orthanc.uclouvain.be/sdk/group__Orthanc.html#ga03e733e9fb437f98700ba99881c37642)
    pub(crate) fn post<'a, D: Deserialize<'a>, B: Serialize, S: AsRef<str>>(
        &self,
        uri: S,
        body: B,
    ) -> serde_json::Result<RestResponse<D>> {
        let body = serde_json::to_vec(&body)?;
        let context = self.context;
        let uri = CString::new(uri.as_ref()).unwrap();
        let target = create_empty_buffer(); // TODO does this work?
        let params = bindings::_OrthancPluginRestApiPostPut {
            target,
            uri: uri.as_ptr(),
            body: body.as_ptr() as *const _,
            bodySize: body.len() as u32,
        };
        let code = invoke_service(
            context,
            bindings::_OrthancPluginService__OrthancPluginService_RestApiPost,
            params,
        );
        let res = RestResponse::new(context, code, target);
        Ok(res)
    }
}
