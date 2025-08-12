//! Utilities for packaging a static web app as an Orthanc plugin. See [`serve_static_file`].

use std::ffi::{CStr, CString};

use http::StatusCode;

use crate::error_code::*;
use crate::sdk::{answer_buffer, set_http_header};
use crate::send_http_status_code;
use crate::{bindings, http::Method, send_method_not_allowed};

/// Create a REST callback handler which serves a static directory.
///
/// Typically, this should be called by the function which was passed to [`register_rest_no_lock`].
///
/// ## Pre-conditions
///
/// The regular expression passed to [`register_rest_no_lock`] _must_ match the
/// relative path as the first capture group. For example, to serve the bundle
/// under a prefix "/my_webapp", use the value `c"/my_webapp/?(.*)"`.
///
/// ## Behavior
///
/// - The paths "" and "/" are mapped to `index.html`.
/// - Client-side routing is not supported (but should be easy to implement,
///   please open a feature or pull request on [GitHub](https://github.com/FNNDSC/orthanc-rs/).)
///
/// ## Example
///
/// _Complete_ Orthanc plugin example which serves a directory called
/// `example_directory` under the path `/my_webapp`:
///
/// ```
/// use std::sync::RwLock;
/// use orthanc_sdk::bindings;
/// use include_webdir::{include_cwebdir, CWebBundle};
///
/// struct OrthancContext(*mut bindings::OrthancPluginContext);
/// unsafe impl Send for OrthancContext {}
/// unsafe impl Sync for OrthancContext {}
///
/// /// Global variable where OrthancPluginContext object will be stored.
/// static CONTEXT: RwLock<Option<OrthancContext>> = RwLock::new(None);
///
/// /// Directory containing static web application bundle (HTML and other files).
/// const DIST: CWebBundle = include_cwebdir!("$CARGO_MANIFEST_DIR/example_directory");
///
/// #[unsafe(no_mangle)]
/// pub extern "C" fn OrthancPluginInitialize(
///     context: *mut bindings::OrthancPluginContext,
/// ) -> bindings::OrthancPluginErrorCode {
///     let mut global_context = CONTEXT.try_write().unwrap();
///     *global_context = Some(OrthancContext(context));
///     orthanc_sdk::register_rest_no_lock(context, c"/my_webapp/?(.*)", Some(rest_callback));
///     bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_Success
/// }
///
/// #[unsafe(no_mangle)]
/// extern "C" fn rest_callback(
///     output: *mut bindings::OrthancPluginRestOutput,
///     url: *const std::ffi::c_char,
///     request: *const bindings::OrthancPluginHttpRequest,
/// ) -> bindings::OrthancPluginErrorCode {
///     if let Ok(global_context) = CONTEXT.try_read().as_ref()
///         && let Some(context) = global_context.as_ref()
///     {
///         orthanc_sdk::serve_static_file(context.0, output, request, &DIST)
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
/// A real-life example can be found here:
/// <https://github.com/FNNDSC/orthanc-patient-list/blob/release/20250812/plugin/src/plugin.rs>
///
/// ## Parameters
///
/// The `bundle` parameter may either be the value returned by [`include_dir!`]
/// or [`include_cwebdir!`].
///
/// - [`include_cwebdir!`] is better for performance: it enables the usage of
///   HTTP cache-related response headers such as `Cache-Control`, `ETag`, and
///   `Last-Modified`.
/// - [`include_dir!`] does not provide the necessary information to set HTTP
///   cache-related response headers.
///
/// [`register_rest_no_lock`]: crate::register_rest_no_lock
/// [`include_dir!`]: include_dir::include_dir
/// [`include_cwebdir!`]: include_webdir::include_cwebdir
pub fn serve_static_file(
    context: *mut bindings::OrthancPluginContext,
    output: *mut bindings::OrthancPluginRestOutput,
    request: *const bindings::OrthancPluginHttpRequest,
    bundle: &impl OrthancServableBundle,
) -> bindings::OrthancPluginErrorCode {
    serve_static_file_impl(context, output, request, bundle).into_code()
}

fn serve_static_file_impl(
    context: *mut bindings::OrthancPluginContext,
    output: *mut bindings::OrthancPluginRestOutput,
    request: *const bindings::OrthancPluginHttpRequest,
    bundle: &impl OrthancServableBundle,
) -> ErrorCodeResult {
    if !method_is_get(request) {
        return send_method_not_allowed(context, output, c"GET").into_result();
    }
    let path = unsafe { first_group_of(request) }.ok_or_else(|| send_not_found(context, output))?;
    // NOTE: Orthanc strips hash `#` and query `?` components from the URI for us
    let resolved_path = if path.is_empty() { "index.html" } else { path };
    let code = if let Some(file) = bundle.get_file(resolved_path) {
        if file.is_immutable() {
            set_http_header(
                context,
                output,
                c"Cache-Control",
                c"public, max-age=31536000, immutable",
            )
            .into_result()?;
        }
        if let Some(date) = file.last_modified() {
            set_http_header(context, output, c"Last-Modified", date).into_result()?;
        }
        if let Some(etag) = file.etag() {
            if let Some(value) = unsafe { get_header(request, c"if-none-match") }
                && value == etag
            {
                send_not_modified(context, output).into_result()
            } else {
                set_http_header(context, output, c"ETag", etag)
                    .into_result()
                    .and_then(|_| {
                        answer_buffer(context, output, file.body(), file.mime()).into_result()
                    })
            }
        } else {
            answer_buffer(context, output, file.body(), file.mime()).into_result()
        }
        .into_code()
    } else {
        send_not_found(context, output)
    };
    Ok(code)
}

