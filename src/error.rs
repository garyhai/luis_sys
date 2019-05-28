//! Common error definitions of the crate for SPX API errors and others.

use crate::speech::events::{CancellationError, NoMatchError, ToJson};
use failure::Fail;
use serde::Serialize;
use serde_json::Value;
use std::error::Error;

#[derive(Fail, Debug, Serialize)]
pub enum SpxError {
    #[fail(display = "speech API return error code: {}", _0)]
    ApiError(usize),
    #[fail(display = "ASR progress is cancelled: {}", _0)]
    Cancellation(Value),
    #[fail(display = "recognition result was not recognized: {}", _0)]
    NoMatch(Value),
    #[fail(display = "error occured with text: {}", _0)]
    Other(String),
    #[fail(display = "there is nothing")]
    IsNothing,
    #[fail(display = "an interior nul byte was found")]
    IsNull,
    #[fail(display = "an entity already exists")]
    AlreadyExists,
    #[fail(display = "mutex lock is poisoned")]
    Poisoned,
    #[fail(display = "operation may be blocked")]
    WouldBlock,
    #[fail(display = "method is unimplemented")]
    Unimplemented,
    #[fail(display = "unknown error")]
    Unknown,
}

impl<T: Error> From<T> for SpxError {
    fn from(err: T) -> Self {
        SpxError::Other(err.to_string())
    }
}

impl From<NoMatchError> for SpxError {
    fn from(err: NoMatchError) -> Self {
        match err.to_json() {
            Ok(v) => SpxError::NoMatch(v),
            Err(e) => e,
        }
    }
}

impl From<CancellationError> for SpxError {
    fn from(err: CancellationError) -> Self {
        match err.to_json() {
            Ok(v) => SpxError::Cancellation(v),
            Err(e) => e,
        }
    }
}

pub use SpxError::*;

pub fn from_hr(code: usize) -> Result<(), SpxError> {
    if code == 0 {
        Ok(())
    } else {
        Err(ApiError(code))
    }
}
