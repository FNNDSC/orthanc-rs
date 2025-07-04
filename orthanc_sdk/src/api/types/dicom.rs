use crate::api::types::{InstanceId, PatientId, SeriesId, StudyId};
use serde::{Deserialize, Serialize};

/// Orthanc patient detail response from
/// [`/patients/{id}`](https://orthanc.uclouvain.be/api/#tag/Patients/paths/~1patients~1{id}/get)
/// (incomplete)
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Patient<T> {
    #[serde(rename = "ID")]
    pub id: PatientId,
    pub requested_tags: T,
}

/// Orthanc study detail response from
/// [`/studies/{id}`](https://orthanc.uclouvain.be/api/#tag/Studies/paths/~1studies~1{id}/get)
/// (incomplete)
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Study<T> {
    #[serde(rename = "ID")]
    pub id: StudyId,
    pub requested_tags: T,
}

/// Orthanc series detail response from
/// [`/series/{id}`](https://orthanc.uclouvain.be/api/#tag/Series/paths/~1series~1{id}/get).
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Series<T> {
    pub expected_number_of_instances: Option<usize>,

    #[serde(rename = "ID")]
    pub id: SeriesId,

    pub instances: Vec<InstanceId>,

    pub is_stable: bool,
    pub labels: Vec<String>,
    pub last_update: String,
    pub parent_study: StudyId,
    pub status: SeriesStatus,

    /// Main DICOM tags.
    ///
    /// Ref: <https://orthanc.uclouvain.be/book/faq/main-dicom-tags.html>
    ///
    /// Note: the schema of "MainDicomTags" is customizable by the
    /// Orthanc configuration, so it cannot have a static type.
    pub main_dicom_tags: serde_json::Value,
    pub requested_tags: T,
}

/// Series completion status.
///
/// Ref: <https://orthanc.uclouvain.be/book/faq/series-completion.html>
#[derive(Serialize, Deserialize, Debug)]
pub enum SeriesStatus {
    Unknown,
    Missing,
    Inconsistent,
    Complete,
}

/// Orthanc instance details response from
/// [`/instances/{id}`](https://orthanc.uclouvain.be/api/#tag/Instances/paths/~1instances~1{id}/get)
/// (incomplete)
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct Instance<T> {
    #[serde(rename = "ID")]
    pub id: InstanceId,
    pub requested_tags: T,
}

/// Marker trait for DICOM resources stored in Orthanc.
pub trait DicomResource<T> {}
impl<T> DicomResource<T> for Patient<T> {}
impl<T> DicomResource<T> for Study<T> {}
impl<T> DicomResource<T> for Series<T> {}
impl<T> DicomResource<T> for Instance<T> {}
