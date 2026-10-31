use super::{Evaluator, Result};
use crate::{Expr, ExprError, Rule};
use pest::iterators::{Pair, Pairs};

/// Infinite and NaN results are `#NUM!` errors
pub(crate) fn number(value: f64) -> Expr {
    if value.is_finite() {
        Expr::Number(value)
    } else {
        Expr::Error(ExprError::Number)
    }
}

impl Evaluator {
    pub(crate) fn get_formula(&self, args: &mut Pairs<Rule>) -> Result<Expr> {
        args.next().map_or(Err(ExprError::Value), |pair| self.parse_pair(pair))
    }

    pub(crate) fn get_opt_formula_with_default(&self, args: &mut Pairs<Rule>, default: Expr) -> Result<Expr> {
        args.next().map_or(Ok(default), |pair| self.parse_pair(pair))
    }

    /// Evaluates a function that takes one number
    pub(crate) fn parse_unary_number(&self, pair: Pair<Rule>, f: impl FnOnce(f64) -> f64) -> Result<Expr> {
        let mut args = pair.into_inner();

        match self.get_formula(&mut args)? {
            Expr::Number(operand) => Ok(number(f(operand))),
            _ => Err(ExprError::Value),
        }
    }

    /// Evaluates a function that takes two numbers
    pub(crate) fn parse_binary_number(&self, pair: Pair<Rule>, f: impl FnOnce(f64, f64) -> Expr) -> Result<Expr> {
        let mut args = pair.into_inner();
        let operand1 = self.get_formula(&mut args)?;
        let operand2 = self.get_formula(&mut args)?;

        match (operand1, operand2) {
            (Expr::Number(operand1), Expr::Number(operand2)) => Ok(f(operand1, operand2)),
            _ => Err(ExprError::Value),
        }
    }
}
