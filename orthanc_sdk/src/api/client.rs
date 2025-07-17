use super::response::{PostJsonResponse, RestResponse};
use crate::bindings;
use crate::helpers::{create_empty_buffer, free_memory_buffer, invoke_service};
use serde::{Deserialize, Serialize};
use std::ffi::CString;

/// Methods for calling the built-in API of Orthanc from a plugin.
#[derive(Copy, Clone)]
pub struct BaseClient {
    context: *mut bindings::OrthancPluginContext,
}

impl BaseClient {
    /// Create a [BaseClient].
    pub fn new(context: *mut bindings::OrthancPluginContext) -> Self {
        Self { context }
    }

    /// Make a GET call to the built-in Orthanc REST API.
    ///
    /// Wrapper for [`OrthancPluginRestApiGet`](https://orthanc.uclouvain.be/sdk/group__Orthanc.html#ga9fdcf0181b1f0a18c5e4c9fa2dd71cc4)
    pub fn get<'a, D: Deserialize<'a>>(&self, uri: String) -> RestResponse<D> {
        let context = self.context;
        let c_uri = CString::new(uri.as_str()).unwrap();
        let target = create_empty_buffer();
        let params = bindings::_OrthancPluginRestApiGet {
            target,
            uri: c_uri.as_ptr(),
        };
        let code = invoke_service(
            context,
            bindings::_OrthancPluginService__OrthancPluginService_RestApiGet,
            params,
        );
        RestResponse::new(context, uri, code, target)
    }

    /// Make a DELETE call to the built-in Orthanc REST API.
    ///
    /// Wrapper for [`OrthancPluginRestApiDelete`](https://orthanc.uclouvain.be/sdk/group__Orthanc.html#gadd36e54c43f6371c59301b8b257e3eee)
    pub fn delete(&self, uri: String) -> bindings::OrthancPluginErrorCode {
        let context = self.context;
        let c_uri = CString::new(uri.as_str()).unwrap();
        let service = bindings::_OrthancPluginService__OrthancPluginService_RestApiDelete;
        unsafe {
            let invoker = (*context).InvokeService;
            invoker.unwrap()(context, service, c_uri.as_ptr() as *const std::ffi::c_void)
        }
    }

    /// Make a DELETE call to the built-in Orthanc REST API and get its JSON response.
    ///
    /// **Requires Orthanc 1.12.9** or later.
    /// See <https://discourse.orthanc-server.org/t/response-to-plugin-from-orthanc-api-delete-endpoint/6022>
    pub fn delete_with_response<'a, D: Deserialize<'a>>(&self, uri: String) -> RestResponse<D> {
        tracing::warn!(
            "It seems like Orthanc never responds with a body when DELETE is called from a plugin."
        );
        let context = self.context;
        let c_uri = CString::new(uri.as_str()).unwrap();
        let answer_body = create_empty_buffer();
        let answer_headers = create_empty_buffer();
        let http_status = Box::new(0u16);
        let params = bindings::_OrthancPluginCallRestApi {
            answerBody: answer_body,
            answerHeaders: answer_headers,
            httpStatus: Box::into_raw(http_status),
            method: bindings::OrthancPluginHttpMethod_OrthancPluginHttpMethod_Delete,
            uri: c_uri.as_ptr(),
            headersCount: 0,
            headersKeys: std::ptr::null(),
            headersValues: std::ptr::null(),
            body: std::ptr::null(),
            bodySize: 0,
            afterPlugins: 0,
        };
        let code = invoke_service(
            context,
            bindings::_OrthancPluginService__OrthancPluginService_CallRestApi,
            params,
        );
        let http_status = unsafe {
            // headers not handled at the moment
            free_memory_buffer(context, answer_headers);
            Box::from_raw(params.httpStatus)
        };
        // dbg!(*http_status);
        // dbg!(code);
        // dbg!(unsafe {*answer_body}.size );
        RestResponse::new(context, uri, code, answer_body).with_status(*http_status)
    }

    /// Make a POST call to the built-in Orthanc REST API.
    ///
    /// Wrapper for [`OrthancPluginRestApiPost`](https://orthanc.uclouvain.be/sdk/group__Orthanc.html#ga03e733e9fb437f98700ba99881c37642)
    pub fn post<'a, D: Deserialize<'a>, B: Serialize>(
        &self,
        uri: String,
        body: B,
    ) -> PostJsonResponse<D> {
        let body = match serde_json::to_vec(&body) {
            Ok(body) => body,
            Err(e) => {
                return PostJsonResponse::new(uri, Err(e));
            }
        };
        let context = self.context;
        let c_uri = CString::new(uri.as_str()).unwrap();
        let target = create_empty_buffer();
        let params = bindings::_OrthancPluginRestApiPostPut {
            target,
            uri: c_uri.as_ptr(),
            body: body.as_ptr() as *const _,
            bodySize: body.len() as u32,
        };
        let code = invoke_service(
            context,
            bindings::_OrthancPluginService__OrthancPluginService_RestApiPost,
            params,
        );
        let res = RestResponse::new(context, uri.clone(), code, target);
        PostJsonResponse::new(uri, Ok(res))
    }
}
