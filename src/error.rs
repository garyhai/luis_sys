//! Common error definitions of the crate for SPX API errors and others.

use crate::speech::events::{CancellationError, NoMatchError, ToJson};
use failure::Fail;
use serde::Serialize;
use serde_json::{Error as JsonError, Value};
use std::{
    ffi, io,
    string::FromUtf8Error,
    sync::{self, mpsc},
};

#[derive(Fail, Debug, Serialize)]
pub enum SpxError {
    #[fail(display = "{}", _0)]
    Misc(String),
    #[fail(display = "speech API return error code: {}", _0)]
    ApiError(usize),
    #[fail(display = "I/O error: {}", _0)]
    IoError(String),
    #[fail(display = "ASR progress is cancelled: {}", _0)]
    Cancellation(Value),
    #[fail(display = "recognition result was not recognized: {}", _0)]
    NoMatch(Value),
    #[fail(display = "failed to parse as JSON format: {}", _0)]
    ParseJson(String),
    #[fail(display = "an interior nul byte was found")]
    NulError,
    #[fail(display = "invalid UTF-8 string")]
    Utf8Error(String),
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

impl SpxError {
    pub fn misc_err<T: ToString>(err: T) -> SpxError {
        SpxError::Misc(err.to_string())
    }
}

pub use SpxError::*;

impl From<usize> for SpxError {
    fn from(code: usize) -> Self {
        assert!(code != 0);
        SpxError::ApiError(code)
    }
}

impl From<io::Error> for SpxError {
    fn from(err: io::Error) -> Self {
        SpxError::IoError(err.to_string())
    }
}

impl From<mpsc::RecvError> for SpxError {
    fn from(err: mpsc::RecvError) -> Self {
        SpxError::IoError(err.to_string())
    }
}

impl<T> From<mpsc::SendError<T>> for SpxError {
    fn from(err: mpsc::SendError<T>) -> Self {
        SpxError::IoError(err.to_string())
    }
}

impl From<JsonError> for SpxError {
    fn from(err: JsonError) -> Self {
        SpxError::ParseJson(err.to_string())
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
    fn from(err: ffi::IntoStringError) -> Self {
        SpxError::Utf8Error(err.to_string())
    }
}

impl From<std::str::Utf8Error> for SpxError {
    fn from(err: std::str::Utf8Error) -> Self {
        SpxError::Utf8Error(err.to_string())
    }
}

impl From<FromUtf8Error> for SpxError {
    fn from(err: FromUtf8Error) -> Self {
        SpxError::Utf8Error(err.to_string())
    }
}

impl<T> From<sync::TryLockError<T>> for SpxError {
    fn from(err: sync::TryLockError<T>) -> Self {
        match err {
            sync::TryLockError::WouldBlock => SpxError::WouldBlock,
            _ => SpxError::Poisoned,
        }
    }
}

impl<T> From<sync::PoisonError<T>> for SpxError {
    fn from(_: sync::PoisonError<T>) -> Self {
        SpxError::Poisoned
    }
}
