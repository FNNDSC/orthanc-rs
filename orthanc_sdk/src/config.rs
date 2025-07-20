use std::mem::MaybeUninit;

use crate::{
    bindings,
    helpers::{free_string, invoke_service},
};

/// Return the content of the configuration file(s).
///
/// Rust-friendly wrapper to a translation of
/// [OrthancPluginGetConfiguration](https://orthanc.uclouvain.be/hg/orthanc/file/Orthanc-1.12.8/OrthancServer/Plugins/Include/orthanc/OrthancCPlugin.h#l3961).
pub fn get_configuration(
    context: *mut bindings::OrthancPluginContext,
) -> Option<OrthancConfigurationBuffer> {
    get_configuration_raw(context).map(|buffer| OrthancConfigurationBuffer { context, buffer })
}

/// Translation of [OrthancPluginGetConfiguration](https://orthanc.uclouvain.be/hg/orthanc/file/Orthanc-1.12.8/OrthancServer/Plugins/Include/orthanc/OrthancCPlugin.h#l3961).
fn get_configuration_raw(
    context: *mut bindings::OrthancPluginContext,
) -> Option<MaybeUninit<*mut std::ffi::c_char>> {
    let mut buffer = MaybeUninit::<*mut std::ffi::c_char>::uninit();
    let params = bindings::_OrthancPluginRetrieveDynamicString {
        argument: std::ptr::null(),
        result: buffer.as_mut_ptr(),
    };
    invoke_service(
        context,
        bindings::_OrthancPluginService__OrthancPluginService_GetConfiguration,
        params,
    );
    if buffer.as_ptr().is_null() {
        None
    } else {
        Some(buffer)
    }
}

/// A wrapper for the pointer to the Orthanc configuration as JSON string.
pub struct OrthancConfigurationBuffer {
    context: *mut bindings::OrthancPluginContext,
    buffer: MaybeUninit<*mut std::ffi::c_char>,
}

impl OrthancConfigurationBuffer {
    /// Deserialize the Orthanc JSON config.
    pub fn deserialize<'a, T: serde::Deserialize<'a>>(&self) -> serde_json::Result<T> {
        let c_str = unsafe { std::ffi::CStr::from_ptr(self.buffer.assume_init()) };
        let bytes = c_str.to_bytes();
        serde_json::from_slice(bytes)
    }
}

impl Drop for OrthancConfigurationBuffer {
    fn drop(&mut self) {
        unsafe { free_string(self.context, self.buffer) }
    }
}
