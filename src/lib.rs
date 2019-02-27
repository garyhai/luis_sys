#[macro_use]
extern crate bitflags;
use std::{ffi::CString, os::raw::c_char};

pub(crate) mod audio;
pub(crate) mod macros;
pub(crate) mod properities;
pub(crate) mod speech_api;

pub mod asr;
pub mod error;

pub use asr::*;
pub use error::SpxError;
pub type Result<T = (), E = SpxError> = std::result::Result<T, E>;

pub use speech_api::{SPXHANDLE, SPXHR};
pub(crate) const INVALID_HANDLE: SPXHANDLE = std::usize::MAX as SPXHANDLE;
// pub(crate) const INVALID_HANDLE: SPXHANDLE = std::ptr::null_mut();

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
    length: u32,
) -> Result<String> {
    let max_len = if length == 0 { 1024 } else { length };
    let s = String::with_capacity(max_len as usize + 1);
    let buf = r#try!(CString::new(s));
    let buf_ptr = buf.into_raw();
    hr!(cf(handle, buf_ptr, max_len))?;
    let output = unsafe { CString::from_raw(buf_ptr) };
    Ok(output.into_string()?)
}
