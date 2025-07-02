use crate::blt::BltDatabase;
use crate::orthanc::{OnChangeEvent, bindings};
use crate::orthanc::api::JobId;

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
            on_job_success(context, db, JobId::new(resource_id))
        }
        _ => (),
    }
}

pub fn on_job_success(
    context: *mut bindings::OrthancPluginContext,
    db: &mut BltDatabase,
    id: JobId
) {
    if !db.has_retrieve(&id) {
        return;
    }
    tracing::info!("I should now filter, anonymize, and push this study.");
}