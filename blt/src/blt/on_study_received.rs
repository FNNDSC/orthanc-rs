use orthanc_sdk::api::DicomClient;
use orthanc_sdk::api::types::{JobId, StudyId};
use orthanc_sdk::bindings::OrthancPluginContext;
use orthanc_sdk::openapi::{
    PatientsIdAnonymizePostRequest as AnonymizePostRequest, ToolsFindPostRequest,
};

use super::BltDatabase;
use super::error::{DoNothing, TraceAndReturn};
use super::models::{AccessionNumber, BltStudy};

/// Allow-list of DICOM tags to keep during anonymization of study.
const TAGS_TO_KEEP: [&'static str; 2] = ["StudyDescription", "SeriesDescription"];

/// Enqueue a job to anonymize the study.
pub(crate) fn on_study_received(
    context: *mut OrthancPluginContext,
    study_instance_uid: String,
    db: &mut BltDatabase,
) -> TraceAndReturn {
    for study in get_series_of_retrieve_job(context, study_instance_uid)? {
        let accession_number = study.requested_tags.accession_number;
        if let Some(blt_request) = db.get(&accession_number) {
            let job_id = anonymize_study(context, study.id, blt_request)?;
            db.add_anonymization(job_id, accession_number);
        } else {
            tracing::error!(
                AccessionNumber = accession_number.to_string(),
                "no BLT study found (this is a bug)"
            );
            return Err(DoNothing);
        }
    }
    Ok(())
}

fn get_series_of_retrieve_job(
    context: *mut OrthancPluginContext,
    study_instance_uid: String,
) -> Result<Vec<StudyDetails>, DoNothing> {
    let request = FindByStudyUID(study_instance_uid);
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

/// A request to find a study by `StudyInstanceUID`.
struct FindByStudyUID(pub String);

impl From<FindByStudyUID> for ToolsFindPostRequest {
    fn from(value: FindByStudyUID) -> Self {
        ToolsFindPostRequest {
            case_sensitive: Some(false),
            expand: Some(true),
            level: Some("Study".to_string()),
            response_content: Some(vec!["RequestedTags".to_string()]),
            query: Some(serde_json::json!({"StudyInstanceUID": value.0})),
            requested_tags: Some(vec!["AccessionNumber".to_string()]),
            ..Default::default()
        }
    }
}

impl orthanc_sdk::api::Find for FindByStudyUID {
    type Item = StudyDetails;
}

#[derive(serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
struct StudyDetails {
    #[serde(rename = "ID")]
    id: StudyId,
    #[serde(rename = "RequestedTags")]
    requested_tags: StudyRequestedTags,
}

#[derive(serde::Deserialize, Clone, Debug)]
struct StudyRequestedTags {
    #[serde(rename = "AccessionNumber")]
    accession_number: AccessionNumber,
}
