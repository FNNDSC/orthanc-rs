#![allow(non_snake_case)]

use crate::orthanc;

/// Wrapper struct for a callback function whose FFI will be generated automatically by `bindgen`.
#[repr(C)]
struct OnChangeParams {
    callback: orthanc::OrthancPluginOnChangeCallback,
}

#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "C" fn OrthancPluginInitialize(context: *mut orthanc::OrthancPluginContext) -> i32 {
    // let mut app_state = GLOBAL_STATE.try_write().expect("unable to obtain lock");
    // app_state.context = Some(OrthancContext(context));

    let params = Box::new(OnChangeParams {
        callback: Some(on_change),
    });

    let params: *const std::ffi::c_void = Box::into_raw(params) as *mut std::ffi::c_void;
    unsafe {
        let invoker = (*context).InvokeService;
        invoker.unwrap()(
            context,
            orthanc::_OrthancPluginService__OrthancPluginService_RegisterOnChangeCallback,
            params,
        );
    }

    println!("HELLO I have registered the callback.");

    0
}

#[unsafe(no_mangle)]
pub extern "C" fn OrthancPluginFinalize() {
    // let mut app_state = GLOBAL_STATE.try_write().expect("unable to obtain lock");
    // if let Some(runtime) = app_state.runtime.take() {
    //     runtime.shutdown_timeout(Duration::from_secs(5));
    // }
    //
    // //
    // // Give background runtime time to clean up
    // //
    // std::thread::sleep(Duration::from_secs(5));
    //
    // info!("finalized");
}

#[unsafe(no_mangle)]
pub extern "C" fn OrthancPluginGetName() -> *const u8 {
    // info!("OrthancPluginGetName");
    "neochris-notifier\0".as_ptr()
}

#[unsafe(no_mangle)]
pub extern "C" fn OrthancPluginGetVersion() -> *const u8 {
    // info!("OrthancPluginGetVersion");
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
