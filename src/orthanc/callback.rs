//! Orthanc plugin initialization callback registration functions.

use crate::orthanc::http::{Request, Response};
use http::StatusCode;
use std::ffi::CString;
use std::str::FromStr;

/// Translated from `OrthancPluginRegisterOnChangeCallback`.
pub(crate) fn register_on_change(
    context: *mut super::OrthancPluginContext,
    callback: super::OrthancPluginOnChangeCallback,
) {
    let params = Box::new(super::_OrthancPluginOnChangeCallback { callback });
    let params: *const std::ffi::c_void = Box::into_raw(params) as *mut std::ffi::c_void;
    unsafe {
        let invoker = (*context).InvokeService;
        invoker.unwrap()(
            context,
            super::_OrthancPluginService__OrthancPluginService_RegisterOnChangeCallback,
            params,
        );
    }
}

/// Translated from `OrthancPluginRegisterRestCallbackNoLock`.
pub(crate) fn register_rest_no_lock(
    context: *mut super::OrthancPluginContext,
    path_regex: &str,
    callback: super::OrthancPluginRestCallback,
) {
    let path_regex_c = CString::new(path_regex).unwrap();
    let params = Box::new(super::_OrthancPluginRestCallback {
        pathRegularExpression: path_regex_c.as_ptr(),
        callback,
    });
    let params: *const std::ffi::c_void = Box::into_raw(params) as *mut std::ffi::c_void;
    unsafe {
        let invoker = (*context).InvokeService;
        invoker.unwrap()(
            context,
            super::_OrthancPluginService__OrthancPluginService_RegisterRestCallbackNoLock,
            params,
        );
    }
}

/// Create an Orthanc REST callback that uses JSON in its request and response bodies.
pub(crate) fn create_json_rest_callback<
    'a,
    S: serde::Serialize,
    D: serde::Deserialize<'a>,
    F: FnOnce(Request<D>) -> Response<S>,
>(
    context: *mut super::OrthancPluginContext,
    output: *mut super::OrthancPluginRestOutput,
    url: *const std::os::raw::c_char,
    request: *const super::OrthancPluginHttpRequest,
    handle: F,
) -> super::OrthancPluginErrorCode {
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
                return super::OrthancPluginErrorCode_OrthancPluginErrorCode_InternalError;
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
    context: *mut super::OrthancPluginContext,
    output: *mut super::OrthancPluginRestOutput,
    code: StatusCode,
    body: Vec<u8>,
    mime_type: CString,
) -> super::OrthancPluginErrorCode {
    match code {
        StatusCode::OK => {
            answer_buffer(context, output, body, mime_type);
            super::OrthancPluginErrorCode_OrthancPluginErrorCode_Success
        }
        StatusCode::MOVED_PERMANENTLY => {
            // TODO must use ::OrthancPluginRedirect()
            super::OrthancPluginErrorCode_OrthancPluginErrorCode_NotImplemented
        }
        StatusCode::UNAUTHORIZED => {
            // TODO must use ::OrthancPluginSendUnauthorized()
            super::OrthancPluginErrorCode_OrthancPluginErrorCode_NotImplemented
        }
        StatusCode::METHOD_NOT_ALLOWED => {
            // TODO must use ::OrthancPluginSendMethodNotAllowed()
            super::OrthancPluginErrorCode_OrthancPluginErrorCode_NotImplemented
        }
        StatusCode::NOT_ACCEPTABLE => {
            send_http_status_code(context, output, code);
            super::OrthancPluginErrorCode_OrthancPluginErrorCode_NotAcceptable
        }
        StatusCode::NOT_IMPLEMENTED => {
            send_http_status_code(context, output, code);
            super::OrthancPluginErrorCode_OrthancPluginErrorCode_NotImplemented
        }
        StatusCode::BAD_REQUEST => {
            send_http_status_code(context, output, code);
            super::OrthancPluginErrorCode_OrthancPluginErrorCode_BadRequest
        }
        // note: OrthancPluginErrorCode_Timeout is *not* used for codes 408 nor 504
        _ => {
            send_http_status_code(context, output, code);
            super::OrthancPluginErrorCode_OrthancPluginErrorCode_Success
        }
    }
}

