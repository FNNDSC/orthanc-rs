//! BLT Protocol:
//!
//! User submits [BltStudy] to this plugin. The protocol is to:
//!
//! 1. Query for study in PACS by AccessionNumber
//! 2. Retrieve study from PACS
//! 3. Anonymize study DICOM
//! 4. Push to an Orthanc peer

mod api;
mod database;
mod dicom_date;
mod models;

pub use api::route_http_request;
pub use database::BltDatabase;
