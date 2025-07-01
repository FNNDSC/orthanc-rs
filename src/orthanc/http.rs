//! Safe Rust-friendly types for Orthanc HTTP callbacks.

// TODO HTTP headers are not handled

use super::bindings;
use serde::Serialize;
use std::ffi::CStr;

/// An HTTP response from Orthanc.
pub struct Response<S: serde::Serialize> {
    pub code: http::StatusCode,
    pub body: Option<S>,
}

impl<S: serde::Serialize> Response<S> {
    /// Create an HTTP response with a body.
    pub fn ok(body: S) -> Self {
        Self {
            code: http::StatusCode::OK,
            body: Some(body),
        }
    }

    /// Change the body.
    pub fn map_body<T: Serialize, F: FnOnce(S) -> T>(self, f: F) -> Response<T> {
        Response {
            body: self.body.map(f),
            code: self.code,
        }
    }
}

impl Response<serde_json::Value> {
    /// Produce an "internal server error" response with an error message.
    pub fn error(msg: String) -> Self {
        Self {
            code: http::StatusCode::INTERNAL_SERVER_ERROR,
            body: Some(serde_json::json!({"error": msg})),
        }
    }
}

impl<S: Serialize> From<http::StatusCode> for Response<S> {
    fn from(code: http::StatusCode) -> Self {
        Self { code, body: None }
    }
}

/// A HTTP request to Orthanc.
pub struct Request<'a, D: serde::Deserialize<'a>> {
    pub url: &'a str,
    pub body: Option<D>,
    pub method: Method,
}

impl<'a, D: serde::Deserialize<'a>> Request<'a, D> {
    /// Deserialize an HTTP request and optional JSON body as safe Rust types.
    pub(crate) unsafe fn try_new(
        url: *const std::os::raw::c_char,
        request: *const bindings::OrthancPluginHttpRequest,
    ) -> Result<Self, bindings::OrthancPluginErrorCode> {
        let method = match Method::try_from(unsafe { (*request).method }) {
            Ok(method) => method,
            Err(()) => {
                return Err(bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_BadRequest);
            }
        };

        let c_url = unsafe { CStr::from_ptr(url) };
        let url = match c_url.to_str() {
            Ok(s) => s,
            Err(_) => {
                return Err(bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_BadRequest);
            }
        };

        let body_size = unsafe { (*request).bodySize as usize };

        let body = if body_size == 0 {
            None
        } else {
            let slice = unsafe {
                let data = (*request).body as *const u8;
                std::slice::from_raw_parts(data, body_size)
            };
            match serde_json::from_slice(slice) {
                Ok(body) => body,
                Err(_e) => {
                    return Err(bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_BadJson);
                }
            }
        };
        Ok(Self { url, body, method })
    }
}

/// HTTP method
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) enum Method {
    /// HTTP GET method
    Get,
    /// HTTP POST method
    Post,
    /// HTTP PUT method
    Put,
    /// HTTP DELETE method
    Delete,
}

impl TryFrom<bindings::OrthancPluginHttpMethod> for Method {
    type Error = ();

    fn try_from(value: bindings::OrthancPluginHttpMethod) -> Result<Self, Self::Error> {
        match value {
            bindings::OrthancPluginHttpMethod_OrthancPluginHttpMethod_Get => Ok(Self::Get),
            bindings::OrthancPluginHttpMethod_OrthancPluginHttpMethod_Post => Ok(Self::Post),
            bindings::OrthancPluginHttpMethod_OrthancPluginHttpMethod_Put => Ok(Self::Put),
            bindings::OrthancPluginHttpMethod_OrthancPluginHttpMethod_Delete => Ok(Self::Delete),
            _ => Err(()),
        }
    }
}
