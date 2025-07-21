# orthanc_api

> [!CAUTION]
> Work in progress.
> Please reach out via [GitHub](https://github.com/FNNDSC/orthanc-rs/discussions)
> or [Matrix](https://matrix.to/#/#chris-general:fedora.im) if you're using this
> crate! It would motivate me to complete the missing features.

Rust package defining types for Orthanc REST API requests and responses.

## Motivation

Orthanc has an [OpenAPI schema](https://orthanc.uclouvain.be/api/) from which
we can generate model code automatically, e.g. see
[orthanc_client_ogen](https://crates.io/crates/orthanc_client_ogen).
The type definitions here supplement the OpenAPI specification where response
models are untyped. Furthermore, the handwritten definitions are more precise
and idiomatic than what can be expressed using OpenAPI.

## Features

- Type-safe deserialization of complex [enums](https://serde.rs/enum-representations.html)
  not covered by Orthanc's OpenAPI specification, e.g.
  [`orthanc_api::JobInfo`](https://docs.rs/orthanc_api/latest/orthanc_api/struct.JobInfo.html).
- Use of [nutype](https://docs.rs/nutype/0.6.1/nutype/index.html)
- Related types are associated each other using traits: e.g. the trait implementation
  of [`DicomResourceId` for `SeriesId`](https://docs.rs/orthanc_api/latest/orthanc_api/trait.DicomResourceId.html#impl-DicomResourceId%3CT%3E-for-SeriesId)
  (1) associates it with [`Series`](https://docs.rs/orthanc_api/latest/orthanc_api/struct.Series.html),
  which contains detailed information about a DICOM series, and (2) denotes its ancestor type as
  [`StudyId`](https://docs.rs/orthanc_api/latest/orthanc_api/struct.StudyId.html)
  because a DICOM study is a set of DICOM series.
- This crate is [sans-IO](https://www.firezone.dev/blog/sans-io): it describes API types,
  but it does not implement communication with Orthanc. It can be built upon to provide
  an HTTP client for Orthanc (not implemented) or to be used in developing an Orthanc
  plugin that speaks to the built-in API (see [orthanc_sdk](https://crates.io/crates/orthanc_sdk)).
