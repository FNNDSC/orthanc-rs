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
    code: bindings::OrthancPluginErrorCode,
    buffer: *mut bindings::OrthancPluginMemoryBuffer,
    context: *mut bindings::OrthancPluginContext,
    phantom: PhantomData<D>,
}

impl<D> RestResponse<D> {
    pub(crate) fn new(
        context: *mut bindings::OrthancPluginContext,
        code: bindings::OrthancPluginErrorCode,
        buffer: *mut bindings::OrthancPluginMemoryBuffer,
    ) -> Self {
        Self {
            code,
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
    unsafe fn data(&self) -> serde_json::Result<Option<D>> {
        let size = unsafe { (*self.buffer).size as usize };
        if size == 0 {
            return Ok(None);
        }
        let slice = unsafe {
            let data = (*self.buffer).data as *const u8;
            dbg!(size);
            std::slice::from_raw_parts(data, size)
        };
        dbg!("i got the slice!");
        serde_json::from_slice(slice).map(Some)
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
pub struct PostJsonResponse<T>(pub serde_json::Result<RestResponse<Possibly<T>>>);

impl<'a, T: Deserialize<'a>> PostJsonResponse<T> {
    /// Produce a response assuming all has gone well. Any serialization+deserialization
    /// errors are responded to with [StatusCode::INTERNAL_SERVER_ERROR].
    pub fn map<S: Serialize, F: FnOnce(T) -> Response<S>>(self, f: F) -> Response<S> {
        // NOTE: the error messages emitted by this function are not technically accurate.
        // We have not actually called OrthancPluginCallRestApi(), nor was it any of
        // OrthancPluginRestApiGet2(), OrthancPluginRestApiPost(), OrthancPluginRestApiPut(),
        // OrthancPluginRestApiDelete()... regardless, I still think it's still a meaningful
        // way to word this (rare) error message.
        let res = match self.0 {
            Ok(res) => res,
            Err(e) => {
                tracing::error!(
                    error = e.to_string(),
                    "Could not serialize request to OrthancPluginCallRestApi"
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
                        "Could not deserialize response from OrthancPluginCallRestApi"
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
            tracing::error!("No response from OrthancPluginCallRestApi");
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
                    "Unexpected JSON from OrthancPluginCallRestApi"
                );
                return Response {
                    code: StatusCode::INTERNAL_SERVER_ERROR,
                    body: None,
                };
            }
        };
        f(value)
    }
    // /// See [PostJsonResponse::map].
    // pub fn map_possibly<S: Serialize, F: FnOnce(T) -> Response<S>>(self, f: F) -> Response<Possibly<S>> {
    //     let response = self.map(f);
    //     Response {
    //         body: response.body.map(Possibly::Typed),
    //         code: response.code
    //     }
    // }
}
