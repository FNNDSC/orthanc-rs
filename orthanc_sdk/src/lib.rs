//! Idiomatic and hopefully safe abstractions for developing an Orthanc plugin in Rust.
//!
//! The documentation here aims to be concise. Please refer to the
//! [basic example plugin code](https://github.com/FNNDSC/orthanc-rs/blob/master/examples/basic/src/plugin.rs)
//! as a gentler introduction on how to use this crate.

#![cfg_attr(docsrs, feature(doc_auto_cfg))]

pub mod bindings;
mod error_code;
pub use orthanc_client_ogen::models as openapi;

pub mod api;
pub mod http;
pub mod utils;

mod config;
mod rest;
mod sdk;
mod tracing_subscriber;

pub use config::{OrthancConfigurationBuffer, get_configuration};
pub use rest::*;
pub use sdk::*;
pub use tracing_subscriber::OrthancLogger;

#[cfg(feature = "webapp")]
pub mod webapp;
#[cfg(feature = "webapp")]
pub use webapp::serve_static_file;
