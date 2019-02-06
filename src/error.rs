pub use failure::{Error, Fail};

/// Export all enumerations.
pub use SpxError::*;

/// A set of common errors.
#[derive(Fail, Debug)]
pub enum SpxError {
    /// As std::option::NoneError
    #[fail(display = "there is nothing")]
    Nothing,
    /// As ErrorKind::NotFound
    #[fail(display = "entity not found")]
    NotFound,
    /// As InvalidData and InvalidInput
    #[fail(display = "invalid data or parameter")]
    Invalid,
    /// As ErrorKind::AlreadyExists
    #[fail(display = "entity already exists")]
    AlreadyExists,
    /// Something unknown.
    #[fail(display = "something unknown")]
    Unknown,
    /// Unimplemented method.
    #[fail(display = "method is not implemented")]
    Unimplemented,
    /// Misc error with description.
    #[fail(display = "{}", _0)]
    Misc(String),
}
