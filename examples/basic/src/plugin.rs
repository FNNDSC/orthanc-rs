//! Example Orthanc plugin in Rust.
//!
//! This plugin has two features:
//!
//! - REST API `/add` which performs integer addition.
//! - Write MRNs to a file whenever a new patient is added.
//!
//! **IMPORTANT**: When developing a Rust plugin for Orthanc,
//! make sure you have the following configuration in your
//! `Cargo.toml` file:
//!
//! ```toml
//! [lib]
//! crate-type = ["cdylib"]
//! ```

#![allow(non_snake_case)]

use std::io::Write;
use std::sync::RwLock;

use orthanc_sdk::api::types::{Patient, PatientId};
use orthanc_sdk::bindings;

/// The [bindings::OrthancPluginContext] is a magical object which must
/// be passed as a parameter to every function of the Orthanc plugin
/// interface. [OrthancContext] is a custom wrapper for it.
struct OrthancContext(*mut bindings::OrthancPluginContext);

// We must indicate that `OrthancContext` is thread-safe.
unsafe impl Send for OrthancContext {}
unsafe impl Sync for OrthancContext {}

/// The instance of [bindings::OrthancPluginContext] is given to us
/// when Orthanc calls [OrthancPluginInitialize]. We must store the
/// instance in a global variable and pass it around.
///
/// [RwLock] is a good data structure for here, it makes it easy to
/// have a global variable that is written to once, then read many
/// times (with possibly concurrent reads).
static GLOBAL_STATE: RwLock<AppState> = RwLock::new(AppState {
    context: None,
    config: None,
    on_change_thread: None,
});

/// Global application state.
#[derive(Default)]
struct AppState {
    context: Option<OrthancContext>,
    config: Option<ExamplePluginConfig>,
    on_change_thread: Option<orthanc_sdk::utils::OnChangeThread>,
}

/// A macro which produces the plugin name as a string literal.
///
/// NOTE: we use a macro instead of `const` so that it can be used with [concat].
macro_rules! plugin_name {
    () => {
        "example_rust_plugin"
    };
}

#[unsafe(no_mangle)]
pub extern "C" fn OrthancPluginGetName() -> *const u8 {
    // C strings must be nul-terminated!
    concat!(plugin_name!(), "\0").as_ptr()
}

#[unsafe(no_mangle)]
pub extern "C" fn OrthancPluginGetVersion() -> *const u8 {
    // C strings must be nul-terminated!
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr()
}

/// Entire Orthanc configuration JSON file content.
#[derive(serde::Deserialize)]
struct OrthancConfiguration {
    #[serde(rename = "ExampleRustPlugin")]
    example_rust_plugin: Option<ExamplePluginConfig>,
}

/// Configuration of just this plugin.
#[derive(serde::Deserialize, Default)]
struct ExamplePluginConfig {
    #[serde(rename = "MrnFile")]
    output_file: Option<std::path::PathBuf>,
}

/// [OrthancPluginInitialize] is called once at Orthanc startup.
/// It initializes the plugin by:
///
/// 1. Providing the [bindings::OrthancPluginContext] instance
/// 2. Registering any plugin callbacks
/// 3. Arbitrary initialization tasks, e.g. spawning background threads
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[unsafe(no_mangle)]
pub extern "C" fn OrthancPluginInitialize(
    context: *mut bindings::OrthancPluginContext,
) -> bindings::OrthancPluginErrorCode {
    // Create a logger which reports log messages using Orthanc Toolbox's built-in
    // logging system.
    let logger = orthanc_sdk::OrthancLogger {
        context,
        plugin_name: plugin_name!(),
        verbose: true,
    };
    // Set the global `tracing` subscriber. `tracing` is the de-facto crate for Rust logging.
    if let Err(e) = tracing::subscriber::set_global_default(logger)
    // caveat: OrthancPluginInitialize might be called twice or more (if `/tools/reset`
    // is called on the REST API), so we should ignore the error when this happens.
        && !e.to_string().contains("has already been set")
    {
        eprintln!("Failed to initialize logging in Rust plugin: {e}");
    }

    let mut app_state = GLOBAL_STATE.try_write().unwrap();
    // Store the [bindings::OrthancPluginContext] instance in the global variable
    app_state.context = Some(OrthancContext(context));

    // Read the Orthanc configuration
    let configuration: OrthancConfiguration = orthanc_sdk::get_configuration(context)
        .unwrap()
        .deserialize()
        .unwrap();
    // Store the plugin configuration section in the global state
    app_state.config = Some(configuration.example_rust_plugin.unwrap_or_default());

    // Spawn a background thread to handle OnChange events.
    app_state.on_change_thread = Some(orthanc_sdk::utils::OnChangeThread::spawn(move |event| {
        let app_state = GLOBAL_STATE.try_read().unwrap();
        let context = app_state.context.as_ref().unwrap().0;
        let config = app_state.config.as_ref().unwrap();
        let _ = on_change_handler(context, config, event);
    }));

    // register plugin callback functions
    orthanc_sdk::register_on_change(context, Some(on_change_callback));
    orthanc_sdk::register_rest_no_lock(context, c"/rustexample/add", Some(rest_callback));

    // return a successful error code.
    bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_Success
}

/// Called when Orthanc is shutting down. The plugin must release all allocated resources.
#[unsafe(no_mangle)]
pub extern "C" fn OrthancPluginFinalize() {
    let mut app_state = GLOBAL_STATE.try_write().expect("unable to obtain lock");
    if let Some(thread) = app_state.on_change_thread.take() {
        thread.join().unwrap()
    }
    app_state.context = None;
    app_state.config = None;
}

