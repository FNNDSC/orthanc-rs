//! Near-direct translations (i.e. "low-level") of Orthanc C SDK functions.
//!
//! The Rust functions here are nearly equivalent to the C code of `OrthancCPlugin.h`,
//! with minor differences:
//!
//! - Functions such as [register_on_change] and [register_rest] will panic if
//!   `context->InvokeService` produces an unsuccessful error code.
//! - Functions such as [answer_buffer] and [send_method_not_allowed] will
//!   return the error code produced by `context->InvokeService` (instead of
//!   having a `void` signature like `OrthancCPlugin.h`).

use std::ffi::{CStr, CString};

use crate::bindings;

/// Translation of the C code which appears as the last line of most functions in `OrthancCPlugin.h`,
/// e.g. <https://orthanc.uclouvain.be/sdk/OrthancCPlugin_8h_source.html#l03056>
///
/// ```c
/// context->InvokeService(context, service, &params);
/// ```
#[inline(always)]
pub(crate) fn invoke_service<T>(
    context: *mut bindings::OrthancPluginContext,
    service: bindings::_OrthancPluginService,
    params: T,
) -> bindings::OrthancPluginErrorCode {
    let boxed = Box::new(params);
    let params: *const std::ffi::c_void = Box::into_raw(boxed) as *mut std::ffi::c_void;
    unsafe {
        let invoker = (*context).InvokeService;
        invoker.unwrap()(context, service, params)
    }
}

/// Calls [invoke_service], panics if unsuccessful.
#[inline(always)]
pub(crate) fn must_invoke_service<T>(
    context: *mut bindings::OrthancPluginContext,
    service: bindings::_OrthancPluginService,
    params: T,
    caller: &'static str,
) {
    let code = invoke_service(context, service, params);
    if code != bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_Success {
        panic!(
            "Unsuccessful call to context->InvokeService in {}::sdk::{caller} (code {code})",
            env!("CARGO_PKG_NAME")
        )
    }
}

/// Translation of [OrthancPluginFreeMemoryBuffer](https://orthanc.uclouvain.be/sdk/OrthancCPlugin_8h_source.html#l02241)
#[inline(always)]
pub(crate) unsafe fn free_memory_buffer(
    context: *mut bindings::OrthancPluginContext,
    buffer: *mut bindings::OrthancPluginMemoryBuffer,
) {
    unsafe { (*context).Free.unwrap()((*buffer).data) }
}

/// Create a raw pointer to a memory buffer.
#[inline(always)]
pub(crate) fn create_empty_buffer() -> *mut bindings::OrthancPluginMemoryBuffer {
    let data = std::ptr::null_mut();
    let buf = bindings::OrthancPluginMemoryBuffer { data, size: 0 };
    let boxed = Box::new(buf);
    Box::into_raw(boxed)
}

/// Translation of [OrthancPluginFreeString](https://orthanc.uclouvain.be/sdk/OrthancCPlugin_8h_source.html#l02079).
#[inline(always)]
pub(crate) unsafe fn free_string(
    context: *mut bindings::OrthancPluginContext,
    buffer: std::mem::MaybeUninit<*mut std::ffi::c_char>,
) {
    unsafe { (*context).Free.unwrap()(buffer.assume_init() as *mut std::ffi::c_void) }
}

/// Register a callback function that is called whenever a change happens to some DICOM resource.
///
/// Translated from [`OrthancPluginRegisterOnChangeCallback`](https://orthanc.uclouvain.be/sdk/OrthancCPlugin_8h_source.html#l03597).
pub fn register_on_change(
    context: *mut bindings::OrthancPluginContext,
    callback: bindings::OrthancPluginOnChangeCallback,
) {
    let params = bindings::_OrthancPluginOnChangeCallback { callback };
    must_invoke_service(
        context,
        bindings::_OrthancPluginService__OrthancPluginService_RegisterOnChangeCallback,
        params,
        "register_on_change",
    )
}

/// Register a REST callback.
///
/// Translated from [OrthancPluginRegisterRestCallback](https://orthanc.uclouvain.be/sdk/OrthancCPlugin_8h_source.html#l02341)
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
        "register_rest",
    )
}

