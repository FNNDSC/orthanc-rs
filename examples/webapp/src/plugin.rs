//! Example and test case for `webapp` feature.

#![allow(non_snake_case)]

use std::sync::RwLock;

use include_dir::include_dir;
use include_webdir::{CWebBundle, include_cwebdir};
use orthanc_sdk::bindings;

struct OrthancContext(*mut bindings::OrthancPluginContext);
unsafe impl Send for OrthancContext {}
unsafe impl Sync for OrthancContext {}

const DIST: include_dir::Dir = include_dir!("$CARGO_MANIFEST_DIR/dist");
const PREPARED: CWebBundle = include_cwebdir!("$CARGO_MANIFEST_DIR/dist");

static GLOBAL_STATE: RwLock<Option<OrthancContext>> = RwLock::new(None);

#[unsafe(no_mangle)]
pub extern "C" fn OrthancPluginGetName() -> *const u8 {
    c"example_plugin_webapp".as_ptr() as *const _
}

#[unsafe(no_mangle)]
pub extern "C" fn OrthancPluginGetVersion() -> *const u8 {
    c"0.0.0".as_ptr() as *const _
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "C" fn OrthancPluginInitialize(
    context: *mut bindings::OrthancPluginContext,
) -> bindings::OrthancPluginErrorCode {
    orthanc_sdk::register_rest_no_lock(context, c"/simple/?(.*)", Some(serve_simple));
    orthanc_sdk::register_rest_no_lock(context, c"/prepared/?(.*)", Some(serve_prepared));
    let mut global_state = GLOBAL_STATE.try_write().unwrap();
    *global_state = Some(OrthancContext(context));
    bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_Success
}

#[unsafe(no_mangle)]
pub extern "C" fn OrthancPluginFinalize() {
    let mut app_state = GLOBAL_STATE.try_write().unwrap();
    *app_state = None;
}

#[unsafe(no_mangle)]
extern "C" fn serve_simple(
    output: *mut bindings::OrthancPluginRestOutput,
    _url: *const std::ffi::c_char,
    request: *const bindings::OrthancPluginHttpRequest,
) -> bindings::OrthancPluginErrorCode {
    if let Ok(app_state) = GLOBAL_STATE.try_read()
        && let Some(context) = app_state.as_ref()
    {
        orthanc_sdk::webapp::serve_static_file(context.0, output, request, &DIST)
    } else {
        bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_InternalError
    }
}

#[unsafe(no_mangle)]
extern "C" fn serve_prepared(
    output: *mut bindings::OrthancPluginRestOutput,
    _url: *const std::ffi::c_char,
    request: *const bindings::OrthancPluginHttpRequest,
) -> bindings::OrthancPluginErrorCode {
    if let Ok(app_state) = GLOBAL_STATE.try_read()
        && let Some(context) = app_state.as_ref()
    {
        orthanc_sdk::webapp::serve_static_file(context.0, output, request, &PREPARED)
    } else {
        bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_InternalError
    }
}
