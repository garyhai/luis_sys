use std::ffi;
use failure::Fail;
use crate::asr::results::{NoMatchResult, CancellationResult};

#[derive(Fail, Debug)]
pub enum SpxError {
    #[fail(display = "speech API return error code: {}", _0)]
    ApiError(usize),
    #[fail(display = "ASR progress is cancelled: {}", _0)]
    Cancellation(String),
    #[fail(display = "recognition result was not recognized: {}", _0)]
    NoMatch(String),
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
    #[fail(display = "unknown error")]
    Unknown,
}

impl From<usize> for SpxError {
    fn from(code: usize) -> Self {
        assert!(code != 0);
        SpxError::ApiError(code)
    }
}

impl From<NoMatchResult> for SpxError {
    fn from(err: NoMatchResult) -> Self {
        SpxError::NoMatch(err.to_string())
    }
}

impl From<CancellationResult> for SpxError {
    fn from(err: CancellationResult) -> Self {
        SpxError::Cancellation(err.to_string())
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
            WouldBlock => SpxError::WouldBlock,
            _ => SpxError::Poisoned,
        }
    }
}

impl<T> From<std::sync::PoisonError<T>> for SpxError {
    fn from(err: std::sync::PoisonError<T>) -> Self {
        SpxError::Poisoned
    }
}