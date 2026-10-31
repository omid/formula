use super::helper::parser_error;
use crate::{Expr, Formula, Result, Rule};
use pest::iterators::Pair;
use std::cmp::Ordering;

impl Formula<'_> {
    pub(crate) fn parse_add(pair: Pair<Rule>) -> Result<Expr> {
        Self::parse_binary_number(pair, |operand1, operand2| operand1 + operand2)
    }

    pub(crate) fn parse_sub(pair: Pair<Rule>) -> Result<Expr> {
        Self::parse_binary_number(pair, |operand1, operand2| operand1 - operand2)
    }

    pub(crate) fn parse_mul(pair: Pair<Rule>) -> Result<Expr> {
        Self::parse_binary_number(pair, |operand1, operand2| operand1 * operand2)
    }

    /// Dividing by zero gives an infinite or NaN number, which becomes `Expr::Null`
    pub(crate) fn parse_div(pair: Pair<Rule>) -> Result<Expr> {
        Self::parse_binary_number(pair, |operand1, operand2| operand1 / operand2)
    }

    pub(crate) fn parse_pow(pair: Pair<Rule>) -> Result<Expr> {
        Self::parse_binary_number(pair, f64::powf)
    }

    pub(crate) fn parse_eq(pair: Pair<Rule>) -> Result<Expr> {
        Self::parse_comparison(pair, Ordering::is_eq)
    }

    pub(crate) fn parse_ne(pair: Pair<Rule>) -> Result<Expr> {
        Self::parse_comparison(pair, Ordering::is_ne)
    }

    pub(crate) fn parse_gt(pair: Pair<Rule>) -> Result<Expr> {
        Self::parse_comparison(pair, Ordering::is_gt)
    }

    pub(crate) fn parse_lt(pair: Pair<Rule>) -> Result<Expr> {
        Self::parse_comparison(pair, Ordering::is_lt)
    }

    pub(crate) fn parse_gte(pair: Pair<Rule>) -> Result<Expr> {
        Self::parse_comparison(pair, Ordering::is_ge)
    }

    pub(crate) fn parse_lte(pair: Pair<Rule>) -> Result<Expr> {
        Self::parse_comparison(pair, Ordering::is_le)
    }

    pub(crate) fn parse_percent(pair: Pair<Rule>) -> Result<Expr> {
        Self::parse_unary_number(pair, |operand| operand / 100.0)
    }

    pub(crate) fn parse_negate(pair: Pair<Rule>) -> Result<Expr> {
        Self::parse_unary_number(pair, |operand| -operand)
    }

    /// Compares two values of the same type.
    /// Like Excel, numbers that only differ by floating point noise (e.g. `0.1 + 0.2` and `0.3`) are equal.
    fn parse_comparison(pair: Pair<Rule>, check: fn(Ordering) -> bool) -> Result<Expr> {
        let rule = pair.as_rule();
        let mut args = pair.into_inner();
        let operand1 = Self::get_formula(&mut args, rule)?;
        let operand2 = Self::get_formula(&mut args, rule)?;

        let ordering = match (&operand1, &operand2) {
            (Expr::Number(n1), Expr::Number(n2)) if (n1 - n2).abs() <= 1e-15 * n1.abs().max(n2.abs()) => {
                Some(Ordering::Equal)
            }
            (Expr::Number(_), Expr::Number(_))
            | (Expr::String(_), Expr::String(_))
            | (Expr::Time(_), Expr::Time(_))
            | (Expr::Datetime(_), Expr::Datetime(_))
            | (Expr::Date(_), Expr::Date(_))
            | (Expr::Bool(_), Expr::Bool(_)) => operand1.partial_cmp(&operand2),
            _ => None,
        };
        ordering.map(|o| Expr::Bool(check(o))).ok_or_else(|| parser_error(rule))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Formula;

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_parse_operator_types() {
        let formula = Formula::new("=F.ADD(5,6)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(11.0));

        let formula = Formula::new("=F.SUB(5,6)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(-1.0));

        let formula = Formula::new("=F.MUL(5,6)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(30.0));

        let formula = Formula::new("=F.DIV(10,2)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(5.0));

        let formula = Formula::new("=F.DIV(10,0)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Null);

        let formula = Formula::new("=F.DIV(0,0)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Null);

        let formula = Formula::new("=F.DIV(-10,4)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(-2.5));

        let formula = Formula::new("=F.POW(5,2)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(25.0));

        let formula = Formula::new("=F.EQ(5,2)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Bool(false));

        let formula = Formula::new("=F.NE('a','a')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Bool(false));

        let formula = Formula::new("=F.EQ(F.ADD(0.1,0.2),0.3)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Bool(true));

        let formula = Formula::new("=F.NE(F.ADD(0.1,0.2),0.3)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Bool(false));

        let formula = Formula::new("=F.NE(1,1.00000001)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Bool(true));

        let formula = Formula::new("=F.GT(TRUE,FALSE)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Bool(true));

        let formula = Formula::new("=F.EQ(1,'1')").unwrap();
        assert!(formula.parse().is_err());

        let formula = Formula::new("=F.GT('a','a')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Bool(false));

        let formula = Formula::new("=F.LT(5,6)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Bool(true));

        let formula = Formula::new("=F.GTE(5,5)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Bool(true));

        let formula = Formula::new("=F.LTE(6,5)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Bool(false));

        let formula = Formula::new("=F.PERCENT(5)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(0.05));

        let formula = Formula::new("=F.NEGATE(5)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(-5.0));

        let formula = Formula::new("=F.LT('a','b')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Bool(true));

        let formula = Formula::new("=F.GTE(DATE(2020,1,2),DATE(2020,1,1))").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Bool(true));

        let formula = Formula::new("=F.EQ(1e-20,2e-20)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Bool(false));

        let formula = Formula::new("=F.NE(TRUE,FALSE)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Bool(true));

        let formula = Formula::new("=F.ADD(F.DIV(1,0),1)").unwrap();
        assert!(formula.parse().is_err());
    }
}
