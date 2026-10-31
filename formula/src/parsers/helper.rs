use super::Result;
use crate::{Expr, ExprError, Formula, Rule};
use pest::iterators::{Pair, Pairs};

/// Infinite and NaN results are `#NUM!` errors
pub(crate) fn number(value: f64) -> Expr {
    if value.is_finite() {
        Expr::Number(value)
    } else {
        Expr::Error(ExprError::Number)
    }
}

impl Formula<'_> {
    pub(crate) fn get_formula(args: &mut Pairs<Rule>) -> Result<Expr> {
        args.next().map_or(Err(ExprError::Value), Self::parse_pair)
    }

    pub(crate) fn get_opt_formula_with_default(args: &mut Pairs<Rule>, default: Expr) -> Result<Expr> {
        args.next().map_or(Ok(default), Self::parse_pair)
    }

    /// Evaluates a function that takes one number
    pub(crate) fn parse_unary_number(pair: Pair<Rule>, f: impl FnOnce(f64) -> f64) -> Result<Expr> {
        let mut args = pair.into_inner();

        match Self::get_formula(&mut args)? {
            Expr::Number(operand) => Ok(number(f(operand))),
            _ => Err(ExprError::Value),
        }
    }

    /// Evaluates a function that takes two numbers
    pub(crate) fn parse_binary_number(pair: Pair<Rule>, f: impl FnOnce(f64, f64) -> Expr) -> Result<Expr> {
        let mut args = pair.into_inner();
        let operand1 = Self::get_formula(&mut args)?;
        let operand2 = Self::get_formula(&mut args)?;

        match (operand1, operand2) {
            (Expr::Number(operand1), Expr::Number(operand2)) => Ok(f(operand1, operand2)),
            _ => Err(ExprError::Value),
        }
    }
}
