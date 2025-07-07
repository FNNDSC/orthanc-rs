# orthanc_sdk

Idiomatic and somewhat safe abstractions for developing an Orthanc plugin in Rust.

## Naming Conventions

Orthanc can do many operations asynchronously using a built-in job queue.
This is requested by including `{ "Asynchronous": true }` in the POST body.
Doing so is recommended.

API client methods have the following name conventions:

- `*_request`, e.g. `DicomClient::anonymize_request`, are low-level methods for sending an arbitrary request (either synchronous or asynchronous) to the API endpoint with a generic return type.
- The name without a suffix, e.g. `DicomCLient::anonymize`, are high-level methods for making the request with `{ "Asynchronous": true }` in the POST body. The return type will always be `PostJsonResponse<IdAndPath<JobId>>`.
  - (The name is simpler to imply that it's what you are recommended to use.)
