//! BLT Protocol:
//!
//! User submits [BltStudy] to this plugin. The protocol is to:
//!
//! 1. Query for study in PACS by AccessionNumber
//! 2. Retrieve study from PACS
//! 3. Anonymize study DICOM
//! 4. Push to an Orthanc peer

use crate::dicom_date::DicomDate;
use crate::orthanc::bindings::OrthancPluginContext;
use crate::orthanc::http::{Method, Request, Response};
use http::StatusCode;
use std::collections::HashMap;

/// Request for a study under the BLT protocol.

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub(crate) struct BltStudy {
    #[serde(rename = "MRN")]
    patient_id: String,
    #[serde(rename = "Anon_PatientID")]
    anon_patient_id: String,
    #[serde(rename = "PatientName")]
    patient_name: String,
    #[serde(rename = "Anon_PatientName")]
    anon_patient_name: String,
    #[serde(rename = "PatientBirthDate")]
    patient_birth_date: DicomDate,
    #[serde(rename = "Search_AccessionNumber")]
    accession_number: AccessionNumber,
    #[serde(rename = "Anon_AccessionNumber")]
    anon_accession_number: String,
    #[serde(rename = "Anon_PatientBirthDate")]
    anon_patient_birth_date: DicomDate,
}

/// DICOM AccessionNumber
#[nutype::nutype(derive(Serialize, Deserialize, Clone, Hash, Eq, PartialEq))]
pub(crate) struct AccessionNumber(String);

/// - On `GET`: return list of BLT studies.
/// - On `POST`: query for the study by AccessionNumber in the first modality
///              Orthanc is configured with. If the study is found, then
///              add its details to an in-memory database and start a retrieval job.
pub(crate) fn route_http_request(
    context: *mut OrthancPluginContext,
    req: Request<BltStudy>,
    db: &mut HashMap<AccessionNumber, BltStudy>,
) -> Response<serde_json::Value> {
    match req.method {
        Method::Get => {
            let studies: Vec<_> = db.values().map(|s| s.clone()).collect();
            Response::ok(serde_json::to_value(studies).unwrap())
        }
        Method::Post => {
            if let Some(study) = req.body {
                query_and_retrieve(context, db, study)
            } else {
                Response::from(StatusCode::BAD_REQUEST)
            }
        }
        _ => Response::from(StatusCode::METHOD_NOT_ALLOWED),
    }
}

fn query_and_retrieve(
    context: *mut OrthancPluginContext,
    db: &mut HashMap<AccessionNumber, BltStudy>,
    study: BltStudy,
) -> Response<serde_json::Value> {
    let client = crate::orthanc::api::ModalitiesClient::new(context);
    let modality = if let Some(m) = client.list_modalities().into_iter().next() {
        m
    } else {
        return Response::error("Orthanc is not configured properly with modalities.".to_string());
    };

    // hard-coded request for testing purposes
    let studies = vec!["1.2.840.113845.11.1000000001785349915.20130308061609.6346698".to_string()];
    let res = client.c_move_studies(&modality, studies);

    db.insert(study.accession_number.clone(), study);

    res.map(|job| Response {
        code: StatusCode::CREATED,
        body: Some(serde_json::to_value(job).unwrap()),
    })
}
