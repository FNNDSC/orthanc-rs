#![allow(unused)]

use crate::dicom_date::DicomDate;
use crate::orthanc::bindings::OrthancPluginContext;
use crate::orthanc::http::{Method, Request, Response};
use crate::orthanc::models::ModalitiesIdMovePostRequest;
use http::StatusCode;
use std::collections::HashMap;

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
/// - On `POST`: append to in-memory database of BLT studies.
pub(crate) fn route_http_request(
    context: *mut OrthancPluginContext,
    req: Request<BltStudy>,
    blt_studies: &mut HashMap<AccessionNumber, BltStudy>,
) -> Response<serde_json::Value> {
    match req.method {
        Method::Get => {
            let studies: Vec<_> = blt_studies.values().map(|s| s.clone()).collect();
            Response::ok(serde_json::to_value(studies).unwrap())
        }
        Method::Post => {
            if let Some(study) = req.body {
                let client = crate::orthanc::api::ModalitiesClient::new(context);

                // hard-coded request for testing purposes
                let studies = vec![
                    "1.2.840.113845.11.1000000001785349915.20130308061609.6346698".to_string(),
                ];
                let res = client.c_move_studies("PACS", studies);

                blt_studies.insert(study.accession_number.clone(), study);

                res.map(|job| Response {
                    code: StatusCode::CREATED,
                    // FIXME should have response body, but does not
                    body: Some(serde_json::to_value(dbg!(job)).unwrap()),
                })
            } else {
                Response::from(StatusCode::BAD_REQUEST)
            }
        }
        _ => Response::from(StatusCode::METHOD_NOT_ALLOWED),
    }
}
