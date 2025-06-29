#![allow(unused)] // TODO remove me
use crate::dicom_date::DicomDate;
use crate::orthanc::http::{Request, Response};

#[derive(serde::Deserialize)]
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
    accession_number: String,
    #[serde(rename = "Anon_AccessionNumber")]
    anon_accession_number: String,
    #[serde(rename = "Anon_PatientBirthDate")]
    anon_patient_birth_date: DicomDate,
}

pub(crate) fn route_http_request(req: Request<BltStudy>) -> Response<()> {
    Response {
        code: http::StatusCode::CREATED,
        body: None,
    }
}
