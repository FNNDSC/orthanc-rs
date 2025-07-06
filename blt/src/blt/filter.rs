use crate::blt::error::DoNothing;
use orthanc_sdk::api::types::{RequestedTags, Series, SeriesId};
use orthanc_sdk::api::{DicomClient, GeneralClient};
use orthanc_sdk::bindings::OrthancPluginContext;

pub(crate) fn filter_received_series(
    context: *mut OrthancPluginContext,
    series: &[SeriesId],
) -> Result<usize, DoNothing> {
    let dicom_client = DicomClient::new(context);
    let client = GeneralClient::new(context);
    let mut deleted_count = 0;
    for series_id in series {
        let details: Series<SeriesDetails> = dicom_client.get(series_id).data()?;
        let tags = details.requested_tags;
        if let Some(reason) = tags.should_delete() {
            client.delete(details.id)?;
            deleted_count += 1;
            tracing::info!(
                SeriesInstanceUID = tags.series_instance_uid,
                SeriesDescription = tags.series_description,
                tag = reason.tag,
                value = reason.value,
                reason = reason.reason,
                "BLT series deleted"
            );
        }
    }
    Ok(deleted_count)
}

struct RequestedTagsForSeries;

impl From<RequestedTagsForSeries> for &'static [&'static str] {
    fn from(_: RequestedTagsForSeries) -> Self {
        &["Modality", "SeriesDescription", "SeriesInstanceUID"]
    }
}

#[derive(serde::Deserialize, Debug)]
struct SeriesDetails {
    // #[serde(rename = "SOPClassUID")]
    // sop_class_uid: String,
    #[serde(rename = "SeriesDescription")]
    series_description: String,
    #[serde(rename = "SeriesInstanceUID")]
    series_instance_uid: String,
    #[serde(rename = "Modality")]
    modality: compact_str::CompactString,
}

impl RequestedTags for SeriesDetails {
    fn names() -> &'static [&'static str] {
        &["Modality", "SeriesDescription", "SeriesInstanceUID"]
    }
}

impl SeriesDetails {
    fn should_delete(&self) -> Option<DeleteReason<'_>> {
        // TODO Sandip excludes series with: {"SOPClassUID": "Secondary Capture Image Storage"}
        if self.modality == "US" {
            Some(DeleteReason {
                reason: "ultrasound images should not be uploaded to BLT",
                tag: "Modality",
                value: self.modality.as_str(),
            })
        } else {
            None
        }
    }
}

struct DeleteReason<'a> {
    reason: &'static str,
    tag: &'static str,
    value: &'a str,
}
