use super::client::BaseClient;
use super::response::JsonResponseError;
use compact_str::CompactString;
use std::collections::HashMap;

/// A client for the Orthanc API [`/queries/{id}/answers`](https://orthanc.uclouvain.be/api/#tag/Networking/paths/~1queries~1{id}~1answers/get).
///
/// Use the [IntoIterator] trait to get all answer contents.
/// (Currently, no method is provided for getting one answer content.)
pub struct Answers {
    ids: Vec<CompactString>,
    path: String,
    client: BaseClient,
}

impl Answers {
    pub fn new(client: BaseClient, path: String, ids: Vec<CompactString>) -> Self {
        Self { client, path, ids }
    }

    /// Get the answer IDs.
    pub fn ids(&self) -> &[CompactString] {
        &self.ids
    }

    /// Returns `true` if there are no answers to the query.
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    /// Returns the number of answers.
    pub fn len(&self) -> usize {
        self.ids.len()
    }
}

/// The response type from
/// [`/queries/{id}/answers/{index}/content`](https://orthanc.uclouvain.be/api/#tag/Networking/paths/~1queries~1{id}~1answers~1{index}~1content/get).
pub type AnswerContent = HashMap<CompactString, AnswerTag>;

/// DICOM tag data
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Hash, Clone)]
pub struct AnswerTag {
    /// Name of DICOM tag
    name: CompactString,
    /// Data type of value
    type_: CompactString,
    /// DICOM value
    value: String,
}

impl IntoIterator for Answers {
    type Item = Result<AnswerContent, JsonResponseError<AnswerContent>>;
    type IntoIter = AnswersIter;

    fn into_iter(mut self) -> Self::IntoIter {
        self.ids.reverse();
        AnswersIter {
            ids: self.ids,
            path: self.path,
            client: self.client,
        }
    }
}

/// An iterator over query answers.
pub struct AnswersIter {
    ids: Vec<CompactString>,
    path: String,
    client: BaseClient,
}

impl Iterator for AnswersIter {
    type Item = Result<AnswerContent, JsonResponseError<AnswerContent>>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(id) = self.ids.pop() {
            let url = format!("{}/{id}", &self.path);
            Some(self.client.get(url).data())
        } else {
            None
        }
    }
}
