//! Orthanc API response types.
//!
//! (These type definitions are handwritten with better ergonomics
//! than the automatically generated ones found in [crate::openapi]).

mod id;
mod job;

pub use id::*;
pub use job::*;
