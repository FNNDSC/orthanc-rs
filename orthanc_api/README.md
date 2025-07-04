# orthanc_api

Rust package defining types for Orthanc REST API requests and responses.

- These handwritten type definitions are more precise and idiomatic than what can be
automatically generated using [OpenAPI Generator](https://openapi-generator.tech/).
- Use of [nutype](https://docs.rs/nutype/0.6.1/nutype/index.html)
- Related types are associated each other using traits: e.g. the trait implementation
  of `DicomResourceId` for `SeriesId` (1) associates it with `Series`, which contains 
  detailed information about a DICOM series, and (2) denotes its ancestor type as
  `StudyId` because a DICOM study is a set of DICOM series.
- This is a [sans-IO](https://www.firezone.dev/blog/sans-io) crate: it describes API
  types, but it does not implement communication with Orthanc. It can be built upon
  to provide an HTTP client for Orthanc (not implemented) or to be used in developing
  an Orthanc plugin that speaks to the built-in API (see [orthanc_sdk](../orthanc_sdk)).