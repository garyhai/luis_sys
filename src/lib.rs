pub mod error;
pub(crate) mod properities;
pub(crate) mod speech_api;
pub use error::SpxError;
pub type Result<T = (), E = SpxError> = std::result::Result<T, E>;
pub use speech_api::SPXHANDLE as Handle;

pub(crate) fn hr(code: speech_api::SPXHR) -> Result {
    if code == 0 {
        Ok(())
    } else {
        Err(SpxError::from(code))
    }
}

pub mod audio;
pub mod asr;

pub trait SpxHandle {
    fn handle(&self) -> Handle;
}