use crate::{error::Error, Expr, Formula, Result, Rule};
use pest::iterators::{Pair, Pairs};

/// Builds the error for an invalid call of `rule`, only formatting the rule name when an error actually happens
pub(crate) fn parser_error(rule: Rule) -> Error {
    Error::Parser(format!("{rule:?}"))
}

/// Spreadsheet errors (`#NUM!`, `#DIV/0!`, ...) are represented as `Expr::Null`
pub(crate) fn number(value: f64) -> Expr {
    if value.is_finite() {
        Expr::Number(value)
    } else {
        Expr::Null
    }
}

impl Formula<'_> {
    pub(crate) fn get_formula(args: &mut Pairs<Rule>, rule: Rule) -> Result<Expr> {
        Self::parse_pair(args.next().ok_or_else(|| parser_error(rule))?)
    }

    pub(crate) fn get_opt_formula_with_default(args: &mut Pairs<Rule>, default: Expr) -> Result<Expr> {
        args.next().map_or(Ok(default), Self::parse_pair)
    }

    /// Evaluates a function that takes one number
    pub(crate) fn parse_unary_number(pair: Pair<Rule>, f: impl FnOnce(f64) -> f64) -> Result<Expr> {
        let rule = pair.as_rule();
        let mut args = pair.into_inner();

        match Self::get_formula(&mut args, rule)? {
            Expr::Number(operand) => Ok(number(f(operand))),
            _ => Err(parser_error(rule)),
        }
    }

    /// Evaluates a function that takes two numbers
    pub(crate) fn parse_binary_number(pair: Pair<Rule>, f: impl FnOnce(f64, f64) -> f64) -> Result<Expr> {
        let rule = pair.as_rule();
        let mut args = pair.into_inner();
        let operand1 = Self::get_formula(&mut args, rule)?;
        let operand2 = Self::get_formula(&mut args, rule)?;

        match (operand1, operand2) {
            (Expr::Number(operand1), Expr::Number(operand2)) => Ok(number(f(operand1, operand2))),
            _ => Err(parser_error(rule)),
        }
    }
}
