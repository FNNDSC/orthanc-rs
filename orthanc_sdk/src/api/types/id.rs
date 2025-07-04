use super::{dicom::*, job::JobInfo};
use crate::api::{RestResponse, client::BaseClient};
use crate::bindings::{OrthancPluginContext, OrthancPluginErrorCode};
use kstring::KString;
use nutype::nutype;
use serde::{Deserialize, Serialize};

/// ID of an Orthanc query to a remote modality.
#[nutype(derive(Serialize, Deserialize, Clone, Display, Debug, Eq, PartialEq, Hash))]
pub struct QueryId(String);

impl ResourceId for QueryId {
    fn uri(&self) -> String {
        format!("/queries/{}", &self)
    }
}

impl SystemResourceId<'_> for QueryId {
    // note: `/queries/{id}` always responds with `["answers", "level", "modality", "query", "retrieve"]`
    type Item = Vec<KString>;
}

/// ID of an Orthanc job.
#[nutype(derive(Serialize, Deserialize, Clone, Display, Debug, Eq, PartialEq, Hash))]
pub struct JobId(String);

impl ResourceId for JobId {
    fn uri(&self) -> String {
        format!("/jobs/{}", &self)
    }
}

impl SystemResourceId<'_> for JobId {
    type Item = JobInfo;
}

/// ID of a patient in Orthanc.
#[nutype(derive(Serialize, Deserialize, Clone, Display, Debug, Eq, PartialEq, Hash))]
pub struct PatientId(String);

impl ResourceId for PatientId {
    fn uri(&self) -> String {
        format!("/patients/{}", &self)
    }
}

impl<'a, T> DicomResourceId<'_, T> for PatientId {
    type Item = Patient<T>;
    type Ancestor = PatientId;
}

/// ID of a DICOM study stored by Orthanc.
#[nutype(derive(Serialize, Deserialize, Clone, Display, Debug, Eq, PartialEq, Hash))]
pub struct StudyId(String);

impl ResourceId for StudyId {
    fn uri(&self) -> String {
        format!("/studies/{}", &self)
    }
}

impl<'a, T> DicomResourceId<'_, T> for StudyId {
    type Item = Study<T>;
    type Ancestor = PatientId;
}

/// ID of a DICOM series stored by Orthanc.
#[nutype(derive(Serialize, Deserialize, Clone, Display, Debug, Eq, PartialEq, Hash))]
pub struct SeriesId(String);

impl ResourceId for SeriesId {
    fn uri(&self) -> String {
        format!("/series/{}", &self)
    }
}

impl<'a, T> DicomResourceId<'a, T> for SeriesId {
    type Item = Series<T>;
    type Ancestor = StudyId;
}

/// ID of a DICOM instance stored by Orthanc.
#[nutype(derive(Serialize, Deserialize, Clone, Display, Debug, Eq, PartialEq, Hash))]
pub struct InstanceId(String);

impl ResourceId for InstanceId {
    fn uri(&self) -> String {
        format!("/instances/{}", &self)
    }
}

impl<'a, T> DicomResourceId<'a, T> for InstanceId {
    type Item = Instance<T>;
    type Ancestor = SeriesId;
}

/// ID of an Orthanc resource.
pub trait ResourceId {
    /// Get the API URI of this resource.
    fn uri(&self) -> String;
}

/// ID of an Orthanc system (i.e. not DICOM data) resource, e.g. job, query.
pub trait SystemResourceId<'a>: ResourceId {
    type Item: Deserialize<'a>;

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

/// ID of an Orthanc DICOM resource, e.g. patient, study, series, instance.
pub trait DicomResourceId<'a, T>: ResourceId + Deserialize<'a> {
    type Item: DicomResource<T>;
    type Ancestor: DicomResourceId<'a, T>;

    /// Get this resource.
    fn get(&self, context: *mut OrthancPluginContext) -> RestResponse<Self::Item>
    where
        Self::Item: Deserialize<'a>,
        T: RequestedTags,
    {
        let client = BaseClient::new(context);
        let requested_tags = T::names().join(";");
        let uri = format!("{}?requested-tags={}", self.uri(), requested_tags);
        client.get(uri)
    }

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

/// A type for the "RequestedDicomTags" field in Orthanc's JSON response to
/// getting a DICOM patient, study, series, or instance.
pub trait RequestedTags {
    /// DICOM tag names of this type.
    fn names() -> &'static [&'static str];
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
