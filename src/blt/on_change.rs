use crate::blt::BltDatabase;
use crate::orthanc::api::{JobContent, JobId, JobState, JobsClient, JsonResponseError};
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
                let _ = on_job_success(context, db, JobId::new(id));
            } else {
                tracing::warn!("resource_id is null");
            }
        }
        _ => (),
    }
}

fn on_job_success(
    context: *mut bindings::OrthancPluginContext,
    db: &mut BltDatabase,
    id: JobId,
) -> Result<(), ()> {
    if !db.has_retrieve(&id) {
        return Ok(());
    }
    let job = JobsClient::new(context).get(id).data().map_err(|e| {
        e.trace();
    })?;
    assert_eq!(job.state, JobState::Success);
    let study_instance_uid = if let JobContent::DicomMoveScu { query, .. } = job.content
        && let Some(study_instance_uid) = query.first().and_then(|q| q.study_instance_uid())
    {
        tracing::info!(
            StudyInstanceUID = study_instance_uid,
            "I should now filter, anonymize, and push this."
        );
    } else {
        tracing::error!(
            job = job.id,
            "job was not a DicomMoveScu operation with StudyInstanceUID in its content"
        );
        return Err(());
    };

    Ok(())
}

fn filter_received_series(context: *mut bindings::OrthancPluginContext) {
    // TODO Sandip excludes series with: {"SOPClassUID": "Secondary Capture Image Storage"}
    todo!()
}
