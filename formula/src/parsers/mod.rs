pub(crate) mod basic;
pub(crate) mod date_and_time;
pub(crate) mod engineering;
pub(crate) mod financial;
mod helper;
pub(crate) mod logical;
pub(crate) mod math;
pub(crate) mod operator;
pub(crate) mod reference;
pub(crate) mod statistical;
pub(crate) mod text;
pub(crate) mod web;

use crate::{Cells, ExprError};

/// Evaluating stops at a spreadsheet error, functions pass it on until `IFERROR`, `IFNA` or the result
pub(crate) type Result<T> = std::result::Result<T, ExprError>;

/// Evaluates parsed formulas, with the cells that their references read
pub(crate) struct Evaluator<'c> {
    pub(crate) cells: &'c dyn Cells,
}
