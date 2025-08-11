//! Utilities for packaging a static web app as an Orthanc plugin. See [`serve_static_file`].

use std::borrow::Cow;
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
/// ## Pre-conditions
///
/// The regular expression passed to [`register_rest_no_lock`] _must_ match the
/// relative path as the first capture group. For example, to serve the bundle
/// under a prefix "/my_webapp", use the value `c"/my_webapp/?(.*)`.
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
/// - `request`: The incoming request parameters, as received by the callback passed to [`register_rest_no_lock`].
/// - `bundle`: web bundle to serve&mdash;can either be the value of [`include_dir!`](include_dir::include_dir)
///   or [`prepare_bundle`].
///
/// [`register_rest_no_lock`]: crate::register_rest_no_lock
/// [`include_dir!`]: include_dir::include_dir
pub fn serve_static_file(
    context: *mut bindings::OrthancPluginContext,
    output: *mut bindings::OrthancPluginRestOutput,
    request: *const bindings::OrthancPluginHttpRequest,
    bundle: &impl WebBundle,
) -> bindings::OrthancPluginErrorCode {
    serve_static_file_impl(context, output, request, bundle).into_code()
}

fn serve_static_file_impl(
    context: *mut bindings::OrthancPluginContext,
    output: *mut bindings::OrthancPluginRestOutput,
    request: *const bindings::OrthancPluginHttpRequest,
    bundle: &impl WebBundle,
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

    fn is_immutable(&self) -> bool {
        false
    }

    fn last_modified(&self) -> Option<&CStr> {
        None
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
    immutable: bool,
    date: Cow<'a, CStr>,
}

impl<'a> PreparedFile<'a> {
    fn new(file: &'a include_dir::File<'a>, immutable: bool, date: Cow<'a, CStr>) -> Self {
        let guess = mime_guess::from_path(file.path()).first_or_octet_stream();
        Self {
            body: file.contents(),
            mime: CString::new(guess.essence_str()).unwrap(),
            etag: etag_of(file),
            immutable,
            date,
        }
    }
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

    fn is_immutable(&self) -> bool {
        self.immutable
    }

    fn last_modified(&self) -> Option<&CStr> {
        Some(&self.date)
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
/// const LAST_MODIFIED: &std::ffi::CStr = c"Tue, 22 Feb 2022 20:20:20 GMT";
/// static PREPARED_BUNDLE: RwLock<Option<PreparedBundle<'static>>> = RwLock::new(None);
///
/// #[unsafe(no_mangle)]
/// pub extern "C" fn OrthancPluginInitialize(
///     context: *mut bindings::OrthancPluginContext,
/// ) -> bindings::OrthancPluginErrorCode {
///     let mut prepared_bundle = PREPARED_BUNDLE.try_write().unwrap();
///     *prepared_bundle = Some(orthanc_sdk::webapp::prepare_bundle(&DIST, |_| false, |_| LAST_MODIFIED));
///     bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_Success
/// }
///
/// #[unsafe(no_mangle)]
/// pub extern "C" fn OrthancPluginFinalize() {
///     let mut prepared_bundle = PREPARED_BUNDLE.try_write().unwrap();
///     *prepared_bundle = None;
/// }
/// ```
///
/// ## Immutable Assets
///
/// The `Cache-Control: immutable` header _should_ be set on files which we
/// know will never change. For example, if you have bundled jquery, you might
/// set `is_immutable` to `|p| p == "jquery-3.7.1.min.js"`.
///
/// When using [Vite](https://vite.dev/), consider using
/// `|p| p.starts_with("assets/")`. Vite puts immutable assets in an `assets`
/// directory and appends hashes to file names.
///
/// ## Parameters
///
/// - `dir`: return value of [`include_dir!`][include_dir::include_dir]
/// - `is_immutable`: a function which determines whether to use the response
///                   header `Cache-Control: immutable`.
/// - `date_of`: a function which sets the `Last-Modified` response header.
pub fn prepare_bundle<'a, D: Into<Cow<'a, CStr>>>(
    dir: &'a include_dir::Dir,
    is_immutable: impl Fn(&'a str) -> bool,
    date_of: impl Fn(&'a str) -> D,
) -> PreparedBundle<'a> {
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
    HashMap::from_iter(to_webfile_entries(dir, &is_immutable, &date_of))
}

fn to_webfile_entries<'a, D: Into<Cow<'a, CStr>>>(
    dir: &'a include_dir::Dir,
    is_immutable: &impl Fn(&'a str) -> bool,
    date_of: &impl Fn(&'a str) -> D,
) -> Vec<(&'a str, PreparedFile<'a>)> {
    dir.entries()
        .iter()
        .flat_map(|entry| match entry {
            include_dir::DirEntry::Dir(dir) => to_webfile_entries(dir, is_immutable, date_of),
            include_dir::DirEntry::File(file) => {
                let path = file
                    .path()
                    .to_str()
                    .expect("Web bundle contains a non-unicode path");
                let mutable = is_immutable(path);
                let modified_date = date_of(path).into();
                debug_assert!(
                    modified_date.to_str().unwrap().ends_with("GMT"),
                    "HTTP dates must be expressed in GMT."
                );
                vec![(path, PreparedFile::new(file, mutable, modified_date))]
            }
        })
        .collect()
}

fn etag_of(file: &include_dir::File) -> CString {
    let hash = rapidhash::v3::rapidhash_v3(file.contents());
    let encoded = base32::encode(base32::Alphabet::Crockford, &hash.to_le_bytes());
    CString::new(format!("\"{encoded}\"")).unwrap()
}
