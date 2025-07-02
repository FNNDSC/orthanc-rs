#![allow(non_snake_case)]

use crate::blt::BltDatabase;
use crate::orthanc::callback::{create_json_rest_callback, register_on_change, register_rest};
use crate::orthanc::{OnChangeEvent, OnChangeThread, bindings};
use std::sync::RwLock;

static GLOBAL_STATE: RwLock<AppState> = RwLock::new(AppState {
    context: None,
    database: None,
    on_change_thread: None,
});

struct AppState {
    context: Option<OrthancContext>,
    database: Option<BltDatabase>,
    on_change_thread: Option<OnChangeThread>,
}

struct OrthancContext(*mut bindings::OrthancPluginContext);
unsafe impl Send for OrthancContext {}
unsafe impl Sync for OrthancContext {}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "C" fn OrthancPluginInitialize(context: *mut bindings::OrthancPluginContext) -> i32 {
    if let Err(e) = tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
            // TODO set verbosity from Orthanc JSON
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .finish(),
    ) {
        eprintln!("Failed to initialize logging in Rust plugin: {e}");
        return 1;
    }

    let mut app_state = GLOBAL_STATE
        .try_write()
        .expect("Cannot write to GLOBAL_STATE");
    app_state.context = Some(OrthancContext(context));
    app_state.database = Some(BltDatabase::with_capacity(1000));

    app_state.on_change_thread = Some(OnChangeThread::spawn(move |event| {
        let mut app_state = GLOBAL_STATE
            .try_write()
            .expect("Cannot write to GLOBAL_STATE");
        let context = app_state.context.as_ref().unwrap().0;
        let database = app_state.database.as_mut().unwrap();
        crate::blt::on_change(context, database, event);
    }));

    register_on_change(context, Some(on_change));
    register_rest(context, "/blt/studies", Some(rest_callback));

    bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_Success
}

#[unsafe(no_mangle)]
pub extern "C" fn OrthancPluginFinalize() {
    let mut app_state = GLOBAL_STATE.try_write().expect("unable to obtain lock");
    if let Some(hashmap) = app_state.database.take() {
        drop(hashmap);
    }
    if let Some(thread) = app_state.on_change_thread.take() {
        thread.join().unwrap()
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn OrthancPluginGetName() -> *const u8 {
    "neochris-notifier\0".as_ptr()
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
    match GLOBAL_STATE.try_write() {
        Ok(mut app_state) => {
            let context = app_state.context.as_ref().unwrap().0;
            let blt_studies = app_state.database.as_mut().unwrap();
            create_json_rest_callback(context, output, url, request, |req| {
                crate::blt::route_http_request(context, req, blt_studies)
            })
        }
        Err(_e) => {
            tracing::error!("Failed to read GLOBAL_STATE");
            bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_InternalError
        }
    }
}
