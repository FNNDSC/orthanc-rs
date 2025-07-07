use orthanc_sdk::api::DicomClient;
use orthanc_sdk::api::types::{JobId, RequestedTags, SeriesId, Study, StudyId};
use orthanc_sdk::bindings::OrthancPluginContext;
use orthanc_sdk::openapi::{
    PatientsIdAnonymizePostRequest as AnonymizePostRequest, ToolsFindPostRequest,
};

use super::BltDatabase;
use super::error::{DoNothing, TraceAndReturn};
use super::filter::filter_received_series;
use super::models::{AccessionNumber, BltStudy};

/// Allow-list of DICOM tags to keep during anonymization of study.
const TAGS_TO_KEEP: [&'static str; 2] = ["StudyDescription", "SeriesDescription"];

/// Filter series of the study, deleting series which are undesirable to push as per the BLT protocol.
/// Then, enqueue a job to anonymize the study.
pub(crate) fn on_study_received(
    context: *mut OrthancPluginContext,
    study_instance_uid: String,
    db: &mut BltDatabase,
) -> TraceAndReturn {
    for study in get_series_of_retrieve_job(context, study_instance_uid)? {
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
    study_instance_uid: String,
) -> Result<Vec<SeriesOfStudy>, DoNothing> {
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

pub(crate) fn get_accession_number(
    context: *mut OrthancPluginContext,
    study_id: StudyId,
) -> Result<AccessionNumber, DoNothing> {
    let client = DicomClient::new(context);
    let study: Study<Details> = client.get(study_id).ok_data()?;
    Ok(study.requested_tags.accession_number)
}

/// AccessionNumber of a DICOM study
#[derive(serde::Deserialize, Debug, Clone, PartialEq)]
struct Details {
    #[serde(rename = "AccessionNumber")]
    accession_number: AccessionNumber,
}

impl RequestedTags for Details {
    fn names() -> &'static [&'static str] {
        &["AccessionNumber"]
    }
}
/// List of series for a study.
#[derive(serde::Deserialize, Clone, Debug)]
#[serde(rename_all = "PascalCase")]
pub(crate) struct SeriesOfStudy {
    /// Study ID
    #[serde(rename = "ID")]
    pub id: StudyId,
    /// Series IDs
    pub series: Vec<SeriesId>,
}

/// A request to find series of a study by StudyInstanceUID.
pub(crate) struct FindSeriesByStudy(pub String);

impl From<FindSeriesByStudy> for ToolsFindPostRequest {
    fn from(value: FindSeriesByStudy) -> Self {
        ToolsFindPostRequest {
            case_sensitive: Some(false),
            expand: Some(true),
            level: Some("Study".to_string()),
            response_content: Some(vec!["Children".to_string()]),
            query: Some(serde_json::json!({"StudyInstanceUID": value.0})),
            ..Default::default()
        }
    }
}

impl orthanc_sdk::api::Find for FindSeriesByStudy {
    type Item = SeriesOfStudy;
}
