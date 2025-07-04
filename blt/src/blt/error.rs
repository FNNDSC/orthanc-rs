use orthanc_sdk::api::JsonResponseError;

/// A return type for functions which, upon encountering an [Err],
/// should trace the [Err] value and return.
pub(crate) type TraceAndReturn = Result<(), DoNothing>;

/// A unit struct which simply calls [JsonResponseError::trace] upon conversion.
pub(crate) struct DoNothing;

impl<T: std::fmt::Debug> From<JsonResponseError<T>> for DoNothing {
    fn from(value: JsonResponseError<T>) -> Self {
        value.trace();
        Self
    }
}
