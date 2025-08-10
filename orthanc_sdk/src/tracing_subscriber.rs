use std::ffi::CString;

use super::sdk::invoke_service;
use crate::bindings;
use tracing::{Level, span};

/// A [tracing::Subscriber] which calls
/// [OrthancPluginLogMessage](https://orthanc.uclouvain.be/sdk/group__Toolbox.html#ga0d9d440dc5622861d357920c12662c95).
///
/// **Note**: only events are supported, spans are not. Hint: just use [tracing::debug].
/// [tracing::info], [tracing::warn], and [tracing::error].
///
/// ### Panics
///
/// Messages must be convertible to [CString] i.e. must not contain nul bytes.
pub struct OrthancLogger {
    /// Orthanc plugin context
    pub context: *mut bindings::OrthancPluginContext,
    /// Plugin name
    pub plugin_name: &'static str,
    /// Force [Level::INFO] to be interpreted as [bindings::OrthancPluginLogLevel_OrthancPluginLogLevel_Warning]
    pub verbose: bool,
}

unsafe impl Send for OrthancLogger {}
unsafe impl Sync for OrthancLogger {}

impl tracing::Subscriber for OrthancLogger {
    fn enabled(&self, _: &tracing::Metadata<'_>) -> bool {
        true
    }

    fn new_span(&self, _: &span::Attributes<'_>) -> span::Id {
        span::Id::from_u64(0)
    }

    fn record(&self, _: &span::Id, _: &span::Record<'_>) {}

    fn record_follows_from(&self, _: &span::Id, _: &span::Id) {}

    fn event(&self, event: &tracing::Event<'_>) {
        let message = serialize_message(event);
        let c_message = CString::new(message).unwrap();
        let c_plugin = CString::new(self.plugin_name).unwrap();
        let c_file = CString::new(event.metadata().file().unwrap_or("unknown")).unwrap();
        let params = bindings::_OrthancPluginLogMessage {
            message: c_message.as_ptr(),
            plugin: c_plugin.as_ptr(),
            file: c_file.as_ptr(),
            line: event.metadata().line().unwrap_or(0),
            category: bindings::OrthancPluginLogCategory_OrthancPluginLogCategory_Generic,
            level: to_orthanc_level(event.metadata().level(), self.verbose),
        };
        let code = invoke_service(
            self.context,
            bindings::_OrthancPluginService__OrthancPluginService_LogMessage,
            params,
        );
        if code != bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_Success {
            eprintln!("ERROR: OrthancPluginLogMessage (code {code})");
        }
    }

    fn enter(&self, _: &span::Id) {}

    fn exit(&self, _: &span::Id) {}
}

#[allow(clippy::if_same_then_else)]
fn to_orthanc_level(level: &Level, verbose: bool) -> bindings::OrthancPluginLogLevel {
    if level <= &Level::ERROR {
        bindings::OrthancPluginLogLevel_OrthancPluginLogLevel_Error
    } else if level <= &Level::WARN {
        bindings::OrthancPluginLogLevel_OrthancPluginLogLevel_Warning
    } else if level >= &Level::INFO && verbose {
        bindings::OrthancPluginLogLevel_OrthancPluginLogLevel_Warning
    } else if level <= &Level::INFO {
        bindings::OrthancPluginLogLevel_OrthancPluginLogLevel_Info
    } else if level <= &Level::DEBUG {
        bindings::OrthancPluginLogLevel_OrthancPluginLogLevel_Trace
    } else if level <= &Level::TRACE {
        bindings::OrthancPluginLogLevel_OrthancPluginLogLevel_Trace
    } else {
        bindings::OrthancPluginLogLevel_OrthancPluginLogLevel_Trace
    }
}

fn serialize_message(event: &tracing::Event<'_>) -> CString {
    let mut visitor = FieldVisitor::new();
    event.record(&mut visitor);
    visitor.try_into().unwrap()
}

#[derive(Default)]
struct FieldVisitor {
    data: Vec<String>,
    message: Option<String>,
}

impl FieldVisitor {
    pub fn new() -> Self {
        Default::default()
    }
}

impl TryFrom<FieldVisitor> for CString {
    type Error = std::ffi::NulError;
    fn try_from(value: FieldVisitor) -> Result<CString, Self::Error> {
        let data = value.data.join(" ");
        let joined = if let Some(message) = value.message {
            format!("{data} {message}")
        } else {
            data
        };
        CString::new(joined)
    }
}

impl tracing::field::Visit for FieldVisitor {
    fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
        if field.name() == "message" {
            self.message.replace(format!("{value:?}"));
        } else {
            let s = format!("{}={value:?}", field.name());
            self.data.push(s);
        }
    }

    fn record_str(&mut self, field: &tracing::field::Field, value: &str) {
        let s = format!("{}=\"{}\"", field.name(), value.replace('"', "\\\""));
        self.data.push(s);
    }
}
