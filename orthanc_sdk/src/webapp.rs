use std::ffi::CStr;

use http::StatusCode;

use crate::{bindings, http::Method, respond_no_body, send_method_not_allowed};

/// Create a REST callback handler which serves a static directory.
pub fn serve_static_files(
    context: *mut bindings::OrthancPluginContext,
    output: *mut bindings::OrthancPluginRestOutput,
    url: *const std::os::raw::c_char,
    request: *const bindings::OrthancPluginHttpRequest,
    dir: &include_dir::Dir,
) -> bindings::OrthancPluginErrorCode {
    match serve_static_files_impl(context, output, url, request, dir) {
        Ok(code) => code,
        Err(code) => code,
    }
}

fn serve_static_files_impl(
    context: *mut bindings::OrthancPluginContext,
    output: *mut bindings::OrthancPluginRestOutput,
    url: *const std::os::raw::c_char,
    request: *const bindings::OrthancPluginHttpRequest,
    dir: &include_dir::Dir,
) -> Result<bindings::OrthancPluginErrorCode, bindings::OrthancPluginErrorCode> {
    if !method_is_get(request) {
        let code = send_method_not_allowed(context, output, c"GET");
        return Ok(code);
    }
    let c_url = unsafe { CStr::from_ptr(url) };
    let r_url = c_url.to_str().or_else(|_| {
        let code = respond_no_body(context, output, StatusCode::NOT_FOUND);
        return Ok(code);
    });
    todo!()
}

fn method_is_get(request: *const bindings::OrthancPluginHttpRequest) -> bool {
    Method::try_from(unsafe { (*request).method })
        .map(|m| matches!(m, Method::Get))
        .unwrap_or(false)
}
