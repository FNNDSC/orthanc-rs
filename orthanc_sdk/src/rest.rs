//! REST related helper functions.

use crate::bindings;
use crate::error_code::*;
use crate::http::{Request, Response};
use crate::sdk::send_http_status_code;
use crate::sdk::{answer_buffer, send_http_status};
use http::StatusCode;
use std::ffi::CStr;

/// Create an Orthanc REST callback that uses JSON in its request and response bodies.
pub fn create_json_rest_callback<
    'a,
    S: serde::Serialize,
    D: serde::Deserialize<'a>,
    R: Into<Response<S>>,
    F: FnOnce(Request<D>) -> R,
>(
    context: *mut bindings::OrthancPluginContext,
    output: *mut bindings::OrthancPluginRestOutput,
    url: *const std::os::raw::c_char,
    request: *const bindings::OrthancPluginHttpRequest,
    handle: F,
) -> bindings::OrthancPluginErrorCode {
    let req = match unsafe { Request::try_new(url, request) } {
        Ok(req) => req,
        Err(e) => {
            return e;
        }
    };
    let res = handle(req).into();

    if let Some(body) = &res.body {
        let body = match serde_json::to_vec(body) {
            Ok(body) => body,
            Err(_e) => {
                return bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_InternalError;
            }
        };
        respond_with_body(context, output, res.code, body, c"application/json")
    } else {
        respond_no_body(context, output, res.code)
    }
}

/// Respond to an HTTP request with a body.
///
/// Note: this function handles the "must use" requirements of Orthanc. See
/// <https://orthanc.uclouvain.be/sdk/group__REST.html#gadc077803cf6cfc5306491097f9063627>
fn respond_with_body(
    context: *mut bindings::OrthancPluginContext,
    output: *mut bindings::OrthancPluginRestOutput,
    code: StatusCode,
    body: Vec<u8>,
    mime_type: &CStr,
) -> bindings::OrthancPluginErrorCode {
    match code {
        StatusCode::OK => answer_buffer(context, output, &body, mime_type),
        StatusCode::MOVED_PERMANENTLY => {
            // TODO must use ::OrthancPluginRedirect()
            bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_NotImplemented
        }
        StatusCode::UNAUTHORIZED => {
            // TODO must use ::OrthancPluginSendUnauthorized()
            bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_NotImplemented
        }
        StatusCode::METHOD_NOT_ALLOWED => {
            // TODO must use ::OrthancPluginSendMethodNotAllowed()
            bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_NotImplemented
        }
        StatusCode::NOT_ACCEPTABLE => send_http_status(context, output, code.as_u16(), body)
            .map_ok(bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_NotAcceptable),
        StatusCode::NOT_IMPLEMENTED => send_http_status(context, output, code.as_u16(), body)
            .map_ok(bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_NotImplemented),
        StatusCode::BAD_REQUEST => send_http_status(context, output, code.as_u16(), body)
            .map_ok(bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_BadRequest),
        // note: OrthancPluginErrorCode_Timeout is *not* used for codes 408 nor 504
        _ => send_http_status(context, output, code.as_u16(), body),
    }
}

/// Respond to an HTTP request without a body.
///
/// Note: this function handles the "must use" logic required by Orthanc. See
/// <https://orthanc.uclouvain.be/sdk/group__REST.html#ga61be84f0a8886c6c350b20055f97ddc5>
fn respond_no_body(
    context: *mut bindings::OrthancPluginContext,
    output: *mut bindings::OrthancPluginRestOutput,
    code: StatusCode,
) -> bindings::OrthancPluginErrorCode {
    match code {
        StatusCode::OK => {
            answer_buffer(context, output, &[], c"text/plain");
            bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_Success
        }
        StatusCode::MOVED_PERMANENTLY => {
            // TODO must use ::OrthancPluginRedirect()
            bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_NotImplemented
        }
        StatusCode::UNAUTHORIZED => {
            // TODO must use ::OrthancPluginSendUnauthorized()
            bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_NotImplemented
        }
        StatusCode::METHOD_NOT_ALLOWED => {
            // TODO must use ::OrthancPluginSendMethodNotAllowed()
            bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_NotImplemented
        }
        StatusCode::NOT_ACCEPTABLE => {
            bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_NotAcceptable
        }
        StatusCode::NOT_IMPLEMENTED => {
            bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_NotImplemented
        }
        StatusCode::NOT_FOUND => {
            bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_UnknownResource
        }
        StatusCode::BAD_REQUEST => {
            bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_BadRequest
        }
        // note: OrthancPluginErrorCode_Timeout is *not* used for codes 408 nor 504
        _ => send_http_status_code(context, output, code.as_u16()),
    }
}
