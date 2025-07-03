//! Orthanc API response types.
//!
//! (These type definitions are handwritten with better ergonomics
//! than the automatically generated ones found in [crate::openapi]).

use kstring::KString;
use nutype::nutype;
use serde::{Deserialize, Serialize};

/// ID of an Orthanc query to a remote modality.
#[nutype(derive(Serialize, Deserialize, Clone, Display, Debug, Eq, PartialEq, Hash))]
pub struct QueryId(String);

/// ID of an Orthanc job.
#[nutype(derive(Serialize, Deserialize, Clone, Display, Debug, Eq, PartialEq, Hash))]
pub struct JobId(String);

/// Orthanc asynchronous job response data.
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Hash)]
#[serde(rename_all = "PascalCase")]
pub struct Job {
    #[serde(rename = "ID")]
    pub id: JobId,
    pub path: String,
}

/// Orthanc job detail response from
/// [`/jobs/{id}`](orthanc.uclouvain.be/api/#tag/Jobs/paths/~1jobs~1{id}/get)
///
/// Ref: <https://orthanc.uclouvain.be/hg/orthanc/file/tip/OrthancFramework/Sources/JobsEngine/JobInfo.cpp#l180>
#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
pub struct JobInfo {
    pub completion_time: String,
    #[serde(flatten)]
    pub content: JobContent,
    pub creation_time: String,
    pub effective_runtime: f64,
    pub error_code: i32,
    pub error_description: String,
    pub error_details: String,
    #[serde(rename = "ID")]
    pub id: String,
    pub priority: i32,
    pub progress: u8,
    pub state: JobState,
    pub timestamp: String,
}

/// Orthanc job state.
///
/// <https://orthanc.uclouvain.be/book/users/advanced-rest.html#monitoring-jobs>
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, Copy, Clone, Hash)]
pub enum JobState {
    /// The job is waiting to be executed.
    Pending,
    /// The job is being executed. The Progress field will be continuously
    /// updated to reflect the progression of the execution.
    Running,
    /// The job has finished with success.
    Success,
    /// The job has finished with failure. Check out the [JobInfo::error_code] and
    /// [JobInfo::error_description] fields for more information.
    Failure,
    /// The job has been paused.
    Paused,
    /// The job has failed internally, and has been scheduled for re-submission after a delay.
    /// As of Orthanc 1.12.8, this feature is not used by any type of job.
    Retry,
}

/// The content of an Orthanc job.
///
/// **WARNING**: Only [JobContent::DicomMoveScu] is implemented.
///
/// Job type ref: <https://orthanc.uclouvain.be/hg/orthanc/file/Orthanc-1.12.8/OrthancServer/Sources/ServerJobs/OrthancJobUnserializer.cpp#l66>
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "Type", content = "Content")]
pub enum JobContent {
    /// DICOM MOVE-SCU job.
    #[serde(rename_all = "PascalCase")]
    DicomMoveScu {
        description: KString,
        local_aet: KString,
        query: Vec<MoveScuJobQuery>,
        remote_aet: KString,
        target_aet: KString,
    },
    DicomModalityStore,
    OrthancPeerStore,
    ResourceModification,
    MergeStudy,
    SplitStudy,
    StorageCommitmentScp,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "0008,0052")]
pub enum MoveScuJobQuery {
    #[serde(rename = "SERIES")]
    Series {
        #[serde(rename = "0010,0020")]
        patient_id: KString,
        #[serde(rename = "0008,0050")]
        accession_number: KString,
        #[serde(rename = "0020,000d")]
        study_instance_uid: String,
        #[serde(rename = "0020,000e")]
        series_instance_uid: String,
    },
    #[serde(rename = "STUDY")]
    Study {
        #[serde(rename = "0010,0020")]
        patient_id: KString,
        #[serde(rename = "0008,0050")]
        accession_number: KString,
        #[serde(rename = "0020,000d")]
        study_instance_uid: String,
    },
    #[serde(rename = "PATIENT")]
    Patient {
        #[serde(rename = "0010,0020")]
        patient_id: KString,
    },
}

