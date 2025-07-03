use crate::api::types::job::JobInfo;
use crate::api::{RestResponse, client::BaseClient};
use crate::bindings::{OrthancPluginContext, OrthancPluginErrorCode};
use kstring::KString;
use nutype::nutype;
use serde::{Deserialize, Serialize};

/// ID of an Orthanc query to a remote modality.
#[nutype(derive(Serialize, Deserialize, Clone, Display, Debug, Eq, PartialEq, Hash))]
pub struct QueryId(String);

impl ResourceId<'_> for QueryId {
    /// `/queries/{id}` always responds with `["answers", "level", "modality", "query", "retrieve"]`
    type Item = Vec<KString>;

    fn uri(&self) -> String {
        format!("/queries/{}", &self)
    }
}

/// ID of an Orthanc job.
#[nutype(derive(Serialize, Deserialize, Clone, Display, Debug, Eq, PartialEq, Hash))]
pub struct JobId(String);

impl ResourceId<'_> for JobId {
    type Item = JobInfo;

    fn uri(&self) -> String {
        format!("/jobs/{}", &self)
    }
}

/// ID of a patient in Orthanc.
#[nutype(derive(Serialize, Deserialize, Clone, Display, Debug, Eq, PartialEq, Hash))]
pub struct PatientId(String);

impl ResourceId<'_> for PatientId {
    type Item = (); // TODO

    fn uri(&self) -> String {
        format!("/patients/{}", &self)
    }
}

/// ID of a DICOM study stored by Orthanc.
#[nutype(derive(Serialize, Deserialize, Clone, Display, Debug, Eq, PartialEq, Hash))]
pub struct StudyId(String);

impl ResourceId<'_> for StudyId {
    type Item = (); // TODO

    fn uri(&self) -> String {
        format!("/studies/{}", &self)
    }
}

/// ID of a DICOM series stored by Orthanc.
#[nutype(derive(Serialize, Deserialize, Clone, Display, Debug, Eq, PartialEq, Hash))]
pub struct SeriesId(String);

impl ResourceId<'_> for SeriesId {
    type Item = (); // TODO

    fn uri(&self) -> String {
        format!("/series/{}", &self)
    }
}

/// ID of a DICOM instance stored by Orthanc.
#[nutype(derive(Serialize, Deserialize, Clone, Display, Debug, Eq, PartialEq, Hash))]
pub struct InstanceId(String);

impl ResourceId<'_> for InstanceId {
    type Item = (); // TODO

    fn uri(&self) -> String {
        format!("/instances/{}", &self)
    }
}

/// ID of an Orthanc resource.
pub trait ResourceId<'a> {
    type Item: Deserialize<'a>;

    /// Get the API URI of this resource.
    fn uri(&self) -> String;

    /// Get this resource.
    fn get(&self, context: *mut OrthancPluginContext) -> RestResponse<Self::Item> {
        let client = BaseClient::new(context);
        let uri = self.uri();
        client.get(uri)
    }

    /// Delete this resource.
    fn delete(&self, context: *mut OrthancPluginContext) -> OrthancPluginErrorCode {
        let client = BaseClient::new(context);
        let uri = self.uri();
        client.delete(uri)
    }
}

pub trait DicomResourceId<'a>: ResourceId<'a> + Deserialize<'a> {
    type Ancestor: DicomResourceId<'a>;

    /// Delete this DICOM resource.
    fn delete(
        &self,
        context: *mut OrthancPluginContext,
    ) -> RestResponse<DeleteResponse<Self::Ancestor>> {
        let client = BaseClient::new(context);
        let uri = self.uri();
        client.delete_with_response(uri)
    }
}

/// Response from deleting a DICOM resource from Orthanc.
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct DeleteResponse<T> {
    pub remaining_ancestor: Option<IdAndPath<T>>,
}

/// ID and path of an Orthanc resource.
///
/// Note: the Orthanc response typically has `{ "Type": "Patient|Study|Series|Instance" }`,
/// which is missing from this struct because the information is conveyed by the generic type.
#[derive(Serialize, Deserialize, Debug, PartialEq, Hash)]
pub struct IdAndPath<T> {
    #[serde(rename = "ID")]
    pub id: T,
    #[serde(rename = "Path")]
    pub path: String,
}
