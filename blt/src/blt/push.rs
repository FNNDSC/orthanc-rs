use orthanc_sdk::api::{PeersClient, types::ResourceId};
use orthanc_sdk::bindings::OrthancPluginContext;
use orthanc_sdk::openapi::PeersIdStorePostRequest as StoreRequest;

use crate::blt::error::DoNothing;

use super::BltDatabase;
use super::error::TraceAndReturn;

pub(crate) fn push_to_peer<I: ResourceId + std::string::ToString>(
    context: *mut OrthancPluginContext,
    _db: &mut BltDatabase,
    resource_id: I,
) -> TraceAndReturn {
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
    // TODO store job.id in database
    Ok(())
}
