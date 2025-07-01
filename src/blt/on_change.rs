use crate::blt::BltDatabase;
use crate::orthanc::bindings;

pub fn on_change(
    context: *mut bindings::OrthancPluginContext,
    db: &mut BltDatabase,
    change_type: bindings::OrthancPluginChangeType,
    resource_type: bindings::OrthancPluginResourceType,
    resource_id: String,
) -> bindings::OrthancPluginErrorCode {
    bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_Success
}
