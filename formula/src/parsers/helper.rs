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

/// Like Excel, an empty cell is 0 where a number is needed
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn as_number(value: Expr) -> Result<f64> {
    match value {
        Expr::Number(number) => Ok(number),
        Expr::Null => Ok(0.0),
        _ => Err(ExprError::Value),
    }
}

/// Like Excel, an empty cell is an empty text where a text is needed
pub(crate) fn as_text(value: Expr) -> Result<String> {
    match value {
        Expr::String(text) => Ok(text),
        Expr::Null => Ok(String::new()),
        _ => Err(ExprError::Value),
    }
}

/// Like Excel, an empty cell is FALSE where a boolean is needed
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn as_bool(value: Expr) -> Result<bool> {
    match value {
        Expr::Bool(bool) => Ok(bool),
        Expr::Null => Ok(false),
        _ => Err(ExprError::Value),
    }
}

impl Evaluator<'_> {
    pub(crate) fn get_formula(&self, args: &mut Pairs<Rule>) -> Result<Expr> {
        args.next().map_or(Err(ExprError::Value), |pair| self.parse_pair(pair))
    }

    pub(crate) fn get_opt_formula_with_default(&self, args: &mut Pairs<Rule>, default: Expr) -> Result<Expr> {
        args.next().map_or(Ok(default), |pair| self.parse_pair(pair))
    }

    pub(crate) fn get_number(&self, args: &mut Pairs<Rule>) -> Result<f64> {
        as_number(self.get_formula(args)?)
    }

    pub(crate) fn get_text(&self, args: &mut Pairs<Rule>) -> Result<String> {
        as_text(self.get_formula(args)?)
    }

    pub(crate) fn get_bool(&self, args: &mut Pairs<Rule>) -> Result<bool> {
        as_bool(self.get_formula(args)?)
    }

    /// A missing optional argument is `default`
    pub(crate) fn get_opt_number(&self, args: &mut Pairs<Rule>, default: f64) -> Result<f64> {
        as_number(self.get_opt_formula_with_default(args, Expr::Number(default))?)
    }

    /// A missing optional argument is `default`
    pub(crate) fn get_opt_bool(&self, args: &mut Pairs<Rule>, default: bool) -> Result<bool> {
        as_bool(self.get_opt_formula_with_default(args, Expr::Bool(default))?)
    }

    /// Evaluates a function that takes one number
    pub(crate) fn parse_unary_number(&self, pair: Pair<Rule>, f: impl FnOnce(f64) -> f64) -> Result<Expr> {
        let mut args = pair.into_inner();
        let operand = self.get_number(&mut args)?;

        Ok(number(f(operand)))
    }

    /// Evaluates a function that takes two numbers
    pub(crate) fn parse_binary_number(&self, pair: Pair<Rule>, f: impl FnOnce(f64, f64) -> Expr) -> Result<Expr> {
        let mut args = pair.into_inner();
        let operand1 = self.get_number(&mut args)?;
        let operand2 = self.get_number(&mut args)?;

        Ok(f(operand1, operand2))
    }
}
