use thiserror::Error;

/// Error type for the crate
///
/// `Error::Parser` means the formula is not valid
/// `Error::NotImplemented` means there is a function that is not implemented yet
#[derive(Error, Debug)]
pub enum Error {
    #[error("Parser Error: Invalid `{0}` expression")]
    Parser(String),
    #[error("Not Implemented Yet: `{0}` function is not implemented yet")]
    NotImplemented(String),
}
