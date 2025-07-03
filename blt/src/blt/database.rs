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
}

#[derive(serde::Serialize)]
pub struct BltStudyState {
    info: BltStudy,
    query_id: QueryId,
    retrieve_job_id: JobId,
}

impl BltDatabase {
    /// Create an empty [BltDatabase] with at least the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            studies: HashMap::with_capacity(capacity),
            queries: BiMap::with_capacity(capacity),
            retrieve_jobs: BiMap::with_capacity(capacity),
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
}
