use crate::blt::error::TraceAndReturn;
use orthanc_sdk::api::DicomClient;
use orthanc_sdk::api::types::{RequestedTags, Series, SeriesId};
use orthanc_sdk::bindings::OrthancPluginContext;

pub(crate) fn filter_received_series(
    context: *mut OrthancPluginContext,
    series: &[SeriesId],
) -> TraceAndReturn {
    let client = DicomClient::new(context);
    for series_id in series {
        let details: Series<SeriesDetails> = client.get(series_id).data()?;
        let tags = details.requested_tags;
        tracing::info!(
            series = series_id.to_string(),
            Modality = tags.modality.as_str(),
            SOPClassUID = tags.sop_class_uid,
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
struct SeriesDetails {
    #[serde(rename = "SOPClassUID")]
    sop_class_uid: String,
    #[serde(rename = "Modality")]
    modality: compact_str::CompactString,
}

impl RequestedTags for SeriesDetails {
    fn names() -> &'static [&'static str] {
        &["Modality", "SOPClassUID"]
    }
}

impl SeriesDetails {
    fn should_delete(&self) -> bool {
        // TODO Sandip excludes series with: {"SOPClassUID": "Secondary Capture Image Storage"}
        self.modality != "MR"
    }
}