/// Respond to an HTTP request without a body.
///
/// Note: this function handles the "must use" logic required by Orthanc. See
/// https://orthanc.uclouvain.be/sdk/group__REST.html#ga61be84f0a8886c6c350b20055f97ddc5
fn respond_no_body(
    context: *mut super::OrthancPluginContext,
    output: *mut super::OrthancPluginRestOutput,
    code: StatusCode,
) -> super::OrthancPluginErrorCode {
    match code {
        StatusCode::OK => {
            answer_buffer(
                context,
                output,
                Vec::new(),
                CString::from_str("text/plain").unwrap(),
            );
            super::OrthancPluginErrorCode_OrthancPluginErrorCode_Success
        }
        StatusCode::MOVED_PERMANENTLY => {
            // TODO must use ::OrthancPluginRedirect()
            super::OrthancPluginErrorCode_OrthancPluginErrorCode_NotImplemented
        }
        StatusCode::UNAUTHORIZED => {
            // TODO must use ::OrthancPluginSendUnauthorized()
            super::OrthancPluginErrorCode_OrthancPluginErrorCode_NotImplemented
        }
        StatusCode::METHOD_NOT_ALLOWED => {
            // TODO must use ::OrthancPluginSendMethodNotAllowed()
            super::OrthancPluginErrorCode_OrthancPluginErrorCode_NotImplemented
        }
        StatusCode::NOT_ACCEPTABLE => {
            super::OrthancPluginErrorCode_OrthancPluginErrorCode_NotAcceptable
        }
        StatusCode::NOT_IMPLEMENTED => {
            super::OrthancPluginErrorCode_OrthancPluginErrorCode_NotImplemented
        }
        StatusCode::BAD_REQUEST => super::OrthancPluginErrorCode_OrthancPluginErrorCode_BadRequest,
        // note: OrthancPluginErrorCode_Timeout is *not* used for codes 408 nor 504
        _ => {
            send_http_status_code(context, output, code);
            super::OrthancPluginErrorCode_OrthancPluginErrorCode_Success
        }
    }
}

/// Answer to a REST request. Translated from `OrthancPluginAnswerBuffer`.
fn answer_buffer(
    context: *mut super::OrthancPluginContext,
    output: *mut super::OrthancPluginRestOutput,
    body: Vec<u8>,
    mime_type: CString,
) {
    let params = super::_OrthancPluginAnswerBuffer {
        output,
        answer: body.as_ptr() as *const _,
        answerSize: body.len() as u32,
        mimeType: mime_type.as_ptr(),
    };
    invoke_service(
        context,
        Box::new(params),
        super::_OrthancPluginService__OrthancPluginService_AnswerBuffer,
    )
}

/// Send an HTTP status, with a custom body.
///
/// Translated from `OrthancPluginSendHttpStatus`.
fn send_http_status<S: serde::Serialize>(
    context: *mut super::OrthancPluginContext,
    output: *mut super::OrthancPluginRestOutput,
    code: StatusCode,
    body: Vec<u8>,
) {
    let params = super::_OrthancPluginSendHttpStatus {
        output,
        status: code.as_u16(),
        body: body.as_ptr() as *const _,
        bodySize: body.len() as u32,
    };
    invoke_service(
        context,
        Box::new(params),
        super::_OrthancPluginService__OrthancPluginService_SendHttpStatus,
    )
}

/// Send an HTTP status.
///
/// Translated from `OrthancPluginSendHttpStatusCode`.
fn send_http_status_code(
    context: *mut super::OrthancPluginContext,
    output: *mut super::OrthancPluginRestOutput,
    code: StatusCode,
) {
    let params = super::_OrthancPluginSendHttpStatusCode {
        output,
        status: code.as_u16(),
    };
    invoke_service(
        context,
        Box::new(params),
        super::_OrthancPluginService__OrthancPluginService_SendHttpStatusCode,
    )
}

fn invoke_service<T>(
    context: *mut super::OrthancPluginContext,
    params: Box<T>,
    service: super::_OrthancPluginService,
) {
    let params: *const std::ffi::c_void = Box::into_raw(params) as *mut std::ffi::c_void;
    unsafe {
        let invoker = (*context).InvokeService;
        invoker.unwrap()(context, service, params);
    }
}
