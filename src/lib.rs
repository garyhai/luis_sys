#[macro_use]
extern crate bitflags;

pub mod asr;
pub mod error;
pub use error::SpxError;
pub use speech_api::SPXHANDLE as Handle;
pub type Result<T = (), E = SpxError> = std::result::Result<T, E>;
pub trait SpxHandle {
    fn handle(&self) -> Handle;
}
pub(crate) mod audio;
pub(crate) mod macros;
pub(crate) mod properities;
pub(crate) mod speech_api;

use speech_api::SPXHR;
use std::{ffi::CString, os::raw::c_char};

pub(crate) fn hr(code: SPXHR) -> Result {
    if code == 0 {
        Ok(())
    } else {
        Err(SpxError::from(code))
    }
}

pub(crate) fn get_cf_string(
    cf: unsafe extern "C" fn(Handle, *mut c_char, u32) -> SPXHR,
    handle: Handle,
    length: u32,
) -> Result<String> {
    let max_len = if length == 0 { 1024 } else { length };
    let s = String::with_capacity(max_len as usize + 1);
    let buf = r#try!(CString::new(s));
    let buf_ptr = buf.into_raw();
    unsafe {
        hr(cf(handle, buf_ptr, max_len))?;
        let output = CString::from_raw(buf_ptr);
        Ok(output.into_string()?)
    }
}
