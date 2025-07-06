use orthanc_sdk::api::{DeleteError, JsonResponseError, ResponseError, ResponseErrorCode};

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

impl From<ResponseErrorCode> for DoNothing {
    fn from(value: ResponseErrorCode) -> Self {
        tracing::error!("{}", value.to_string());
        Self
    }
}

impl<T: std::fmt::Debug> From<ResponseError<T>> for DoNothing {
    fn from(value: ResponseError<T>) -> Self {
        match value {
            ResponseError::Code(e) => e.into(),
            ResponseError::Json(e) => e.into(),
        }
    }
}

impl<I: std::fmt::Debug> From<DeleteError<I>> for DoNothing {
    fn from(value: DeleteError<I>) -> Self {
        tracing::error!(
            resource = format!("{:?}", value.id),
            code = value.code,
            "unsuccessful delete"
        );
        Self
    }
}
