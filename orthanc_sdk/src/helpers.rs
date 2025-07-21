use crate::bindings;

/// Translation of the C code which appears as the last line of most functions in `OrthancCPlugin.h`,
/// e.g. <https://orthanc.uclouvain.be/sdk/OrthancCPlugin_8h_source.html#l03056>
///
/// ```c
/// context->InvokeService(context, service, &params);
/// ```
pub(crate) fn invoke_service<T>(
    context: *mut bindings::OrthancPluginContext,
    service: bindings::_OrthancPluginService,
    params: T,
) -> bindings::OrthancPluginErrorCode {
    let boxed = Box::new(params);
    let params: *const std::ffi::c_void = Box::into_raw(boxed) as *mut std::ffi::c_void;
    unsafe {
        let invoker = (*context).InvokeService;
        invoker.unwrap()(context, service, params)
    }
}

/// Calls [invoke_service], panics if unsuccessful.
pub(crate) fn must_invoke_service<T>(
    context: *mut bindings::OrthancPluginContext,
    service: bindings::_OrthancPluginService,
    params: T,
) {
    let code = invoke_service(context, service, params);
    if code != bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_Success {
        panic!("Unsuccessful service invocation from Orthanc plugin (code {code})")
    }
}

/// Translation of [OrthancPluginFreeMemoryBuffer](https://orthanc.uclouvain.be/sdk/OrthancCPlugin_8h_source.html#l02241)
pub(crate) unsafe fn free_memory_buffer(
    context: *mut bindings::OrthancPluginContext,
    buffer: *mut bindings::OrthancPluginMemoryBuffer,
) {
    unsafe { (*context).Free.unwrap()((*buffer).data) }
}

/// Create a raw pointer to a memory buffer.
pub(crate) fn create_empty_buffer() -> *mut bindings::OrthancPluginMemoryBuffer {
    let data = std::ptr::null_mut();
    let buf = bindings::OrthancPluginMemoryBuffer { data, size: 0 };
    let boxed = Box::new(buf);
    Box::into_raw(boxed)
}

/// Translation of [OrthancPluginFreeString](https://orthanc.uclouvain.be/sdk/OrthancCPlugin_8h_source.html#l02079).
pub(crate) unsafe fn free_string(
    context: *mut bindings::OrthancPluginContext,
    buffer: std::mem::MaybeUninit<*mut std::ffi::c_char>,
) {
    unsafe { (*context).Free.unwrap()(buffer.assume_init() as *mut std::ffi::c_void) }
}

// pub(crate) unsafe fn create_memory_buffer(
//     context: *mut bindings::OrthancPluginContext,
// ) -> *mut bindings::OrthancPluginMemoryBuffer {
//     let data = std::ptr::null_mut();
//     let target = bindings::OrthancPluginMemoryBuffer { data, size: 0 };
//     let boxed_target = Box::new(target);
//     let params = bindings::_OrthancPluginCreateMemoryBuffer {
//         size: 0,
//         target: Box::into_raw(boxed_target),
//     };
//     must_invoke_service(
//         context,
//         bindings::_OrthancPluginService__OrthancPluginService_CreateMemoryBuffer,
//         Box::new(params),
//     );
// }
