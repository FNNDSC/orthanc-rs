use orthanc_sdk::api::{
    PeersClient,
    types::{JobId, ResourceId},
};
use orthanc_sdk::bindings::OrthancPluginContext;
use orthanc_sdk::openapi::PeersIdStorePostRequest as StoreRequest;

use crate::blt::error::DoNothing;

pub(crate) fn push_to_peer<I: ResourceId + std::string::ToString>(
    context: *mut OrthancPluginContext,
    resource_id: I,
) -> Result<JobId, DoNothing> {
    let client = PeersClient::new(context);
    let peer = if let Some(peer) = client.list().into_iter().next() {
        peer
    } else {
        tracing::error!("Orthanc is not configured with any peers");
        return Err(DoNothing);
    };
    let request = StoreRequest {
        asynchronous: Some(true),
        compress: Some(true), // TODO let this be configurable
        resources: Some(vec![resource_id.to_string()]),
        ..Default::default()
    };
    let job = client.store(&peer, request).into_result()?;
    tracing::info!(
        job = job.id.to_string(),
        resource = resource_id.to_string(),
        peer = &peer,
        "will push to peer"
    );
    Ok(job.id)
}
