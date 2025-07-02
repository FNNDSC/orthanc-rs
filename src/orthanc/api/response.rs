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
    pub uri: String,
}

impl<T> PostJsonResponse<T> {
    pub fn new(uri: String, result: serde_json::Result<RestResponse<Possibly<T>>>) -> Self {
        Self { uri, result }
    }
}

#[derive(thiserror::Error, Debug)]
#[error("Error calling built-in Orthanc API from plugin at uri={uri}: {kind}")]
pub struct JsonResponseError<T> {
    pub uri: String,
    pub kind: JsonResponseErrorKind<T>,
}

#[derive(thiserror::Error, Debug)]
pub enum JsonResponseErrorKind<T> {
    #[error("cannot deserialize request body: {0}")]
    Serialization(serde_json::Error),
    #[error("cannot deserialize response body: {0}")]
    Deserialization(serde_json::Error),
    #[error("no response body")]
    NoResponse,
    #[error("unexpected JSON value: {0}")]
    UnexpectedJson(serde_json::Value),
    #[error("bad value: {reason} in data {value}")]
    BadValue { value: T, reason: &'static str },
}

impl<T: std::fmt::Debug> JsonResponseError<T> {
    pub fn trace(&self) {
        match &self.kind {
            JsonResponseErrorKind::Serialization(kind) => {
                tracing::error!(
                    uri = self.uri,
                    error = kind.to_string(),
                    "cannot serialize request body to Orthanc built-in API"
                );
            }
            JsonResponseErrorKind::Deserialization(kind) => {
                tracing::error!(
                    uri = self.uri,
                    error = kind.to_string(),
                    "cannot deserialize response from Orthanc built-in API"
                );
            }
            JsonResponseErrorKind::NoResponse => {
                tracing::error!(uri = self.uri, "no response body from Orthanc built-in API");
            }
            JsonResponseErrorKind::UnexpectedJson(value) => {
                tracing::error!(
                    uri = self.uri,
                    value = value.to_string(),
                    "unexpected response from Orthanc built-in API"
                );
            }
            JsonResponseErrorKind::BadValue { value, reason } => {
                tracing::error!(
                    uri = self.uri,
                    value = format!("{value:?}"),
                    "bad value: {reason}"
                );
            }
        };
    }
}

impl<T, R: Serialize> From<JsonResponseError<T>> for Response<R> {
    fn from(_: JsonResponseError<T>) -> Self {
        Response {
            code: StatusCode::INTERNAL_SERVER_ERROR,
            body: None,
        }
    }
}

impl<'a, T: Deserialize<'a>> PostJsonResponse<T> {
    /// Apply the given function after handling all serialization+deserialization errors.
    ///
    /// If the given function is to produce [Err], it is to return its parameter along
    /// with an error message.
    pub fn and_then<U, F: FnOnce(T) -> Result<U, (T, &'static str)>>(
        self,
        f: F,
    ) -> Result<U, JsonResponseError<T>> {
        let res = self
            .result
            .map_err(JsonResponseErrorKind::Serialization)
            .map_err(|kind| JsonResponseError {
                uri: self.uri.clone(),
                kind,
            })?;
        let possibly = unsafe { res.data() }
            .map_err(JsonResponseErrorKind::Deserialization)
            .map_err(|kind| JsonResponseError {
                uri: self.uri.clone(),
                kind,
            })?
            .ok_or_else(|| JsonResponseError {
                uri: self.uri.clone(),
                kind: JsonResponseErrorKind::NoResponse,
            })?;
        let value = match possibly {
            Possibly::Typed(value) => value,
            Possibly::Other(value) => {
                return Err(JsonResponseError {
                    uri: self.uri,
                    kind: JsonResponseErrorKind::UnexpectedJson(value),
                });
            }
        };
        f(value).map_err(|(value, reason)| JsonResponseError {
            uri: self.uri,
            kind: JsonResponseErrorKind::BadValue { value, reason },
        })
    }

    /// Obtain the `T` value, converting all errors to [JsonResponseError].
    pub fn into_result(self) -> Result<T, JsonResponseError<T>> {
        todo!()
    }
}

impl<'a, T: std::fmt::Debug + Deserialize<'a>> PostJsonResponse<T> {
    /// Return the value as [Ok], and any serialization+deserialization
    /// errors as [Err] with `code` being [StatusCode::INTERNAL_SERVER_ERROR].
    /// Additionally, errors are also reported by [JsonResponseError::trace].
    pub fn into_response_result<R: Serialize>(self) -> Result<T, Response<R>> {
        self.into_result().map_err(|e| {
            e.trace();
            Response {
                code: StatusCode::INTERNAL_SERVER_ERROR,
                body: None,
            }
        })
    }

    /// Convenience function which calls [PostJsonResponse::into_response_result],
    /// calling the given `f` if the value is [Ok], otherwise returning the [Err]
    /// value without calling `f`.
    pub fn map_into_response<S: Serialize, F: FnOnce(T) -> Response<S>>(self, f: F) -> Response<S> {
        match self.into_response_result() {
            Ok(value) => f(value),
            Err(response) => response,
        }
    }
}