/// The OnChange callback function invoked directly by Orthanc.
///
/// It does plumbing before sending the event to the background thread's channel.
extern "C" fn on_change_callback(
    change_type: bindings::OrthancPluginChangeType,
    resource_type: bindings::OrthancPluginResourceType,
    resource_id: *const std::ffi::c_char,
) -> bindings::OrthancPluginErrorCode {
    // read resource_id as a Rust-friendly type
    let resource_id = if resource_id.is_null() {
        None
    } else if let Ok(cstr) = unsafe { std::ffi::CStr::from_ptr(resource_id).to_str() } {
        Some(cstr.to_string())
    } else {
        tracing::warn!("resource_id is not UTF-8");
        return bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_InternalError;
    };
    // Get the global app state
    if let Ok(app) = GLOBAL_STATE.try_read()
        && let Some(channel) = &app.on_change_thread
    {
        let event = orthanc_sdk::utils::OnChangeEvent {
            change_type,
            resource_type,
            resource_id,
        };
        if channel.send(event).is_ok() {
            bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_Success
        } else {
            bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_InternalError
        }
    } else {
        bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_InternalError
    }
}

/// The function which handles on_change events.
///
/// This function is called from in a background thread, so it is OK to do
/// blocking work (e.g. calling the Orthanc built-in API or other I/O).
///
/// In this example, we will call [mrn_append_to_file] every time a new
/// patient is added.
fn on_change_handler(
    context: *mut bindings::OrthancPluginContext,
    config: &ExamplePluginConfig,
    orthanc_sdk::utils::OnChangeEvent {
        change_type,
        resource_type,
        resource_id,
    }: orthanc_sdk::utils::OnChangeEvent,
) {
    if change_type == bindings::OrthancPluginChangeType_OrthancPluginChangeType_NewPatient
        && resource_type == bindings::OrthancPluginResourceType_OrthancPluginResourceType_Patient
        && let Some(patient_id) = resource_id
        && let Some(output_file) = &config.output_file
    {
        let patient_id = PatientId::new(patient_id);
        let _ = mrn_append_to_file(context, patient_id, output_file);
    }
}

/// Append the patient's MRN to a text file.
fn mrn_append_to_file(
    context: *mut bindings::OrthancPluginContext,
    patient_id: PatientId,
    output_file: &std::path::Path,
) {
    // Get the Patient MRN by calling the built-in Orthanc API
    let client = orthanc_sdk::api::DicomClient::new(context);
    let patient: Patient<PatientDetails> = client.get(patient_id).unwrap();
    let mrn = patient.requested_tags.mrn;
    let mut file = std::fs::OpenOptions::new()
        .append(true)
        .open(output_file)
        .unwrap();
    file.write_all(mrn.as_bytes()).unwrap();
    file.write_all("\n".as_bytes()).unwrap();
}

/// The patient-level DICOM tags we care about getting back from the Orthanc API.
#[derive(serde::Deserialize, Debug)]
struct PatientDetails {
    #[serde(rename = "PatientID")]
    mrn: String,
}

impl orthanc_sdk::api::types::RequestedTags for PatientDetails {
    fn names() -> &'static [&'static str] {
        &["PatientID"] // field names of `PatientDetails`
    }
}

/// The HTTP handler callback function invoked directly by Orthanc.
///
/// It does plumbing before calling [http_route_add].
extern "C" fn rest_callback(
    output: *mut bindings::OrthancPluginRestOutput,
    url: *const std::ffi::c_char,
    request: *const bindings::OrthancPluginHttpRequest,
) -> bindings::OrthancPluginErrorCode {
    let app_state = if let Ok(app_state) = GLOBAL_STATE.try_read() {
        app_state
    } else {
        tracing::error!("Failed to read GLOBAL_STATE");
        return bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_InternalError;
    };
    let context = app_state.context.as_ref().unwrap().0;
    // `create_json_rest_callback` is a helper function which adapts FFI
    // to Rust-friendly data types.
    orthanc_sdk::create_json_rest_callback(context, output, url, request, |req| http_route_add(req))
}

/// HTTP route which adds two integers.
fn http_route_add(
    req: orthanc_sdk::http::Request<ExampleHttpBody>,
) -> orthanc_sdk::http::Response<ExampleResponseBody> {
    tracing::info!(method = req.method.as_str(), uri = req.url);
    if req.method == orthanc_sdk::http::Method::Post {
        if let Some(body) = req.body {
            let sum = body.a + body.b;
            orthanc_sdk::http::Response::ok(ExampleResponseBody { sum })
        } else {
            orthanc_sdk::http::Response {
                code: http::StatusCode::BAD_REQUEST,
                body: None,
            }
        }
    } else {
        orthanc_sdk::http::Response {
            // TODO: OrthancPluginSendMethodNotAllowed is not yet implemented in orthanc_sdk
            // code: http::StatusCode::METHOD_NOT_ALLOWED,
            code: http::StatusCode::BAD_REQUEST,
            body: None,
        }
    }
}

/// Example HTTP POST body.
#[derive(serde::Deserialize)]
struct ExampleHttpBody {
    a: u32,
    b: u32,
}

#[derive(serde::Serialize)]
struct ExampleResponseBody {
    sum: u32,
}
