use crate::blt::BltDatabase;
use crate::blt::error::{DoNothing, TraceAndReturn};
use crate::blt::on_study_received::on_study_received;
use orthanc_sdk::api::GeneralClient;
use orthanc_sdk::api::types::{
    JobContent, JobId, JobState, MoveScuJobQueryAny, ResourceModificationContent,
};
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
    let job = GeneralClient::new(context).get(id).ok_data()?;
    assert_eq!(job.state, JobState::Success);
    match job.content {
        JobContent::DicomMoveScu { query, .. } => {
            if !db.has_retrieve(&job.id) {
                return Ok(());
            }
            if let Some(study_instance_uid) = query
                .into_iter()
                .map(MoveScuJobQueryAny::from)
                .next()
                .and_then(|q| q.study_instance_uid)
            {
                on_study_received(context, study_instance_uid, db)
            } else {
                tracing::error!(
                    job = job.id.to_string(),
                    "query does not contain first element with StudyInstanceUID"
                );
                Err(DoNothing)
            }
        }
        JobContent::ResourceModification(modification) => {
            if !db.has_anonymization(&job.id) {
                return Ok(());
            }
            match modification {
                ResourceModificationContent::Study(modification) => {
                    tracing::info!("I should send this study to the peer Orthanc.");
                    Ok(())
                }
                _ => Ok(()),
            }
        }
        _ => Ok(()),
    }
}
