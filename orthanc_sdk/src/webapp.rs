use std::ffi::{CStr, CString};

use http::StatusCode;

use crate::callback::{answer_buffer, respond_no_body};
use crate::{bindings, http::Method, send_method_not_allowed};

/// Create a REST callback handler which serves a static directory.
///
/// Typically, this should be called by the function which was passed to [`register_rest_no_lock`].
///
/// ## Behavior
///
/// - The paths "" and "/" are mapped to `index.html`.
/// - URI hash and query components (i.e. everything after and including `#` ori
///   `?`) are stripped.
/// - "Client-side routing" not (yet) supported.
///
/// ## Example
///
/// _Complete_ Orthanc plugin example which serves a directory called
/// `example_directory` under the path `/my_webapp`:
///
/// ```
/// # #![allow(non_snake_case)]
/// use std::sync::RwLock;
/// use orthanc_sdk::bindings;
/// use include_dir::include_dir;
///
/// struct OrthancContext(*mut bindings::OrthancPluginContext);
/// unsafe impl Send for OrthancContext {}
/// unsafe impl Sync for OrthancContext {}
///
/// /// Global variable where OrthancPluginContext object will be stored.
/// static CONTEXT: RwLock<Option<OrthancContext>> = RwLock::new(None);
///
/// /// Directory containing static web application bundle (HTML and other files).
/// const DIST: include_dir::Dir = include_dir!("$CARGO_MANIFEST_DIR/example_directory");
///
/// #[allow(clippy::not_unsafe_ptr_arg_deref)]
/// #[unsafe(no_mangle)]
/// pub extern "C" fn OrthancPluginInitialize(
///     context: *mut bindings::OrthancPluginContext,
/// ) -> bindings::OrthancPluginErrorCode {
///     let mut global_context = CONTEXT.try_write().unwrap();
///     *global_context = Some(OrthancContext(context));
///     orthanc_sdk::register_rest_no_lock(context, c"/my_webapp(/.*)?", Some(rest_callback));
///     bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_Success
/// }
///
/// extern "C" fn rest_callback(
///     output: *mut bindings::OrthancPluginRestOutput,
///     url: *const std::ffi::c_char,
///     request: *const bindings::OrthancPluginHttpRequest,
/// ) -> bindings::OrthancPluginErrorCode {
///     if let Ok(global_context) = CONTEXT.try_read().as_ref()
///         && let Some(context) = global_context.as_ref()
///     {
///         orthanc_sdk::serve_static_file(context.0, output, url, request, &DIST, "/my_webapp")
///     } else {
///         bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_InternalError
///     }
/// }
///
/// #[unsafe(no_mangle)]
/// pub extern "C" fn OrthancPluginGetName() -> *const u8 {
///     "example_webapp\0".as_ptr()
/// }
///
/// #[unsafe(no_mangle)]
/// pub extern "C" fn OrthancPluginGetVersion() -> *const u8 {
///     concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr()
/// }
///
/// #[unsafe(no_mangle)]
/// pub extern "C" fn OrthancPluginFinalize() {
///     let mut global_context = CONTEXT.try_write().unwrap();
///     *global_context = None;
/// }
/// ```
///
/// ## Parameters
///
/// - `context`: The Orthanc plugin context, as received by `OrthancPluginInitialize()`.`
/// - `output`: The HTTP connection to the client application.
/// - `url`: The URL, as received by the callback passed to [`register_rest_no_lock`].
/// - `request`: The incoming request parameters, as received by the callback passed to [`register_rest_no_lock`].
/// - `dir`: Static files directory imported by [`include_dir!`]
/// - `base`: Base path the web app is being served from.
///
/// [`register_rest_no_lock`]: crate::register_rest_no_lock
/// [`include_dir!`]: include_dir::include_dir
pub fn serve_static_file(
    context: *mut bindings::OrthancPluginContext,
    output: *mut bindings::OrthancPluginRestOutput,
    url: *const std::os::raw::c_char,
    request: *const bindings::OrthancPluginHttpRequest,
    dir: &include_dir::Dir,
    base: &str,
) -> bindings::OrthancPluginErrorCode {
    match serve_static_files_impl(context, output, url, request, dir, base) {
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
    base: &str,
) -> Result<bindings::OrthancPluginErrorCode, bindings::OrthancPluginErrorCode> {
    if !method_is_get(request) {
        let code = send_method_not_allowed(context, output, c"GET");
        return Ok(code);
    }
    let c_url = unsafe { CStr::from_ptr(url) };
    let r_url = c_url
        .to_str()
        .map_err(|_| respond_no_body(context, output, StatusCode::NOT_FOUND))?;
    let path = relative_path_of(base, r_url);
    let code = if let Some(file) = dir.get_file(path) {
        let mime = mime_guess::from_path(path).first_or_octet_stream();
        let c_mime = CString::new(mime.as_ref()).unwrap();
        answer_buffer(context, output, file.contents(), &c_mime)
    } else {
        respond_no_body(context, output, StatusCode::NOT_FOUND)
    };
    Ok(code)
}

/// Strip hash, query, and leading '/' from a URI.
fn relative_path_of<'a, 'b>(base: &'a str, uri: &'b str) -> &'b str {
    let without_hash = uri.split_once('#').map(|(l, _)| l).unwrap_or(uri);
    let rel_path = without_hash
        .split_once('?')
        .map(|(l, _)| l)
        .unwrap_or(without_hash)
        .trim_start_matches(base)
        .trim_start_matches('/');
    if rel_path.is_empty() {
        "index.html"
    } else {
        rel_path
    }
}

fn method_is_get(request: *const bindings::OrthancPluginHttpRequest) -> bool {
    Method::try_from(unsafe { (*request).method })
        .map(|m| matches!(m, Method::Get))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use rstest::*;

    #[rstest]
    #[case("/my", "index.html")]
    #[case("/my/", "index.html")]
    #[case("/my/stuff", "stuff")]
    fn test_relative_path_of(#[case] path: &str, #[case] expected: &str) {
        let actual = relative_path_of("/my", path);
        assert_eq!(actual, expected)
    }
}
