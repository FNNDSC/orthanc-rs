use crate::blt::BltDatabase;
use crate::blt::series_of_study::{FindSeriesByStudy, SeriesOfStudy};
use orthanc_sdk::api::Find;
use orthanc_sdk::api::types::{
    JobContent, JobId, JobInfo, JobState, MoveScuJobQueryAny, ResourceId,
};
use orthanc_sdk::{bindings, on_change::OnChangeEvent};

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
    let job = id.get(context).data().map_err(|e| {
        e.trace();
    })?;
    assert_eq!(job.state, JobState::Success);
    let study = get_series_of_retrieve_job(context, job)?;
    Ok(())
}

fn get_series_of_retrieve_job(
    context: *mut bindings::OrthancPluginContext,
    job: JobInfo,
) -> Result<Vec<SeriesOfStudy>, ()> {
    let study_instance_uid = if let JobContent::DicomMoveScu { query, .. } = job.content
        && let Some(query) = query.into_iter().next().map(MoveScuJobQueryAny::from)
        && let Some(study_instance_uid) = query.study_instance_uid
    {
        study_instance_uid
    } else {
        tracing::error!(
            job = job.id,
            "job was not a DicomMoveScu operation with StudyInstanceUID in its content"
        );
        return Err(());
    };
    let request = FindSeriesByStudy(study_instance_uid);
    request.find(context).into_result().map_err(|e| {
        e.trace();
    })
}

fn filter_received_series(context: *mut bindings::OrthancPluginContext) {
    // TODO Sandip excludes series with: {"SOPClassUID": "Secondary Capture Image Storage"}
    todo!()
}
