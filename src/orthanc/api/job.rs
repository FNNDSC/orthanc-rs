/// Orthanc asynchronous job response data.
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Hash)]
pub struct Job {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "Path")]
    pub path: String,
}
