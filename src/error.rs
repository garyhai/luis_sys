use crate::asr::events::{CancellationError, NoMatchError, ToJson};
use failure::Fail;
use serde_json::{Value, Error as JsonError};
use std::ffi;

#[derive(Fail, Debug)]
pub enum SpxError {
    #[fail(display = "speech API return error code: {}", _0)]
    ApiError(usize),
    #[fail(display = "ASR progress is cancelled: {}", _0)]
    Cancellation(Value),
    #[fail(display = "recognition result was not recognized: {}", _0)]
    NoMatch(Value),
    #[fail(display = "failed to parse as JSON format: {}", _0)]
    ParseJson(JsonError),
    #[fail(display = "an interior nul byte was found")]
    NulError,
    #[fail(display = "invalid UTF-8 string")]
    Utf8Error,
    #[fail(display = "an entity already exists")]
    AlreadyExists,
    #[fail(display = "mutex lock is poisoned")]
    Poisoned,
    #[fail(display = "operation may be blocked")]
    WouldBlock,
    #[fail(display = "method is unimplemented")]
    Unimplemented,
    #[fail(display = "unknown error: {}", _0)]
    Unknown(String),
}

pub use SpxError::*;

impl From<usize> for SpxError {
    fn from(code: usize) -> Self {
        assert!(code != 0);
        SpxError::ApiError(code)
    }
}

impl From<JsonError> for SpxError {
    fn from(err: JsonError) -> Self {
        SpxError::ParseJson(err)
    }
}

impl From<NoMatchError> for SpxError {
    fn from(err: NoMatchError) -> Self {
        match err.to_json() {
            Ok(v) => NoMatch(v),
            Err(e) => e,
        }
    }
}

impl From<CancellationError> for SpxError {
    fn from(err: CancellationError) -> Self {
        match err.to_json() {
            Ok(v) => Cancellation(v),
            Err(e) => e,
        }
    }
}

impl From<ffi::NulError> for SpxError {
    fn from(_err: ffi::NulError) -> Self {
        SpxError::NulError
    }
}

impl From<ffi::IntoStringError> for SpxError {
    fn from(_err: ffi::IntoStringError) -> Self {
        SpxError::Utf8Error
    }
}

impl From<std::str::Utf8Error> for SpxError {
    fn from(_err: std::str::Utf8Error) -> Self {
        SpxError::Utf8Error
    }
}

impl<T> From<std::sync::TryLockError<T>> for SpxError {
    fn from(err: std::sync::TryLockError<T>) -> Self {
        match err {
            std::sync::TryLockError::WouldBlock => SpxError::WouldBlock,
            _ => SpxError::Poisoned,
        }
    }
}

impl<T> From<std::sync::PoisonError<T>> for SpxError {
    fn from(_: std::sync::PoisonError<T>) -> Self {
        SpxError::Poisoned
    }
}
