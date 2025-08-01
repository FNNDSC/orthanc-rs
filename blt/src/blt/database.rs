use super::models::{AccessionNumber, BltStudy};
use bimap::BiMap;
use orthanc_sdk::api::types::{JobId, QueryId};
use std::collections::HashMap;

// TODO use ValKey instead of an in-process HashMap, for persistence and scalability

/// In-process and in-memory database of BLT studies being processed by this Orthanc plugin.
#[derive(Default)]
pub struct BltDatabase {
    studies: HashMap<AccessionNumber, BltStudy>,
    queries: BiMap<QueryId, AccessionNumber>,
    retrieve_jobs: BiMap<JobId, AccessionNumber>,
    anonymize_jobs: BiMap<JobId, AccessionNumber>,
    push_jobs: BiMap<JobId, AccessionNumber>,
}

#[derive(serde::Serialize)]
pub struct BltStudyState {
    #[serde(rename = "Info")]
    info: BltStudy,
    #[serde(rename = "QueryID")]
    query_id: QueryId,
    #[serde(rename = "RetrieveJobID")]
    retrieve_job_id: JobId,
    #[serde(rename = "AnonymizationJobID")]
    anonymization_job_id: Option<JobId>,
    #[serde(rename = "PushJobID")]
    push_job_id: Option<JobId>,
}

impl BltDatabase {
    /// Create an empty [BltDatabase] with at least the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            studies: HashMap::with_capacity(capacity),
            queries: BiMap::with_capacity(capacity),
            retrieve_jobs: BiMap::with_capacity(capacity),
            anonymize_jobs: BiMap::with_capacity(capacity),
            push_jobs: BiMap::with_capacity(capacity),
        }
    }

    pub fn list_studies(&self) -> Vec<BltStudyState> {
        self.studies
            .values()
            .map(|s| BltStudyState {
                info: s.clone(),
                query_id: self
                    .queries
                    .get_by_right(&s.accession_number)
                    .map(|id| id.clone())
                    .unwrap(),
                retrieve_job_id: self
                    .retrieve_jobs
                    .get_by_right(&s.accession_number)
                    .map(|id| id.clone())
                    .unwrap(),
                anonymization_job_id: self
                    .anonymize_jobs
                    .get_by_right(&s.accession_number)
                    .map(|id| id.clone()),
                push_job_id: self
                    .push_jobs
                    .get_by_right(&s.accession_number)
                    .map(|id| id.clone()),
            })
            .collect()
    }

    pub fn add_study(&mut self, study: BltStudy, query_id: QueryId, job_id: JobId) {
        let accession_number = study.accession_number.clone();
        if self
            .studies
            .insert(accession_number.clone(), study)
            .is_some()
        {
            tracing::warn!(
                AccessionNumber = accession_number.as_str(),
                "BLT study requested twice"
            );
        }
        self.queries.insert(query_id, accession_number.clone());
        self.retrieve_jobs.insert(job_id, accession_number);
    }

    /// Returns `true` if the specified job ID is a BLT PACS retrieve job.
    pub fn has_retrieve(&self, id: &JobId) -> bool {
        self.retrieve_jobs.contains_left(id)
    }

    /// Get the original [AccessionNumber] of a BLT anonymization job.
    pub fn get_accession_number_of_anonymization(&self, id: &JobId) -> Option<AccessionNumber> {
        self.anonymize_jobs.get_by_left(id).cloned()
    }

    /// Get the BLT study request for an AccessionNumber.
    pub fn get(&self, accession_number: &AccessionNumber) -> Option<&BltStudy> {
        self.studies.get(accession_number)
    }

    /// Add an anonymization job.
    pub fn add_anonymization(&mut self, job_id: JobId, accession_number: AccessionNumber) {
        assert!(self.retrieve_jobs.contains_right(&accession_number));
        self.anonymize_jobs.insert(job_id, accession_number);
    }

    /// Add a push job.
    pub fn add_push(&mut self, job_id: JobId, accession_number: AccessionNumber) {
        assert!(self.anonymize_jobs.contains_right(&accession_number));
        self.push_jobs.insert(job_id, accession_number);
    }
}
