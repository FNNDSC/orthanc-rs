//! Utilities for packaging a static web app as an Orthanc plugin. See [`serve_static_file`].

use std::collections::HashMap;
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
/// ## Behavior
///
/// - The paths "" and "/" are mapped to `index.html`.
/// - URI hash and query components (i.e. everything after and including `#`
///   and `?`) are stripped.
/// - "Client-side routing" not (yet) supported.
///
/// ## Example
///
/// _Complete_ Orthanc plugin example which serves a directory called
/// `example_directory` under the path `/my_webapp`:
///
/// ```
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
/// #[unsafe(no_mangle)]
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
/// A real-life example can be found here:
/// <https://github.com/FNNDSC/orthanc-patient-list/blob/release/20250810/plugin/src/plugin.rs>
///
/// ## Performance
///
/// Call [`prepare_bundle`] during `OrthancPluginInitialize()` to pre-compute a
/// value for the `ETag` response header for each file and optimize the data
/// structure representing the bundle.
///
/// ## Parameters
///
/// - `context`: The Orthanc plugin context, as received by `OrthancPluginInitialize()`.`
/// - `output`: The HTTP connection to the client application.
/// - `url`: The URL, as received by the callback passed to [`register_rest_no_lock`].
/// - `request`: The incoming request parameters, as received by the callback passed to [`register_rest_no_lock`].
/// - `bundle`: web bundle to serve&mdash;can either be the value of [`include_dir!`](include_dir::include_dir)
///   or [`prepare_bundle`].
/// - `base`: Base path the web app is being served from.
///
/// [`register_rest_no_lock`]: crate::register_rest_no_lock
/// [`include_dir!`]: include_dir::include_dir
pub fn serve_static_file(
    context: *mut bindings::OrthancPluginContext,
    output: *mut bindings::OrthancPluginRestOutput,
    url: *const std::os::raw::c_char,
    request: *const bindings::OrthancPluginHttpRequest,
    bundle: &impl WebBundle,
    base: &str,
) -> bindings::OrthancPluginErrorCode {
    serve_static_file_impl(context, output, url, request, bundle, base).into_code()
}

fn serve_static_file_impl(
    context: *mut bindings::OrthancPluginContext,
    output: *mut bindings::OrthancPluginRestOutput,
    url: *const std::os::raw::c_char,
    request: *const bindings::OrthancPluginHttpRequest,
    bundle: &impl WebBundle,
    base: &str,
) -> ErrorCodeResult {
    if !method_is_get(request) {
        return send_method_not_allowed(context, output, c"GET").into_result();
    }
    let c_url = unsafe { CStr::from_ptr(url) };
    let r_url = c_url
        .to_str()
        .map_err(|_| send_not_found(context, output))?;
    let path = relative_path_of(base, r_url);
    let code = if let Some(file) = bundle.get_file(path) {
        if let Some(etag) = file.etag() {
            set_http_header(context, output, c"ETag", etag).into_result()
        } else {
            Ok(bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_Success)
        }
        .and_then(|_| answer_buffer(context, output, file.body(), file.mime()).into_result())
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

/// Bundle of files to be served by Orthanc's web server.
pub trait WebBundle {
    /// Associated type representing file obtained from this bundle.
    type File<'b>: WebFile
    where
        // https://github.com/rust-lang/rust/issues/87479
        Self: 'b;

    /// Get a file from this bundle.
    fn get_file<'a>(&'a self, path: &'a str) -> Option<Self::File<'a>>;
}

impl WebBundle for include_dir::Dir<'_> {
    type File<'b>
        = IncludedFile<'b>
    where
        Self: 'b;
    fn get_file<'a>(&'a self, path: &'a str) -> Option<Self::File<'a>> {
        self.get_file(path).map(IncludedFile::from)
    }
}

/// A file which can be served by Orthanc's web server.
pub trait WebFile {
    /// File contents as bytes.
    fn body(&self) -> &[u8];

    /// File MIME type essence.
    fn mime(&self) -> &CStr;

    /// Value for HTTP `ETag` response header.
    fn etag(&self) -> Option<&CStr>;
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

impl WebFile for IncludedFile<'_> {
    fn body(&self) -> &[u8] {
        self.file.contents()
    }

    fn mime(&self) -> &CStr {
        &self.mime
    }

    fn etag(&self) -> Option<&CStr> {
        None
    }
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

/// Map of _"prepared"_ web files to be served by Orthanc's web server.
pub type PreparedBundle<'a> = HashMap<&'a str, PreparedFile<'a>>;

impl WebBundle for PreparedBundle<'_> {
    type File<'b>
        = &'b PreparedFile<'b>
    where
        Self: 'b;

    fn get_file<'a>(&'a self, path: &'a str) -> Option<Self::File<'a>> {
        self.get(path)
    }
}

/// File data with pre-computed MIME type and ETag value.
pub struct PreparedFile<'a> {
    body: &'a [u8],
    mime: CString,
    etag: CString,
}

