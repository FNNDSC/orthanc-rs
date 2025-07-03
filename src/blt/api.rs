use super::database::BltDatabase;
use super::models::BltStudy;
use crate::orthanc::bindings::OrthancPluginContext;
use crate::orthanc::http::{Method, Request, Response};
use http::StatusCode;

/// - On `GET`: return list of BLT studies.
/// - On `POST`: query for the study by AccessionNumber in the first modality
///              Orthanc is configured with. If the study is found, then
///              add its details to an in-memory database and start a retrieval job.
pub fn route_http_request(
    context: *mut OrthancPluginContext,
    req: Request<BltStudy>,
    db: &mut BltDatabase,
) -> Response<serde_json::Value> {
    match req.method {
        Method::Get => Response::ok(serde_json::to_value(db.list_studies()).unwrap()),
        Method::Post => {
            if let Some(study) = req.body {
                query_and_retrieve(context, db, study).into()
            } else {
                Response::from(StatusCode::BAD_REQUEST)
            }
        }
        _ => Response::from(StatusCode::METHOD_NOT_ALLOWED),
    }
}

fn query_and_retrieve(
    context: *mut OrthancPluginContext,
    db: &mut BltDatabase,
    study: BltStudy,
) -> Result<Response<serde_json::Value>, Response<serde_json::Value>> {
    let client = crate::orthanc::api::ModalitiesClient::new(context);
    let modality = client.list_modalities().into_iter().next().ok_or_else(|| {
        Response::error("Orthanc is not configured properly with modalities.".to_string())
    })?;

    let accession_number = study.accession_number.clone().to_string();
    let query = client
        .query_study(modality, accession_number)
        .map_err(|e| {
            e.trace();
            Response::from(e)
        })?;
    let answers = query.answers().map_err(|e| {
        e.trace();
        Response::from(e)
    })?;
    if answers.is_empty() {
        return Ok(Response {
            code: StatusCode::NO_CONTENT,
            body: None,
        });
    }
    let job = query.request_retrieve_job().into_result().map_err(|e| {
        tracing::error!("i am here");
        e.trace();
        Response::from(e)
    })?;
    db.add_study(study, query.id.clone(), job.id.clone());
    Ok(Response {
        code: StatusCode::CREATED,
        body: Some(serde_json::json!({
            "QueryID": query.id,
            "JobID": job.id
        })),
    })
}
