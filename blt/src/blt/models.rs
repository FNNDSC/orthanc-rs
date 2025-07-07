use super::dicom_date::DicomDate;

/// Request for a study under the BLT protocol.

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub(crate) struct BltStudy {
    #[serde(rename = "MRN")]
    pub patient_id: String,
    #[serde(rename = "Anon_PatientID")]
    pub anon_patient_id: String,
    #[serde(rename = "PatientName")]
    pub patient_name: String,
    #[serde(rename = "Anon_PatientName")]
    pub anon_patient_name: String,
    #[serde(rename = "PatientBirthDate")]
    pub patient_birth_date: DicomDate,
    #[serde(rename = "Search_AccessionNumber")]
    pub accession_number: AccessionNumber,
    #[serde(rename = "Anon_AccessionNumber")]
    pub anon_accession_number: String,
    #[serde(rename = "Anon_PatientBirthDate")]
    pub anon_patient_birth_date: DicomDate,
}

/// DICOM AccessionNumber
#[nutype::nutype(
    derive(Serialize, Deserialize, Clone, Hash, Eq, PartialEq, AsRef, Deref, Debug, Display),
    validate(predicate = |s| !s.is_empty())
)]
pub struct AccessionNumber(compact_str::CompactString);