impl WebFile for &PreparedFile<'_> {
    fn body(&self) -> &[u8] {
        self.body
    }

    fn mime(&self) -> &CStr {
        &self.mime
    }

    fn etag(&self) -> Option<&CStr> {
        Some(&self.etag)
    }
}

impl<'a> From<&'a include_dir::File<'a>> for PreparedFile<'a> {
    fn from(value: &'a include_dir::File<'a>) -> Self {
        let guess = mime_guess::from_path(value.path()).first_or_octet_stream();
        PreparedFile {
            body: value.contents(),
            mime: CString::new(guess.essence_str()).unwrap(),
            etag: etag_of(value),
        }
    }
}

/// Prepare static files for web serving (guess MIME type and generate `ETag` header value).
///
/// ## Example
///
/// ```
/// use std::sync::RwLock;
/// use include_dir::include_dir;
/// use orthanc_sdk::{bindings, webapp::PreparedBundle};
///
/// const DIST: include_dir::Dir = include_dir!("$CARGO_MANIFEST_DIR/example_directory");
/// static PREPARED_BUNDLE: RwLock<Option<PreparedBundle<'static>>> = RwLock::new(None);
///
/// #[unsafe(no_mangle)]
/// pub extern "C" fn OrthancPluginInitialize(
///     context: *mut bindings::OrthancPluginContext,
/// ) -> bindings::OrthancPluginErrorCode {
///     let mut prepared_bundle = PREPARED_BUNDLE.try_write().unwrap();
///     *prepared_bundle = Some(orthanc_sdk::webapp::prepare_bundle(&DIST));
///     bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_Success
/// }
///
/// #[unsafe(no_mangle)]
/// pub extern "C" fn OrthancPluginFinalize() {
///     let mut prepared_bundle = PREPARED_BUNDLE.try_write().unwrap();
///     *prepared_bundle = None;
/// }
/// ```
pub fn prepare_bundle<'a>(dir: &'a include_dir::Dir) -> PreparedBundle<'a> {
    /*
     * It would be cool if ETag header could be assigned to each file at compile time,
     * but this would be a huge amount of work to implement because:
     *
     * - Many std functions are not const, for example iteration is not const. A
     *   third-party crate must be used to achieve const iteration. `konst` implements
     *   a macro which interprets a DSL to achieve const iteration, but it has
     *   limitations ("can't capture dynamic environment"). https://crates.io/crates/konst
     * - Could use `build.rs` e.g. https://crates.io/crates/precomputed-map
     *   but this has a big burden on the library user.
     * - Could use `proc_macro`. I would want to call `include_dir_macros::include_dir`
     *   and modify its output, but this is not possible.
     *   https://users.rust-lang.org/t/call-proc-macro-and-modify-its-output/96486
     */
    HashMap::from_iter(to_webfile_entries(dir))
}

fn to_webfile_entries<'a>(dir: &'a include_dir::Dir) -> Vec<(&'a str, PreparedFile<'a>)> {
    dir.entries()
        .into_iter()
        .flat_map(|entry| match entry {
            include_dir::DirEntry::Dir(dir) => to_webfile_entries(dir),
            include_dir::DirEntry::File(file) => {
                let path = file
                    .path()
                    .to_str()
                    .expect("Web bundle contains a non-unicode path");
                vec![(path, PreparedFile::from(file))]
            }
        })
        .collect()
}

fn etag_of(file: &include_dir::File) -> CString {
    let hash = rapidhash::v3::rapidhash_v3(file.contents());
    let encoded = base32::encode(base32::Alphabet::Crockford, &hash.to_le_bytes());
    CString::new(encoded).unwrap()
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
