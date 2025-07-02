use nutype::nutype;

/// ID of an Orthanc query to a remote modality.
#[nutype(derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash))]
pub struct QueryId(String);

/// ID of an Orthanc job.
#[nutype(derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Hash))]
pub struct JobId(String);

/// Orthanc asynchronous job response data.
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Hash)]
pub struct Job {
    #[serde(rename = "ID")]
    pub id: JobId,
    #[serde(rename = "Path")]
    pub path: String,
}
