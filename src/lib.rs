pub mod asr;
pub mod audio;
pub mod error;

pub use error::SpxError;
pub use speech_api::SPXHANDLE as Handle;
pub type Result<T = (), E = SpxError> = std::result::Result<T, E>;
pub trait SpxHandle {
    fn handle(&self) -> Handle;
}

pub(crate) mod macros;
pub(crate) mod properities;
pub(crate) mod speech_api;
pub(crate) fn hr(code: speech_api::SPXHR) -> Result {
    if code == 0 {
        Ok(())
    } else {
        Err(SpxError::from(code))
    }
}
