use crate::blt::BltDatabase;
use crate::blt::error::{DoNothing, TraceAndReturn};
use crate::blt::filter::filter_received_series;
use crate::blt::get_accession_number::get_accession_number;
use crate::blt::series_of_study::{FindSeriesByStudy, SeriesOfStudy};
use orthanc_sdk::api::types::{JobContent, JobId, JobInfo, JobState, MoveScuJobQueryAny, StudyId};
use orthanc_sdk::api::{DicomClient, GeneralClient};
use orthanc_sdk::bindings::OrthancPluginContext;
use orthanc_sdk::openapi::PatientsIdAnonymizePostRequest as AnonymizePostRequest;
use orthanc_sdk::{bindings, on_change::OnChangeEvent};

use super::models::BltStudy;

const TAGS_TO_KEEP: [&'static str; 2] = ["StudyDescription", "SeriesDescription"];

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
    let job = GeneralClient::new(context).get(id).data()?;
    assert_eq!(job.state, JobState::Success);
    for study in get_series_of_retrieve_job(context, job)? {
        let deleted_count = filter_received_series(context, &study.series)?;
        if deleted_count == study.series.len() {
            tracing::warn!(study = study.id.to_string(), "all series were deleted");
            return Err(DoNothing);
        }
        let accession_number = get_accession_number(context, study.id.clone())?;
        if let Some(blt_request) = db.get(&accession_number) {
            let job_id = anonymize_study(context, study.id, blt_request)?;
            db.add_anonymization(job_id, accession_number);
        } else {
            tracing::error!(
                AccessionNumber = accession_number.to_string(),
                "study not found in BLT database (this is a bug)"
            );
            return Err(DoNothing);
        }
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
    let client = DicomClient::new(context);
    let data = client.find(request).into_result()?;
    Ok(data)
}

fn anonymize_study(
    context: *mut OrthancPluginContext,
    study: StudyId,
    blt_request: &BltStudy,
) -> Result<JobId, DoNothing> {
    let client = DicomClient::new(context);
    let replacements = serde_json::json!({
        "PatientID": blt_request.anon_patient_id,
        "PatientName": blt_request.anon_patient_name,
        "PatientBirthDate": blt_request.anon_patient_birth_date,
        "AccessionNumber": blt_request.anon_accession_number,
    });
    let keep = TAGS_TO_KEEP.iter().map(|s| s.to_string()).collect();
    let request = AnonymizePostRequest {
        keep_source: Some(false),
        force: Some(true), // required to modify PatientID
        replace: Some(replacements),
        keep: Some(keep),
        ..Default::default()
    };
    let job = client.anonymize(study, request).into_result()?;
    tracing::info!(job = job.id.to_string(), "anonymizing study");
    Ok(job.id)
}
