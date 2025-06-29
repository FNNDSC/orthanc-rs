#![allow(non_snake_case)]

use std::collections::HashMap;
use crate::orthanc;
use crate::orthanc::callback::{create_json_rest_callback, register_on_change, register_rest, register_rest_no_lock};
use crate::orthanc::{OrthancPluginHttpRequest, OrthancPluginRestOutput};
use std::sync::RwLock;
use crate::blt::{AccessionNumber, BltStudy};

static GLOBAL_STATE: RwLock<AppState> = RwLock::new(AppState { context: None, blt_requests: None });

struct AppState {
    context: Option<OrthancContext>,
    blt_requests: Option<HashMap<AccessionNumber, BltStudy>>
}

struct OrthancContext(*mut orthanc::OrthancPluginContext);
unsafe impl Send for OrthancContext {}
unsafe impl Sync for OrthancContext {}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "C" fn OrthancPluginInitialize(context: *mut orthanc::OrthancPluginContext) -> i32 {
    if let Err(e) = tracing::subscriber::set_global_default(
        tracing_subscriber::FmtSubscriber::builder()
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
    app_state.blt_requests = Some(HashMap::with_capacity(1000));

    register_on_change(context, Some(on_change));
    register_rest(context, "/blt/studies", Some(rest_callback));
    0
}

#[unsafe(no_mangle)]
pub extern "C" fn OrthancPluginFinalize() {
    let mut app_state = GLOBAL_STATE.try_write().expect("unable to obtain lock");
    if let Some(hashmap) = app_state.blt_requests.take() {
        drop(hashmap);
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
    change_type: orthanc::OrthancPluginChangeType,
    resource_type: orthanc::OrthancPluginResourceType,
    resource_id: *const std::os::raw::c_char,
) -> orthanc::OrthancPluginErrorCode {
    if change_type != orthanc::OrthancPluginChangeType_OrthancPluginChangeType_StableSeries {
        return orthanc::OrthancPluginErrorCode_OrthancPluginErrorCode_Success;
    }
    if resource_type != orthanc::OrthancPluginResourceType_OrthancPluginResourceType_Series {
        eprintln!(
            "BUG DETECTED by plugin neochris-notifier: change type is StableSeries, but ResourceType is {resource_type}"
        );
        return orthanc::OrthancPluginErrorCode_OrthancPluginErrorCode_Success;
    }

    let resource_id = if resource_id.is_null() {
        None
    } else {
        match unsafe { std::ffi::CStr::from_ptr(resource_id) }.to_str() {
            Ok(cstr) => Some(cstr.to_string()),
            Err(e) => {
                eprintln!(
                    "BUG IN plugin neochris-notifier: unable to parse resource_id to Utf8-String - {e}"
                );
                None
            }
        }
    };

    if let Some(resource_id) = resource_id {
        let uri = format!("http://hello:5678/orthanc/{change_type}/{resource_type}/{resource_id}");
        eprintln!("{uri}");
        orthanc::OrthancPluginErrorCode_OrthancPluginErrorCode_Success
        // match ureq::get(uri).call() {
        //     Ok(_) => orthanc::OrthancPluginErrorCode_OrthancPluginErrorCode_Success,
        //     Err(_) => orthanc::OrthancPluginErrorCode_OrthancPluginErrorCode_InternalError
        // }
    } else {
        orthanc::OrthancPluginErrorCode_OrthancPluginErrorCode_Success
        // orthanc::OrthancPluginErrorCode_OrthancPluginErrorCode_InternalError
    }
}

extern "C" fn rest_callback(
    output: *mut OrthancPluginRestOutput,
    url: *const std::os::raw::c_char,
    request: *const OrthancPluginHttpRequest,
) -> orthanc::OrthancPluginErrorCode {
    match GLOBAL_STATE.try_write() {
        Ok(mut app_state) => {
            let context = app_state.context.as_ref().unwrap().0;
            let blt_studies = app_state.blt_requests.as_mut().unwrap();
            create_json_rest_callback(
                context,
                output,
                url,
                request,
                |req| crate::blt::route_http_request(req, blt_studies),
            )
        }
        Err(_e) => {
            tracing::error!("Failed to read GLOBAL_STATE");
            orthanc::OrthancPluginErrorCode_OrthancPluginErrorCode_InternalError
        }
    }
}
