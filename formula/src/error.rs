use thiserror::Error;

/// Error type for the crate
///
/// `Error::Parser` means the formula is not valid
/// `Error::NotImplemented` means there is a function that is not implemented yet
/// `Error::Nesting` means the formula has more nested levels than Excel allows
#[derive(Error, Debug)]
pub enum Error {
    #[error("Parser Error: Invalid `{0}` expression")]
    Parser(String),
    #[error("Not Implemented Yet: `{0}` function is not implemented yet")]
    NotImplemented(String),
    #[error("Nesting Error: The formula is nested more than {0} levels deep")]
    Nesting(usize),
}
