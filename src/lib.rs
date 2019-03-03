#[macro_use]
extern crate bitflags;
use std::{ffi::CStr, os::raw::c_char};

pub(crate) mod audio;
pub(crate) mod macros;
pub(crate) mod properties;
pub(crate) mod speech_api;

pub mod asr;
pub mod error;

pub use asr::*;
pub use error::SpxError;
pub type Result<T = (), E = SpxError> = std::result::Result<T, E>;

pub use speech_api::{SPXHANDLE, SPXHR};
pub(crate) const INVALID_HANDLE: SPXHANDLE = std::usize::MAX as SPXHANDLE;

pub trait Handle<T = SPXHANDLE> {
    fn handle(&self) -> T;
}

pub(crate) fn ffi_result(code: SPXHR) -> Result {
    if code == 0 {
        Ok(())
    } else {
        Err(SpxError::from(code))
    }
}

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
    Ok(String::from(output.to_str()?))
}
