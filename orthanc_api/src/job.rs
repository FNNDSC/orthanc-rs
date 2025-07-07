use compact_str::CompactString;
use serde::{Deserialize, Serialize};

use crate::{JobId, PatientId, SeriesId, StudyId};

/// Orthanc job detail response from
/// [`/jobs/{id}`](orthanc.uclouvain.be/api/#tag/Jobs/paths/~1jobs~1{id}/get)
///
/// Ref: <https://orthanc.uclouvain.be/hg/orthanc/file/tip/OrthancFramework/Sources/JobsEngine/JobInfo.cpp#l180>
#[derive(Serialize, Deserialize, Debug)]
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
    pub id: JobId,
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
/// **WARNING**: Only [JobContent::DicomMoveScu] and [JobContent::ResourceModification] are implemented.
///
/// Job type ref: <https://orthanc.uclouvain.be/hg/orthanc/file/Orthanc-1.12.8/OrthancServer/Sources/ServerJobs/OrthancJobUnserializer.cpp#l66>
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "Type", content = "Content")]
pub enum JobContent {
    /// DICOM MOVE-SCU job.
    #[serde(rename_all = "PascalCase")]
    DicomMoveScu {
        description: CompactString,
        local_aet: CompactString,
        query: Vec<MoveScuJobQuery>,
        remote_aet: CompactString,
        target_aet: CompactString,
    },
    DicomModalityStore {},
    OrthancPeerStore {},
    ResourceModification(ResourceModificationContent),
    MergeStudy {},
    SplitStudy {},
    StorageCommitmentScp {},
}

/// Generic resource modification job content.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct ResourceModification<I> {
    pub description: CompactString,
    pub failed_instances_count: usize,
    #[serde(rename = "ID")]
    pub id: I,
    pub instances_count: usize,
    pub is_anonymization: bool,
    pub parent_resources: Vec<I>,
    pub path: String,
    #[serde(rename = "PatientID")]
    pub patient_id: PatientId,
}

/// Resource modification job content enum.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "Type")]
pub enum ResourceModificationContent {
    Patient(ResourceModification<PatientId>),
    Study(ResourceModification<StudyId>),
    Series(ResourceModification<SeriesId>),
}

/// The query of a MOVE-SCU job.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "0008,0052")]
pub enum MoveScuJobQuery {
    #[serde(rename = "SERIES")]
    Series {
        #[serde(rename = "0010,0020")]
        patient_id: CompactString,
        #[serde(rename = "0008,0050")]
        accession_number: CompactString,
        #[serde(rename = "0020,000d")]
        study_instance_uid: String,
        #[serde(rename = "0020,000e")]
        series_instance_uid: String,
    },
    #[serde(rename = "STUDY")]
    Study {
        #[serde(rename = "0010,0020")]
        patient_id: CompactString,
        #[serde(rename = "0008,0050")]
        accession_number: CompactString,
        #[serde(rename = "0020,000d")]
        study_instance_uid: String,
    },
    #[serde(rename = "PATIENT")]
    Patient {
        #[serde(rename = "0010,0020")]
        patient_id: CompactString,
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
    pub patient_id: CompactString,
    pub accession_number: Option<CompactString>,
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

/// DICOM level of a query or retrieve operation.
#[derive(Serialize, Deserialize, Debug, Copy, Clone, Eq, PartialEq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum QueryRetrieveLevel {
    Study,
    Series,
    Image,
}
