use orthanc_sdk::api::DicomClient;
use orthanc_sdk::api::types::{RequestedTags, Study, StudyId};
use orthanc_sdk::bindings::OrthancPluginContext;

use super::error::DoNothing;
use super::models::AccessionNumber;

pub(crate) fn get_accession_number(
    context: *mut OrthancPluginContext,
    study_id: StudyId,
) -> Result<AccessionNumber, DoNothing> {
    let client = DicomClient::new(context);
    let study: Study<Details> = client.get(study_id).ok_data()?;
    Ok(study.requested_tags.accession_number)
}

#[derive(serde::Deserialize, Debug, Clone, PartialEq)]
struct Details {
    #[serde(rename = "AccessionNumber")]
    accession_number: AccessionNumber,
}

impl RequestedTags for Details {
    fn names() -> &'static [&'static str] {
        &["AccessionNumber"]
    }
}
