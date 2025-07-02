use super::client::BaseClient;
use crate::orthanc::api::response::JsonResponseError;
use kstring::KString;
use std::collections::HashMap;

/// A client for the Orthanc API [`/queries/{id}/answers`](https://orthanc.uclouvain.be/api/#tag/Networking/paths/~1queries~1{id}~1answers/get).
///
/// Use the [IntoIterator] trait to get all answer contents.
/// (Currently, no method is provided for getting one answer content.)
pub struct Answers {
    ids: Vec<KString>,
    path: String,
    client: BaseClient,
}

impl Answers {
    pub(super) fn new(client: BaseClient, path: String, ids: Vec<KString>) -> Self {
        Self { client, path, ids }
    }

    /// Get the answer IDs.
    pub fn ids(&self) -> &[KString] {
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
pub type AnswerContent = HashMap<KString, AnswerTag>;

/// DICOM tag data
#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq, Hash, Clone)]
pub struct AnswerTag {
    /// Name of DICOM tag
    name: KString,
    /// Data type of value
    type_: KString,
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

pub struct AnswersIter {
    ids: Vec<KString>,
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
