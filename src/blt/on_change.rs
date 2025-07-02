use crate::blt::BltDatabase;
use crate::orthanc::api::JobId;
use crate::orthanc::{OnChangeEvent, bindings};

pub fn on_change(
    context: *mut bindings::OrthancPluginContext,
    db: &mut BltDatabase,
    OnChangeEvent {
        change_type,
        resource_type,
        resource_id,
    }: OnChangeEvent,
) {
    match change_type {
        bindings::OrthancPluginChangeType_OrthancPluginChangeType_JobSuccess => {
            if let Some(id) = resource_id {
                on_job_success(context, db, JobId::new(id))
            } else {
                tracing::warn!("resource_id is null");
            }
        }
        _ => (),
    }
}

pub fn on_job_success(
    context: *mut bindings::OrthancPluginContext,
    db: &mut BltDatabase,
    id: JobId,
) {
    if !db.has_retrieve(&id) {
        return;
    }
    tracing::info!("I should now filter, anonymize, and push this study.");
}
