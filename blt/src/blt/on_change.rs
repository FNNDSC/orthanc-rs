use crate::blt::BltDatabase;
use crate::blt::error::{DoNothing, TraceAndReturn};
use crate::blt::filter::filter_received_series;
use crate::blt::series_of_study::{FindSeriesByStudy, SeriesOfStudy};
use orthanc_sdk::api::Find;
use orthanc_sdk::api::types::{JobContent, JobId, JobInfo, JobState, MoveScuJobQueryAny, StudyId, SystemResourceId};
use orthanc_sdk::bindings::OrthancPluginContext;
use orthanc_sdk::{bindings, on_change::OnChangeEvent};

pub fn on_change(
    context: *mut OrthancPluginContext,
    db: &mut BltDatabase,
    OnChangeEvent {
        change_type,
        resource_type: _resource_type,
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
    context: *mut OrthancPluginContext,
    db: &mut BltDatabase,
    id: JobId,
) -> TraceAndReturn {
    if !db.has_retrieve(&id) {
        return Ok(());
    }
    let job = id.get(context).data()?;
    assert_eq!(job.state, JobState::Success);
    for study in get_series_of_retrieve_job(context, job)? {
        filter_received_series(context, &study.series)?;
        anonymize_study(context, study.id);
    }
    Ok(())
}

fn get_series_of_retrieve_job(
    context: *mut OrthancPluginContext,
    job: JobInfo,
) -> Result<Vec<SeriesOfStudy>, DoNothing> {
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
        return Err(DoNothing);
    };
    let request = FindSeriesByStudy(study_instance_uid);
    let data = request.find(context).into_result()?;
    Ok(data)
}

fn anonymize_study(context: *mut OrthancPluginContext, study: StudyId) {
    let _ = context; // TODO
    tracing::info!(
        study = study.to_string(),
        "I should anonymize this study now"
    );
}