/// Register a REST callback, without locking.
///
/// Translated from [OrthancPluginRegisterRestCallbackNoLock](https://orthanc.uclouvain.be/sdk/OrthancCPlugin_8h_source.html#l02381).
pub fn register_rest_no_lock(
    context: *mut bindings::OrthancPluginContext,
    path_regex: &std::ffi::CStr,
    callback: bindings::OrthancPluginRestCallback,
) {
    let params = bindings::_OrthancPluginRestCallback {
        pathRegularExpression: path_regex.as_ptr(),
        callback,
    };
    must_invoke_service(
        context,
        bindings::_OrthancPluginService__OrthancPluginService_RegisterRestCallbackNoLock,
        params,
        "register_rest_no_lock",
    )
}

/// Answer to a REST request by signaling that the queried URI does not support this method.
///
/// Translated from [`OrthancPluginSendMethodNotAllowed`](https://orthanc.uclouvain.be/sdk/OrthancCPlugin_8h_source.html#l03094).
#[cfg(feature = "webapp")]
pub(crate) fn send_method_not_allowed(
    context: *mut bindings::OrthancPluginContext,
    output: *mut bindings::OrthancPluginRestOutput,
    allowed_methods: &std::ffi::CStr,
) -> bindings::OrthancPluginErrorCode {
    let params = bindings::_OrthancPluginOutputPlusArgument {
        output,
        argument: allowed_methods.as_ptr(),
    };
    invoke_service(
        context,
        bindings::_OrthancPluginService__OrthancPluginService_SendMethodNotAllowed,
        params,
    )
}

/// Answer to a REST request.
///
/// Translated from [`OrthancPluginAnswerBuffer`](https://orthanc.uclouvain.be/sdk/OrthancCPlugin_8h_source.html#l02451).
pub(crate) fn answer_buffer(
    context: *mut bindings::OrthancPluginContext,
    output: *mut bindings::OrthancPluginRestOutput,
    body: &[u8],
    mime_type: &CStr,
) -> bindings::OrthancPluginErrorCode {
    let params = bindings::_OrthancPluginAnswerBuffer {
        output,
        answer: body.as_ptr() as *const _,
        answerSize: body.len() as u32,
        mimeType: mime_type.as_ptr(),
    };
    invoke_service(
        context,
        bindings::_OrthancPluginService__OrthancPluginService_AnswerBuffer,
        params,
    )
}

/// Sets a HTTP header in the HTTP answer.
///
/// Translated from [`OrthancPluginSetHttpHeader`](https://orthanc.uclouvain.be/sdk/OrthancCPlugin_8h_source.html#l03149).
pub(crate) fn set_http_header(
    context: *mut bindings::OrthancPluginContext,
    output: *mut bindings::OrthancPluginRestOutput,
    key: &CStr,
    value: &CStr,
) -> bindings::OrthancPluginErrorCode {
    let params = bindings::_OrthancPluginSetHttpHeader {
        output,
        key: key.as_ptr(),
        value: value.as_ptr(),
    };
    invoke_service(
        context,
        bindings::_OrthancPluginService__OrthancPluginService_SetHttpHeader,
        params,
    )
}

/// Send an HTTP status, with a custom body.
///
/// Translated from [`OrthancPluginSendHttpStatus`](https://orthanc.uclouvain.be/sdk/OrthancCPlugin_8h_source.html#l04225).
pub(crate) fn send_http_status(
    context: *mut bindings::OrthancPluginContext,
    output: *mut bindings::OrthancPluginRestOutput,
    status: u16,
    body: Vec<u8>,
) -> bindings::OrthancPluginErrorCode {
    let params = bindings::_OrthancPluginSendHttpStatus {
        output,
        status,
        body: body.as_ptr() as *const _,
        bodySize: body.len() as u32,
    };
    invoke_service(
        context,
        bindings::_OrthancPluginService__OrthancPluginService_SendHttpStatus,
        params,
    )
}

/// Send an HTTP status.
///
/// Translated from [`OrthancPluginSendHttpStatusCode`](https://orthanc.uclouvain.be/sdk/OrthancCPlugin_8h_source.html#l03048).
pub(crate) fn send_http_status_code(
    context: *mut bindings::OrthancPluginContext,
    output: *mut bindings::OrthancPluginRestOutput,
    status: u16,
) -> bindings::OrthancPluginErrorCode {
    let params = bindings::_OrthancPluginSendHttpStatusCode { output, status };
    invoke_service(
        context,
        bindings::_OrthancPluginService__OrthancPluginService_SendHttpStatusCode,
        params,
    )
}