fn send_not_found(
    context: *mut bindings::OrthancPluginContext,
    output: *mut bindings::OrthancPluginRestOutput,
) -> bindings::OrthancPluginErrorCode {
    send_http_status_code(context, output, StatusCode::NOT_FOUND.as_u16())
}

fn send_not_modified(
    context: *mut bindings::OrthancPluginContext,
    output: *mut bindings::OrthancPluginRestOutput,
) -> bindings::OrthancPluginErrorCode {
    send_http_status_code(context, output, StatusCode::NOT_MODIFIED.as_u16())
}

unsafe fn first_group_of<'a>(
    request: *const bindings::OrthancPluginHttpRequest,
) -> Option<&'a str> {
    let count = (unsafe { *request }).groupsCount;
    if count < 1 {
        return None;
    }
    let c_str = unsafe { CStr::from_ptr(*(*request).groups) };
    c_str.to_str().ok()
}

/// Get header value from request to Orthanc.
///
/// NOTE: Orthanc converts all header keys to lowercase.
unsafe fn get_header<'a>(
    request: *const bindings::OrthancPluginHttpRequest,
    key: &CStr,
) -> Option<&'a CStr> {
    let len = (unsafe { *request }).headersCount as usize;
    let keys = unsafe { std::slice::from_raw_parts((*request).headersKeys, len) };
    let i = keys.iter().enumerate().find_map(|(i, ptr)| {
        if unsafe { CStr::from_ptr(*ptr) } == key {
            Some(i)
        } else {
            None
        }
    });
    if let Some(i) = i {
        let value = unsafe {
            let values = std::slice::from_raw_parts((*request).headersValues, len);
            CStr::from_ptr(values[i])
        };
        Some(value)
    } else {
        None
    }
}

fn method_is_get(request: *const bindings::OrthancPluginHttpRequest) -> bool {
    Method::try_from(unsafe { (*request).method })
        .map(|m| matches!(m, Method::Get))
        .unwrap_or(false)
}

/// Bundle of files to be served by Orthanc's web server.
pub trait OrthancServableBundle {
    /// Associated type representing file obtained from this bundle.
    type File<'b>: OrthancServableFile
    where
        // https://github.com/rust-lang/rust/issues/87479
        Self: 'b;

    /// Get a file from this bundle.
    fn get_file<'a>(&'a self, path: &'a str) -> Option<Self::File<'a>>;
}

impl OrthancServableBundle for include_dir::Dir<'_> {
    type File<'b>
        = IncludedFile<'b>
    where
        Self: 'b;
    fn get_file<'a>(&'a self, path: &'a str) -> Option<Self::File<'a>> {
        self.get_file(path).map(IncludedFile::from)
    }
}

/// A file which can be served by Orthanc's web server.
pub trait OrthancServableFile {
    /// File contents as bytes.
    fn body(&self) -> &[u8];

    /// File MIME type essence.
    fn mime(&self) -> &CStr;

    /// Value for HTTP `ETag` response header.
    fn etag(&self) -> Option<&CStr>;

    /// Returns `true` if the header `Cache-Control: immutable` should be used.
    fn is_immutable(&self) -> bool;

    /// Returns the value for the `Last-Modified` response header.
    fn last_modified(&self) -> Option<&CStr>;
}

/// A wrapper for [`include_dir::File`] with a known MIME type.
pub struct IncludedFile<'a> {
    file: &'a include_dir::File<'a>,
    mime: CString,
}

impl<'a> From<&'a include_dir::File<'a>> for IncludedFile<'a> {
    fn from(file: &'a include_dir::File) -> Self {
        let guess = mime_guess::from_path(file.path()).first_or_octet_stream();
        IncludedFile {
            file,
            mime: CString::new(guess.essence_str()).unwrap(),
        }
    }
}

impl OrthancServableFile for IncludedFile<'_> {
    fn body(&self) -> &[u8] {
        self.file.contents()
    }

    fn mime(&self) -> &CStr {
        &self.mime
    }

    fn etag(&self) -> Option<&CStr> {
        None
    }

    fn is_immutable(&self) -> bool {
        false
    }

    fn last_modified(&self) -> Option<&CStr> {
        None
    }
}

impl OrthancServableBundle for include_webdir::CWebBundle<'_> {
    type File<'b>
        = &'b include_webdir::CWebFile<'b>
    where
        Self: 'b;

    fn get_file<'a>(&'a self, path: &'a str) -> Option<Self::File<'a>> {
        self.get(path)
    }
}

impl OrthancServableFile for &include_webdir::CWebFile<'_> {
    fn body(&self) -> &[u8] {
        self.body
    }

    fn mime(&self) -> &CStr {
        self.mime
    }

    fn etag(&self) -> Option<&CStr> {
        Some(self.etag)
    }

    fn is_immutable(&self) -> bool {
        self.immutable
    }

    fn last_modified(&self) -> Option<&CStr> {
        Some(self.last_modified)
    }
}
