//! Orthanc plugin initialization callback registration functions.

use super::bindings;
use super::helpers::must_invoke_service;
use super::http::{Request, Response};
use http::StatusCode;
use std::ffi::CString;
use std::str::FromStr;

/// Translated from `OrthancPluginRegisterOnChangeCallback`.
pub(crate) fn register_on_change(
    context: *mut bindings::OrthancPluginContext,
    callback: bindings::OrthancPluginOnChangeCallback,
) {
    let params = bindings::_OrthancPluginOnChangeCallback { callback };
    must_invoke_service(
        context,
        bindings::_OrthancPluginService__OrthancPluginService_RegisterOnChangeCallback,
        params,
    )
}

/// Register a REST callback.
///
/// Translated from [OrthancPluginRegisterRestCallback](https://orthanc.uclouvain.be/hg/orthanc/file/Orthanc-1.12.8/OrthancServer/Plugins/Include/orthanc/OrthancCPlugin.h#l2341)
pub fn register_rest(
    context: *mut bindings::OrthancPluginContext,
    path_regex: &str,
    callback: bindings::OrthancPluginRestCallback,
) {
    let path_regex_c = CString::new(path_regex).unwrap();
    let params = bindings::_OrthancPluginRestCallback {
        pathRegularExpression: path_regex_c.as_ptr(),
        callback,
    };
    must_invoke_service(
        context,
        bindings::_OrthancPluginService__OrthancPluginService_RegisterRestCallback,
        params,
    )
}

/// Register a REST callback, without locking.
///
/// Translated from [OrthancPluginRegisterRestCallbackNoLock](https://orthanc.uclouvain.be/hg/orthanc/file/Orthanc-1.12.8/OrthancServer/Plugins/Include/orthanc/OrthancCPlugin.h#l2381).
pub fn register_rest_no_lock(
    context: *mut bindings::OrthancPluginContext,
    path_regex: &str,
    callback: bindings::OrthancPluginRestCallback,
) {
    let path_regex_c = CString::new(path_regex).unwrap();
    let params = bindings::_OrthancPluginRestCallback {
        pathRegularExpression: path_regex_c.as_ptr(),
        callback,
    };
    must_invoke_service(
        context,
        bindings::_OrthancPluginService__OrthancPluginService_RegisterRestCallbackNoLock,
        params,
    )
}

/// Create an Orthanc REST callback that uses JSON in its request and response bodies.
pub(crate) fn create_json_rest_callback<
    'a,
    S: serde::Serialize,
    D: serde::Deserialize<'a>,
    F: FnOnce(Request<D>) -> Response<S>,
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
    let res = handle(req);

    if let Some(body) = &res.body {
        let body = match serde_json::to_vec(body) {
            Ok(body) => body,
            Err(_e) => {
                return bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_InternalError;
            }
        };
        let mime_type = CString::new("application/json").unwrap();
        respond_with_body(context, output, res.code, body, mime_type)
    } else {
        respond_no_body(context, output, res.code)
    }
}

/// Respond to an HTTP request with a body.
///
/// Note: this function handles the "must use" requirements of Orthanc. See
/// https://orthanc.uclouvain.be/sdk/group__REST.html#gadc077803cf6cfc5306491097f9063627
fn respond_with_body(
    context: *mut bindings::OrthancPluginContext,
    output: *mut bindings::OrthancPluginRestOutput,
    code: StatusCode,
    body: Vec<u8>,
    mime_type: CString,
) -> bindings::OrthancPluginErrorCode {
    match code {
        StatusCode::OK => {
            answer_buffer(context, output, body, mime_type);
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
            send_http_status_code(context, output, code);
            bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_NotAcceptable
        }
        StatusCode::NOT_IMPLEMENTED => {
            send_http_status_code(context, output, code);
            bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_NotImplemented
        }
        StatusCode::BAD_REQUEST => {
            send_http_status_code(context, output, code);
            bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_BadRequest
        }
        // note: OrthancPluginErrorCode_Timeout is *not* used for codes 408 nor 504
        _ => {
            send_http_status_code(context, output, code);
            bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_Success
        }
    }
}

/// Respond to an HTTP request without a body.
///
/// Note: this function handles the "must use" logic required by Orthanc. See
/// https://orthanc.uclouvain.be/sdk/group__REST.html#ga61be84f0a8886c6c350b20055f97ddc5
fn respond_no_body(
    context: *mut bindings::OrthancPluginContext,
    output: *mut bindings::OrthancPluginRestOutput,
    code: StatusCode,
) -> bindings::OrthancPluginErrorCode {
    match code {
        StatusCode::OK => {
            answer_buffer(
                context,
                output,
                Vec::new(),
                CString::from_str("text/plain").unwrap(),
            );
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
        _ => {
            send_http_status_code(context, output, code);
            bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_Success
        }
    }
}

/// Answer to a REST request. Translated from `OrthancPluginAnswerBuffer`.
fn answer_buffer(
    context: *mut bindings::OrthancPluginContext,
    output: *mut bindings::OrthancPluginRestOutput,
    body: Vec<u8>,
    mime_type: CString,
) {
    let params = bindings::_OrthancPluginAnswerBuffer {
        output,
        answer: body.as_ptr() as *const _,
        answerSize: body.len() as u32,
        mimeType: mime_type.as_ptr(),
    };
    must_invoke_service(
        context,
        bindings::_OrthancPluginService__OrthancPluginService_AnswerBuffer,
        params,
    )
}

/// Send an HTTP status, with a custom body.
///
/// Translated from `OrthancPluginSendHttpStatus`.
fn send_http_status<S: serde::Serialize>(
    context: *mut bindings::OrthancPluginContext,
    output: *mut bindings::OrthancPluginRestOutput,
    code: StatusCode,
    body: Vec<u8>,
) {
    let params = bindings::_OrthancPluginSendHttpStatus {
        output,
        status: code.as_u16(),
        body: body.as_ptr() as *const _,
        bodySize: body.len() as u32,
    };
    must_invoke_service(
        context,
        bindings::_OrthancPluginService__OrthancPluginService_SendHttpStatus,
        params,
    )
}

/// Send an HTTP status.
///
/// Translated from `OrthancPluginSendHttpStatusCode`.
fn send_http_status_code(
    context: *mut bindings::OrthancPluginContext,
    output: *mut bindings::OrthancPluginRestOutput,
    code: StatusCode,
) {
    let params = bindings::_OrthancPluginSendHttpStatusCode {
        output,
        status: code.as_u16(),
    };
    must_invoke_service(
        context,
        bindings::_OrthancPluginService__OrthancPluginService_SendHttpStatusCode,
        params,
    )
}
