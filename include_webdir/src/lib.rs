//! Like [`include_dir`](https://docs.rs/include_dir), but specialized for
//! serving a directory as a web application.
//!
//! ## Differences from `include_dir`
//!
//! - Only files are included. Directories are excluded.
//! - Paths must be valid UTF-8 without nul characters.
//! - Path lookup is optimized (using [`litemap`]).
//! - File metadata useful for HTTP caching are computed at build time.
//!
//! ## HTTP Caching Headers
//!
//! [`CWebFile`] fields provide values which can be used for HTTP cache-related
//! response headers:
//!
//! - `ETag` is the base32-encoded rapidhash of the file contents.
//! - `Last-Modified` is read from the filesystem metadata.
//! - `Cache-Control: immutable` can be set if the path starts with `assets/`.
//!
//! ### Immutable Assets
//!
//! The assumption that files under `assets/` should be immutable comes from
//! the default settings of [Vite](https://vite.dev/). Most modern web apps use
//! Vite as a bundler. Its default settings implement the "cache-busting"
//! pattern, meaning that static resources will be written to a subpath of
//! `assets/` and a hash will be appended to file names before the file
//! extension. This pattern makes it possible to cache entire static web
//! applications on the client web browser.

use std::ffi::CStr;

pub use include_webdir_macros::include_cwebdir;
pub use litemap;

/// File data with HTTP cache-related metadata.
#[derive(Copy, Clone)]
pub struct CWebFile<'a> {
    /// MIME type essence
    pub mime: &'a CStr,
    /// ETag value
    pub etag: &'a CStr,
    /// Last-Modified date
    pub last_modified: &'a CStr,
    /// Whether this file is immutable
    pub immutable: bool,
    /// File content
    pub body: &'a [u8],
}

/// Map of paths to files.
pub type CWebBundle<'a> = litemap::LiteMap<&'a str, CWebFile<'a>, &'a [(&'a str, CWebFile<'a>)]>;
