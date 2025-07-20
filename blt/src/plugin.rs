#![allow(non_snake_case)]

use crate::blt::BltDatabase;
use orthanc_sdk::bindings;
use orthanc_sdk::callback::{create_json_rest_callback, register_on_change, register_rest};
use orthanc_sdk::on_change::{OnChangeEvent, OnChangeThread};
use std::sync::{Mutex, RwLock};

static GLOBAL_STATE: RwLock<AppState> = RwLock::new(AppState {
    context: None,
    on_change_thread: None,
});

static DATABASE: Mutex<Option<BltDatabase>> = Mutex::new(None);

struct AppState {
    context: Option<OrthancContext>,
    on_change_thread: Option<OnChangeThread>,
}

/// BLT plugin name.
///
/// NOTE: this is a macro instead of `const` so that it can be used with [concat].
macro_rules! plugin_name {
    () => {
        "blt"
    };
}

struct OrthancContext(*mut bindings::OrthancPluginContext);
unsafe impl Send for OrthancContext {}
unsafe impl Sync for OrthancContext {}

/// Orthanc configuration file.
#[derive(serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct OrthancConfig {
    blt: Option<OrthancBltPluginConfig>,
}

/// BLT plugin configuration section of Orthanc configuration file.
#[derive(serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct OrthancBltPluginConfig {
    verbose: Option<bool>,
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "C" fn OrthancPluginInitialize(
    context: *mut bindings::OrthancPluginContext,
) -> bindings::OrthancPluginErrorCode {
    let logger = orthanc_sdk::OrthancLogger {
        context,
        plugin_name: plugin_name!(),
        verbose: get_config(context).and_then(|c| c.verbose).unwrap_or(false),
    };
    if let Err(e) = tracing::subscriber::set_global_default(logger)
        && !e.to_string().contains("has already been set")
    {
        eprintln!("Failed to initialize logging in Rust plugin: {e}");
    }

    let mut app_state = GLOBAL_STATE.try_write().unwrap();
    app_state.context = Some(OrthancContext(context));

    let mut db_mutex = DATABASE.lock().unwrap();
    let _ = db_mutex.insert(BltDatabase::with_capacity(1000));

    app_state.on_change_thread = Some(OnChangeThread::spawn(move |event| {
        let app_state = GLOBAL_STATE.try_read().unwrap();
        let context = app_state.context.as_ref().unwrap().0;
        // NOTE: mutex is being held for a "long" time in on_change because it does
        //       synchronous calls to the Orthanc built-in API. This is yet another
        //       issue which will magically go away by replacing the in-process db
        //       with ValKey.
        let mut db_mutex = DATABASE.lock().unwrap();
        let database = db_mutex.as_mut().unwrap();
        crate::blt::on_change(context, database, event);
    }));

    register_on_change(context, Some(on_change));
    register_rest(context, "/blt/studies", Some(rest_callback));

    bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_Success
}

fn get_config(context: *mut bindings::OrthancPluginContext) -> Option<OrthancBltPluginConfig> {
    let buffer = orthanc_sdk::get_configuration(context)?;
    let config: OrthancConfig = buffer.deserialize().ok()?;
    config.blt
}

#[unsafe(no_mangle)]
pub extern "C" fn OrthancPluginFinalize() {
    let mut app_state = GLOBAL_STATE.try_write().expect("unable to obtain lock");
    if let Some(thread) = app_state.on_change_thread.take() {
        thread.join().unwrap()
    }
    let mut db_mutex = DATABASE.lock().unwrap();
    if let Some(hashmap) = db_mutex.take() {
        drop(hashmap);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn OrthancPluginGetName() -> *const u8 {
    concat!(plugin_name!(), "\0").as_ptr()
}

#[unsafe(no_mangle)]
pub extern "C" fn OrthancPluginGetVersion() -> *const u8 {
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr()
}

extern "C" fn on_change(
    change_type: bindings::OrthancPluginChangeType,
    resource_type: bindings::OrthancPluginResourceType,
    resource_id: *const std::os::raw::c_char,
) -> bindings::OrthancPluginErrorCode {
    let resource_id = if resource_id.is_null() {
        None
    } else if let Ok(cstr) = unsafe { std::ffi::CStr::from_ptr(resource_id).to_str() } {
        Some(cstr.to_string())
    } else {
        tracing::warn!("resource_id is not UTF-8");
        return bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_InternalError;
    };
    if let Ok(app) = GLOBAL_STATE.try_read()
        && let Some(channel) = &app.on_change_thread
    {
        let event = OnChangeEvent {
            change_type,
            resource_type,
            resource_id,
        };
        if channel.send(event).is_ok() {
            bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_Success
        } else {
            bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_InternalError
        }
    } else {
        bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_InternalError
    }
}

extern "C" fn rest_callback(
    output: *mut bindings::OrthancPluginRestOutput,
    url: *const std::os::raw::c_char,
    request: *const bindings::OrthancPluginHttpRequest,
) -> bindings::OrthancPluginErrorCode {
    let app_state = if let Ok(app_state) = GLOBAL_STATE.try_read() {
        app_state
    } else {
        tracing::error!("Failed to read GLOBAL_STATE");
        return bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_InternalError;
    };
    let mut db_mutex = if let Ok(db_mutex) = DATABASE.lock() {
        db_mutex
    } else {
        tracing::error!("Failed to lock database mutex, did a background thread panic?");
        return bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_InternalError;
    };
    let context = app_state.context.as_ref().unwrap().0;
    let database = db_mutex.as_mut().unwrap();
    create_json_rest_callback(context, output, url, request, |req| {
        crate::blt::route_http_request(context, req, database)
    })
}
