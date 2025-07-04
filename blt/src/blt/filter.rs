use crate::blt::error::TraceAndReturn;
use orthanc_sdk::api::types::{DicomResourceId, RequestedTags, SeriesId};
use orthanc_sdk::bindings::OrthancPluginContext;

pub(crate) fn filter_received_series(
    context: *mut OrthancPluginContext,
    series: &[SeriesId],
) -> TraceAndReturn {
    // TODO Sandip excludes series with: {"SOPClassUID": "Secondary Capture Image Storage"}
    for series_id in series {
        let tags: SeriesDetails = series_id.get(context).data()?.requested_tags;
        tracing::info!(
            series = series_id.to_string(),
            Modality = tags.modality.as_str(),
            SOPClassUID = tags.sop_class_uid
        );
    }
    Ok(())
}

struct RequestedTagsForSeries;

impl From<RequestedTagsForSeries> for &'static [&'static str] {
    fn from(_: RequestedTagsForSeries) -> Self {
        &["Modality", "SOPClassUID"]
    }
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct SeriesDetails {
    #[serde(rename = "SOPClassUID")]
    sop_class_uid: String,
    modality: kstring::KString,
}

impl RequestedTags for SeriesDetails {
    fn names() -> &'static [&'static str] {
        &["Modality", "SOPClassUID"]
    }
}
