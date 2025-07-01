use super::models::{AccessionNumber, BltStudy};
use std::collections::HashMap;

// TODO use ValKey instead of an in-process HashMap, for persistence and scalability

/// In-process and in-memory database of BLT studies being processed by this Orthanc plugin.
#[derive(Default)]
pub struct BltDatabase {
    studies: HashMap<AccessionNumber, BltStudy>,
    queries: HashMap<String, AccessionNumber>,
}

impl BltDatabase {
    /// Create an empty [BltDatabase] with at least the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            studies: HashMap::with_capacity(capacity),
            queries: HashMap::with_capacity(capacity),
        }
    }

    pub fn list_studies(&self) -> Vec<BltStudy> {
        self.studies.values().map(|s| s.clone()).collect()
    }

    pub fn add_study(&mut self, study: BltStudy, query_id: String) {
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
        self.queries.insert(query_id, accession_number);
    }
}
