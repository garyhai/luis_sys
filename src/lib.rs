pub mod error;
pub(crate) mod properities;
pub mod speech_api;
pub use error::SpxError;
pub type Result<T = (), E = SpxError> = std::result::Result<T, E>;

pub fn hr(code: speech_api::SPXHR) -> Result {
    if code == 0 {
        Ok(())
    } else {
        Err(SpxError::from(code))
    }
}

pub mod asr;