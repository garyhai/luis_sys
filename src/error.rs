use std::ffi;
use failure::Fail;

#[derive(Fail, Debug)]
pub enum SpxError {
    #[fail(display = "speech API return error code: {}", _0)]
    ApiError(usize),
    #[fail(display = "an interior nul byte was found")]
    NulError,
    #[fail(display = "invalid UTF-8 string")]
    Utf8Error,
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

impl From<usize> for SpxError {
    fn from(code: usize) -> Self {
        assert!(code != 0);
        SpxError::ApiError(code)
    }
}
