//! Traits for idiomatic error handling with [`OrthancPluginErrorCode`](OrthancPluginErrorCode).

use crate::bindings::OrthancPluginErrorCode;
use crate::bindings::OrthancPluginErrorCode_OrthancPluginErrorCode_Success as Success;

/// A [Result] wrapper for [`OrthancPluginErrorCode`](OrthancPluginErrorCode).
///
/// By convention, the [Ok] value _should_ be [`OrthancPluginErrorCode_Success`](Success)
/// and the [Err] type should be all other values.
///
/// It can be unwrapped as an [`OrthancPluginErrorCode`](OrthancPluginErrorCode)
/// using the [Into] trait.
pub type ErrorCodeResult = Result<OrthancPluginErrorCode, OrthancPluginErrorCode>;

/// Provides the [`into_result`](ErrorCode::into_result) method for idiomatic error handling.
pub trait ErrorCode {
    /// Produce [Ok] if the code is [`OrthancPluginErrorCode_Success`](Success),
    /// or [Err] for any other value.
    fn into_result(self) -> ErrorCodeResult;

    /// If this is a success, return the given `code` instead.
    fn map_ok(self, code: OrthancPluginErrorCode) -> OrthancPluginErrorCode;
}

impl ErrorCode for OrthancPluginErrorCode {
    fn into_result(self) -> ErrorCodeResult {
        if self == Success { Ok(self) } else { Err(self) }
    }

    fn map_ok(self, code: OrthancPluginErrorCode) -> OrthancPluginErrorCode {
        if self == Success { code } else { self }
    }
}

/// Provides the [`into_code`](IntoOrthancPluginErrorCode::into_code) method.
pub trait IntoOrthancPluginErrorCode {
    /// Convert to [`OrthancPluginErrorCode`](OrthancPluginErrorCode).
    fn into_code(self) -> OrthancPluginErrorCode;
}

impl IntoOrthancPluginErrorCode for ErrorCodeResult {
    fn into_code(self) -> OrthancPluginErrorCode {
        match self {
            Ok(code) => code,
            Err(code) => code,
        }
    }
}