impl MoveScuJobQuery {
    /// Get the StudyInstanceUID.
    pub fn study_instance_uid(&self) -> Option<&str> {
        match self {
            MoveScuJobQuery::Series {
                study_instance_uid, ..
            } => Some(study_instance_uid),
            MoveScuJobQuery::Study {
                study_instance_uid, ..
            } => Some(study_instance_uid),
            MoveScuJobQuery::Patient { .. } => None,
        }
    }
}

/// Same data as [MoveScuJobQuery] but as a struct with [Option]
/// fields instead of being an `enum`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct MoveScuJobQueryAny {
    pub patient_id: KString,
    pub accession_number: Option<KString>,
    pub study_instance_uid: Option<String>,
    pub series_instance_uid: Option<String>,
}

impl From<MoveScuJobQuery> for MoveScuJobQueryAny {
    fn from(value: MoveScuJobQuery) -> Self {
        match value {
            MoveScuJobQuery::Series {
                patient_id,
                accession_number,
                study_instance_uid,
                series_instance_uid,
            } => Self {
                patient_id,
                accession_number: Some(accession_number),
                study_instance_uid: Some(study_instance_uid),
                series_instance_uid: Some(series_instance_uid),
            },
            MoveScuJobQuery::Study {
                patient_id,
                accession_number,
                study_instance_uid,
            } => Self {
                patient_id,
                accession_number: Some(accession_number),
                study_instance_uid: Some(study_instance_uid),
                series_instance_uid: None,
            },
            MoveScuJobQuery::Patient { patient_id } => Self {
                patient_id,
                accession_number: None,
                study_instance_uid: None,
                series_instance_uid: None,
            },
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum QueryRetrieveLevel {
    Study,
    Series,
    Image,
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;
    use serde_json::json;

    #[test]
    fn test_deserialize_job_retrieve_series() {
        let data = json!({
            "CompletionTime": "20250703T002648.691947",
            "Content": {
                "Description": "REST API",
                "LocalAet": "DEV",
                "Query": [
                    {
                        "0008,0050": "98edede8b2",
                        "0008,0052": "SERIES",
                        "0010,0020": "1449c1d",
                        "0020,000d": "1.2.840.113845.11.1000000001785349915.20130308061609.6346698",
                        "0020,000e": "1.3.12.2.1107.5.2.19.45152.2013030808061520200285270.0.0.0"
                    }
                ],
                "RemoteAet": "PACS",
                "TargetAet": "DEV"
            },
            "CreationTime": "20250703T002645.190848",
            "EffectiveRuntime": 3.5,
            "ErrorCode": 0,
            "ErrorDescription": "Success",
            "ErrorDetails": "",
            "ID": "0b09cfb2-d5c3-4340-9f96-0ae8812eadfe",
            "Priority": 0,
            "Progress": 100,
            "State": "Success",
            "Timestamp": "20250703T002655.833908",
            "Type": "DicomMoveScu"
        });
        let actual: JobInfo = serde_json::from_value(data).unwrap();
        let content = actual.content;
        let expected = JobContent::DicomMoveScu {
            description: KString::from_static("REST API"),
            local_aet: KString::from_static("DEV"),
            query: vec![MoveScuJobQuery::Series {
                patient_id: KString::from_static("1449c1d"),
                accession_number: KString::from_static("98edede8b2"),
                study_instance_uid: "1.2.840.113845.11.1000000001785349915.20130308061609.6346698"
                    .to_string(),
                series_instance_uid: "1.3.12.2.1107.5.2.19.45152.2013030808061520200285270.0.0.0"
                    .to_string(),
            }],
            remote_aet: KString::from_static("PACS"),
            target_aet: KString::from_static("DEV"),
        };
        assert_eq!(content, expected)
    }
}
