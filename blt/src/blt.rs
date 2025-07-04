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
mod error;
mod filter;
mod models;
mod on_change;
mod series_of_study;

pub use api::route_http_request;
pub use database::BltDatabase;
pub use on_change::on_change;
