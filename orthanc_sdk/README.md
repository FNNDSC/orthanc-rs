# orthanc_sdk

> [!CAUTION]
> Work in progress.
> Please reach out via [GitHub](https://github.com/FNNDSC/orthanc-rs/discussions)
> or [Matrix](https://matrix.to/#/#chris-general:fedora.im) if you're using this
> crate! It would motivate me to complete the missing features.

Idiomatic and hopefully safe abstractions for developing an Orthanc plugin in Rust.

## Getting Started

Please refer to the [example plugin](../example_plugin/src/plugin.rs).

## Supported Features

```rust
#[non_exhaustive]
```

- [x] Rust bindings for `OrthancCPlugin.h`: [`orthanc_sdk::bindings`](https://docs.rs/orthanc_sdk/0.2.0/orthanc_sdk/bindings/index.html)
- [x] Read the Orthanc configuration JSON: [`orthanc_sdk::get_configuration`](https://docs.rs/orthanc_sdk/0.2.0/orthanc_sdk/fn.get_configuration.html)
- [x] [`tracing::Subscriber`](https://docs.rs/tracing-core/0.1.34/tracing_core/subscriber/trait.Subscriber.html)implementation for Orthanc's built-in logging: [`orthanc_sdk::OrthancLogger`](https://docs.rs/orthanc_sdk/0.2.0/orthanc_sdk/struct.OrthancLogger.html)
- [x] Helper for handling on change events in a background thread: [`orthanc_sdk::utils::OnChangeThread`](https://docs.rs/orthanc_sdk/0.2.0/orthanc_sdk/utils/struct.OnChangeThread.html)
- [x] Register REST callbacks: [`orthanc_sdk::register_rest`](https://docs.rs/orthanc_sdk/0.2.0/orthanc_sdk/fn.register_rest.html) and [`orthanc_sdk::register_rest_no_lock`](https://docs.rs/orthanc_sdk/0.2.0/orthanc_sdk/fn.register_rest_no_lock.html)
- [x] Call the built-in Orthanc API: [`DicomClient`](https://docs.rs/orthanc_sdk/0.2.0/orthanc_sdk/api/struct.DicomClient.html) and [`GeneralClient`](https://docs.rs/orthanc_sdk/0.2.0/orthanc_sdk/api/struct.GeneralClient.html)
- [x] Easily package a static web application as an Orthanc plugin: [`orthanc_sdk::serve_static_file`](https://docs.rs/orthanc_sdk/0.2.0/orthanc_sdk/fn.serve_static_file.html).
  - Real example: https://github.com/FNNDSC/orthanc-patient-list/
- [ ] Support HTTP headers when calling the Orthanc built-in API
- [ ] Support specific HTTP responses using [`OrthancPluginSendMethodNotAllowed`](https://orthanc.uclouvain.be/sdk/group__REST.html#ga1a060d2b2aba0172eb68ebb69d26722c), [`OrthancPluginRedirect`](https://orthanc.uclouvain.be/sdk/group__REST.html#ga92aebd39a92e2bdbdb1b1dc5f60cadd5), [`OrthancPluginSendUnauthorized`](https://orthanc.uclouvain.be/sdk/group__REST.html#ga0c09ccbddb26011ba30eeddf94819d52)
- [ ] Support HTTP multi-part answer using [`OrthancPluginStartMultipartAnswer`](https://orthanc.uclouvain.be/sdk/group__REST.html#gadfae0b05c5890fe07fd4762ac58dfed4)
- [ ] Support HTTP stream answer using [`OrthancPluginStartStreamAnswer`](https://orthanc.uclouvain.be/sdk/group__REST.html#ga8cd840aae20e180ca8af0aa3a85f9c9e)
- [ ] Call Orthanc peer using [`OrthancPluginCallPeerApi`](https://orthanc.uclouvain.be/sdk/group__Toolbox.html#gadd62594f47cedbb473449be0eb53504c)
- [ ] Make arbitrary HTTP calls using [`OrthancPluginHttpClient`](https://orthanc.uclouvain.be/sdk/group__Toolbox.html#ga053d2c35e6c39b5f6c8fda400c1672d3)
- [ ] Callback for received DICOM instances
- [ ] Custom storage are
- [ ] Custom database back-end area
- [ ] Handler for C-Find SCP
- [ ] Handler for C-Find SCP against DICOM worklists
- [ ] Handler for C-Move SCP
- [ ] Custom decoder for DICOM images
- [ ] Callback to filter incoming HTTP requests
- [ ] Callback to underialize jobs
- [ ] Callback to refresh its metrics
- [ ] Callback to answer chunked HTTP transfers
- [ ] Callback for Storage Commitment SCP
- [ ] Callback to keep/discard/modify incoming DICOM instances
- [ ] Custom transcoder for DICOM images
- [ ] Callback to discard instances received
- [ ] Callback to branch a WebDAV virtual filesystem
- [ ] Macro to generate safe `extern "C" fn` definitions
- [ ] Safe wrappers for [images and compression](https://orthanc.uclouvain.be/sdk/group__Images.html)
- [ ] Safe wrappers for [DicomInstance](https://orthanc.uclouvain.be/sdk/group__DicomInstance.html)
- [ ] Safe wrapper for [DicomCallbacks](https://orthanc.uclouvain.be/sdk/group__DicomCallbacks.html)

## Naming Conventions

Orthanc can do many operations asynchronously using a built-in job queue.
This is requested by including `{ "Asynchronous": true }` in the POST body.
Doing so is recommended.

API client methods have the following name conventions:

- `*_request`, e.g. `DicomClient::anonymize_request`, are low-level methods for sending an arbitrary request (either synchronous or asynchronous) to the API endpoint with a generic return type.
- The name without a suffix, e.g. `DicomCLient::anonymize`, are high-level methods for making the request with `{ "Asynchronous": true }` in the POST body. The return type will always be `PostJsonResponse<IdAndPath<JobId>>`.
  - (The name is simpler to imply that it's what you are recommended to use.)

## Acknowledgements

Thanks to Andrew Webber's [Orthanc Rust Samples](https://github.com/andrewwebber/orthanc-rust-plugins)
for helping me get started with FFI and Orthanc's SDK.
