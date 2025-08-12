//! Implementation details of the `include_webapp`.
//!
//! You probably don't want to use this crate directly,

#![cfg_attr(feature = "nightly", feature(track_path, proc_macro_tracked_env))]

use proc_macro::{TokenStream, TokenTree};
use quote::quote;
use std::borrow::Cow;
use std::ffi::CString;
use std::path::{Path, PathBuf};

mod copy_of_include_dir_macros;
use copy_of_include_dir_macros::*;

/// Embed the contents of a directory in your crate and prepare it for HTTP caching.
///
/// ## Pre-Conditions
///
/// All paths must be valid UTF-8 without nul characters.
#[proc_macro]
pub fn include_cwebdir(input: TokenStream) -> TokenStream {
    let tokens: Vec<_> = input.into_iter().collect();

    let path = match tokens.as_slice() {
        [TokenTree::Literal(lit)] => unwrap_string_literal(lit),
        _ => panic!("This macro only accepts a single, non-empty string argument"),
    };

    let path = resolve_path(&path, get_env).unwrap();
    expand_dir(&path).into()
}

fn expand_dir(path: &Path) -> proc_macro2::TokenStream {
    let mut file_paths = find_files(path.into());
    file_paths.sort_unstable();
    let file_infos: Vec<_> = file_paths
        .into_iter()
        .map(|p| FileInfo::new(path, &p))
        .map(CFileInfo::from)
        .map(info_to_quote)
        .collect();
    quote! {
        ::include_webdir::litemap::LiteMap::from_sorted_store_unchecked(&[#(#file_infos),*])
    }
}

fn find_files(path: Cow<Path>) -> Vec<PathBuf> {
    if path.is_file() {
        return vec![path.into_owned()];
    }
    fs_err::read_dir(path)
        .unwrap()
        .map(|result| result.unwrap())
        .flat_map(|entry| {
            let path = entry.path();
            let metadata = entry.metadata().unwrap();
            if metadata.is_dir() {
                find_files(path.into())
            } else if metadata.is_file() {
                vec![path]
            } else {
                panic!("Path is not a dir nor file: {path:?}");
            }
        })
        .collect()
}

struct FileInfo {
    relative_path: String,
    canonical_path: String,
    etag: String,
    last_modified: String,
    mime: mime_guess::Mime,
    immutable: bool,
}

struct CFileInfo {
    relative_path: String,
    canonical_path: String,
    etag: CString,
    last_modified: CString,
    mime: CString,
    immutable: bool,
}

impl From<FileInfo> for CFileInfo {
    fn from(
        FileInfo {
            relative_path,
            canonical_path,
            mime,
            etag,
            last_modified,
            immutable,
        }: FileInfo,
    ) -> Self {
        CFileInfo {
            relative_path,
            canonical_path,
            mime: CString::new(mime.essence_str()).unwrap(),
            etag: CString::new(etag).unwrap(),
            last_modified: CString::new(last_modified).unwrap(),
            immutable,
        }
    }
}

impl FileInfo {
    fn new(root: &Path, path: &Path) -> Self {
        let rel_path = path.strip_prefix(root).unwrap();
        Self {
            relative_path: path_to_normalized_string(rel_path),
            canonical_path: pathbuf_to_string(fs_err::canonicalize(&path).unwrap()),
            mime: mime_guess::from_path(path).first_or_octet_stream(),
            last_modified: httpdate::fmt_http_date(
                fs_err::metadata(&path).unwrap().modified().unwrap(),
            ),
            etag: etag_of(&path),
            immutable: rel_path.starts_with("assets"),
        }
    }
}

fn info_to_quote(
    CFileInfo {
        relative_path,
        canonical_path,
        mime,
        etag,
        last_modified,
        immutable,
    }: CFileInfo,
) -> proc_macro2::TokenStream {
    quote! {
        (
            #relative_path,
            ::include_webdir::CWebFile {
                body: include_bytes!(#canonical_path),
                mime: #mime,
                etag: #etag,
                last_modified: #last_modified,
                immutable: #immutable
            }
        )
    }
}

fn etag_of(path: &Path) -> String {
    let data = fs_err::read(&path).unwrap();
    let hash = rapidhash::v3::rapidhash_v3(&data);
    let encoded = base32::encode(base32::Alphabet::Crockford, &hash.to_le_bytes());
    format!("\"{encoded}\"")
}

fn path_to_normalized_string(path: &Path) -> String {
    match path.to_str() {
        Some(s) => {
            #[cfg(target_os = "windows")]
            let s = s.replace('\\', "/");
            s.to_string()
        }
        None => panic!("Path is not UTF-8: {path:?}"),
    }
}

fn pathbuf_to_string(path: PathBuf) -> String {
    match path.into_os_string().into_string() {
        Ok(s) => s,
        Err(s) => panic!("Path is not UTF-8: {s:?}"),
    }
}
