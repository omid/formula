pub(crate) mod basic;
pub(crate) mod date_and_time;
pub(crate) mod engineering;
pub(crate) mod financial;
mod helper;
pub(crate) mod logical;
pub(crate) mod math;
pub(crate) mod operator;
pub(crate) mod statistical;
pub(crate) mod text;
pub(crate) mod web;

use crate::{error::Error, ExprError};

/// Stops an evaluation early
#[derive(Debug)]
pub(crate) enum Halt {
    /// The formula is invalid or uses a function that is not implemented yet
    Error(Error),
    /// A spreadsheet error value, functions pass it on until `IFERROR`, `IFNA` or the result
    Value(ExprError),
}

impl From<Error> for Halt {
    fn from(error: Error) -> Self {
        Self::Error(error)
    }
}

pub(crate) type Result<T> = std::result::Result<T, Halt>;
