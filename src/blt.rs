#![allow(unused)]

use std::collections::HashMap;
use http::StatusCode;
use crate::dicom_date::DicomDate;
use crate::orthanc::http::{Method, Request, Response};

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
pub(crate) fn route_http_request(req: Request<BltStudy>, blt_studies: &mut HashMap<AccessionNumber, BltStudy>) -> Response<Vec<BltStudy>> {
    match req.method {
        Method::Get => {
            Response::ok(blt_studies.values().map(|s| s.clone()).collect())
        }
        Method::Post => {
            if let Some(study) = req.body {
                blt_studies.insert(study.accession_number.clone(), study);
                Response::from(StatusCode::CREATED)
            } else {
                Response::from(StatusCode::BAD_REQUEST)
            }
        }
        _ => {
            Response::from(StatusCode::METHOD_NOT_ALLOWED)
        }
    }
}
