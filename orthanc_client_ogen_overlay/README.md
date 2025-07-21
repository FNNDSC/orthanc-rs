# Orthanc API Rust Client by OpenAPI Generator

This crate is a client library for the [Orthanc API](https://orthanc.uclouvain.be/book/users/rest.html)
automatically generated using [OpenAPI Generator](https://openapi-generator.tech/).
With that said, its quality is limited by that of the input OpenAPI specification and `openapi-generator-cli`.

`orthanc_client_ogen` contains a client based on [reqwest](https://crates.io/crates/reqwest)
and API request/response model type definitions. It can be used for its models definitions
only by disabling default features:

```toml
[dependencies]
orthanc_client_ogen = { version = "1", default-features = false }
```

> [!TIP]
> See [orthanc_api](https://crates.io/crates/orthanc_api), which defines models for what is not covered by Orthanc's OpenAPI schema.

## Notes

`openapi-generator-cli` is chosen over alternatives such as [paperclip](https://github.com/paperclip-rs/paperclip)
  or [progenitor](https://github.com/oxidecomputer/progenitor) because `openapi-generator-cli` is tolerant of
  schema invalidity. See https://discourse.orthanc-server.org/t/openapi-specification-validity/6044
