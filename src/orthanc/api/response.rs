use crate::orthanc::bindings;
use crate::orthanc::helpers::free_memory_buffer;
use crate::orthanc::http::Response;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

/// A wrapper for a response from Orthanc's REST API which implements [Drop],
/// making sure that the memory buffer is freed by
/// [OrthancPluginFreeMemoryBuffer](https://orthanc.uclouvain.be/hg/orthanc/file/Orthanc-1.12.8/OrthancServer/Plugins/Include/orthanc/OrthancCPlugin.h#l2241)
pub struct RestResponse<D> {
    /// Code returned by calling the Orthanc function.
    pub code: bindings::OrthancPluginErrorCode,
    pub uri: String,
    buffer: *mut bindings::OrthancPluginMemoryBuffer,
    context: *mut bindings::OrthancPluginContext,
    phantom: PhantomData<D>,
}

impl<D> RestResponse<D> {
    pub(crate) fn new(
        context: *mut bindings::OrthancPluginContext,
        uri: String,
        code: bindings::OrthancPluginErrorCode,
        buffer: *mut bindings::OrthancPluginMemoryBuffer,
    ) -> Self {
        Self {
            code,
            uri,
            buffer,
            context,
            phantom: Default::default(),
        }
    }
}

impl<D> Drop for RestResponse<D> {
    fn drop(&mut self) {
        unsafe { free_memory_buffer(self.context, self.buffer) }
    }
}

impl<'a, D: Deserialize<'a>> RestResponse<D> {
    /// Get the data from Orthanc's REST API response.
    ///
    /// Behind the scenes, this method reads from the memory buffer and deserializes it as JSON.
    ///
    /// # Return Values
    ///
    /// | Value         | Meaning |
    /// |---------------|------------------------------------------------------------------|
    /// | `Err(_)`      | JSON deserialization failed (note: `text/plain` not supported)   |
    /// | `Ok(None)`    | No response from Orthanc (you should check [RestResponse::code]) |
    /// | `Ok(Some(_))` | Successful response                                              |
    pub unsafe fn data(&self) -> serde_json::Result<Option<D>> {
        let size = unsafe { (*self.buffer).size as usize };
        if size == 0 {
            return Ok(None);
        }
        let slice = unsafe {
            let data = (*self.buffer).data as *const u8;
            std::slice::from_raw_parts(data, size)
        };
        serde_json::from_slice(slice).map(Some)
    }

    /// Returns the `OK(Some(_))` value. This function may panic.
    pub unsafe fn unwrap(&self) -> D {
        unsafe { self.data() }.unwrap().unwrap()
    }
}

/// Data which _might_ be typed.
#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum Possibly<T> {
    /// Typed data
    Typed(T),
    /// Untyped data (developer is too lazy to define
    /// types for all possible responses from Orthanc).
    Other(serde_json::Value),
}

impl<T> Possibly<T> {
    /// Borrow the typed value
    pub fn typed(&self) -> Option<&T> {
        if let Self::Typed(x) = &self {
            Some(x)
        } else {
            None
        }
    }

    /// Obtain the typed value (or drop if untyped)
    pub fn into_typed(self) -> Option<T> {
        if let Self::Typed(x) = self {
            Some(x)
        } else {
            None
        }
    }
}

/// Response from sending a POST request with body.
pub struct PostJsonResponse<T> {
    pub result: serde_json::Result<RestResponse<Possibly<T>>>,
    pub uri: String
}

impl<T> PostJsonResponse<T> {
    pub fn new(uri: String, result: serde_json::Result<RestResponse<Possibly<T>>>) -> Self {
        Self { uri, result }
    }
}

impl<'a, T: Deserialize<'a>> PostJsonResponse<T> {
    /// Produce a response assuming all has gone well. Any serialization+deserialization
    /// errors are responded to with [StatusCode::INTERNAL_SERVER_ERROR].
    pub fn map<S: Serialize, F: FnOnce(T) -> Response<S>>(self, f: F) -> Response<S> {
        let res = match self.result {
            Ok(res) => res,
            Err(e) => {
                tracing::error!(
                    error = e.to_string(),
                    uri = self.uri,
                    "Could not serialize request"
                );
                return Response {
                    code: StatusCode::INTERNAL_SERVER_ERROR,
                    body: None,
                };
            }
        };
        let maybe = unsafe {
            match res.data() {
                Ok(data) => data,
                Err(e) => {
                    tracing::error!(
                        error = e.to_string(),
                        uri = self.uri,
                        "Could not deserialize response"
                    );
                    return Response {
                        code: StatusCode::INTERNAL_SERVER_ERROR,
                        body: None,
                    };
                }
            }
        };
        let data = if let Some(data) = maybe {
            data
        } else {
            tracing::error!(uri = self.uri, "No response");
            return Response {
                code: StatusCode::INTERNAL_SERVER_ERROR,
                body: None,
            };
        };
        let value = match data {
            Possibly::Typed(value) => value,
            Possibly::Other(e) => {
                tracing::error!(
                    value = e.to_string(),
                    uri = self.uri,
                    "Unexpected JSON"
                );
                return Response {
                    code: StatusCode::INTERNAL_SERVER_ERROR,
                    body: None,
                };
            }
        };
        f(value)
    }
}
