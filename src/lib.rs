pub mod asr;
pub mod audio;
pub mod error;
pub(crate) mod properities;
pub(crate) mod speech_api;

pub use error::SpxError;
pub use speech_api::SPXHANDLE as Handle;

pub type Result<T = (), E = SpxError> = std::result::Result<T, E>;

pub(crate) fn hr(code: speech_api::SPXHR) -> Result {
    if code == 0 {
        Ok(())
    } else {
        Err(SpxError::from(code))
    }
}

pub trait SpxHandle {
    fn handle(&self) -> Handle;
}

// Quick and dirty "DRY"
#[macro_export]
macro_rules! DeriveSpxHandle {
    ( $name:ident, $release:ident $(, $check:ident)? ) => (
        impl Drop for $name {
            fn drop(&mut self) {
                unsafe {
                    $(
                        if !$check(self.handle) {
                            return;
                        }
                    )?
                    $release(self.handle);
                }
                log::trace!("{}({}) is released",
                            stringify!($name),
                            self.handle as usize);
            }
        }

        impl SpxHandle for $name {
            fn handle(&self) -> Handle {
                self.handle as Handle
            }
        }
    )
}
