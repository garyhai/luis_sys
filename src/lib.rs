//! FFI bindings for Microsoft LUIS API.
//!
//! Current version support only LUIS Speech API.

#[macro_use]
extern crate bitflags;
use std::{ffi::CStr, os::raw::c_char};

pub(crate) mod macros;
pub(crate) mod speech_api;

pub mod error;
pub mod properties;
pub mod speech;

/// Common error type of the crate.
pub use error::SpxError;
pub use speech::*;
/// Redefine the result of the crate for convenience.
pub type Result<T = (), E = SpxError> = std::result::Result<T, E>;

pub use speech_api::{SPXHANDLE, SPXHR};

/// (-1) as INVALID HANDLE for initilization or validation.
pub(crate) const INVALID_HANDLE: SPXHANDLE = std::usize::MAX as SPXHANDLE;

/// (0:usize) as NULL HANDLE for initilization.
pub(crate) const NULL_HANDLE: SPXHANDLE = 0 as SPXHANDLE;

/// Trait for underlying handle of the API.
pub trait Handle<T = SPXHANDLE> {
    fn handle(&self) -> T;
}

/// Retrieve the string from FFI function with pre-allocated buffer.
pub(crate) fn get_cf_string(
    cf: unsafe extern "C" fn(SPXHANDLE, *mut c_char, u32) -> SPXHR,
    handle: SPXHANDLE,
    length: usize,
) -> Result<String> {
    let length = if length == 0 { 1024 } else { length };
    let max_len = length + 1;
    let mut s = Vec::with_capacity(max_len);
    let buf_ptr = s.as_mut_ptr() as *mut std::os::raw::c_char;
    hr!(cf(handle, buf_ptr, max_len as u32))?;
    let output = unsafe { CStr::from_ptr(buf_ptr) };
    Ok(output.to_str().map(String::from)?)
}
