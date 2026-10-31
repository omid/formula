use super::{Evaluator, Result, helper::number};
use crate::{Expr, ExprError, Rule};
use pest::iterators::Pair;
use std::cmp::Ordering;

impl Evaluator<'_> {
    pub(crate) fn parse_add(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_binary_number(pair, |operand1, operand2| number(operand1 + operand2))
    }

    pub(crate) fn parse_sub(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_binary_number(pair, |operand1, operand2| number(operand1 - operand2))
    }

    pub(crate) fn parse_mul(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_binary_number(pair, |operand1, operand2| number(operand1 * operand2))
    }

    pub(crate) fn parse_div(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_binary_number(pair, |operand1, operand2| {
            if operand2 == 0.0 {
                Expr::Error(ExprError::DivisionByZero)
            } else {
                number(operand1 / operand2)
            }
        })
    }

    pub(crate) fn parse_pow(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_binary_number(pair, |operand1, operand2| number(operand1.powf(operand2)))
    }

    pub(crate) fn parse_eq(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_comparison(pair, Ordering::is_eq)
    }

    pub(crate) fn parse_ne(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_comparison(pair, Ordering::is_ne)
    }

    pub(crate) fn parse_gt(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_comparison(pair, Ordering::is_gt)
    }

    pub(crate) fn parse_lt(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_comparison(pair, Ordering::is_lt)
    }

    pub(crate) fn parse_gte(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_comparison(pair, Ordering::is_ge)
    }

    pub(crate) fn parse_lte(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_comparison(pair, Ordering::is_le)
    }

    pub(crate) fn parse_percent(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_unary_number(pair, |operand| operand / 100.0)
    }

    pub(crate) fn parse_negate(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_unary_number(pair, |operand| -operand)
    }

    /// Compares two values of the same type. Like Excel:
    /// - numbers that only differ by floating point noise (e.g. `0.1 + 0.2` and `0.3`) are equal
    /// - texts are compared case-insensitively, `EXACT` is the case-sensitive comparison
    /// - an empty cell is 0, an empty text or FALSE, depending on the other value
    fn parse_comparison(&self, pair: Pair<Rule>, check: fn(Ordering) -> bool) -> Result<Expr> {
        let mut args = pair.into_inner();
        let operand1 = self.get_formula(&mut args)?;
        let operand2 = self.get_formula(&mut args)?;
        let operand1 = Self::fill_empty(operand1, &operand2);
        let operand2 = Self::fill_empty(operand2, &operand1);

        let ordering = match (&operand1, &operand2) {
            (Expr::Number(n1), Expr::Number(n2)) if (n1 - n2).abs() <= 1e-15 * n1.abs().max(n2.abs()) => {
                Some(Ordering::Equal)
            }
            (Expr::String(s1), Expr::String(s2)) => Some(s1.to_lowercase().cmp(&s2.to_lowercase())),
            (Expr::Number(_), Expr::Number(_))
            | (Expr::Time(_), Expr::Time(_))
            | (Expr::Datetime(_), Expr::Datetime(_))
            | (Expr::Date(_), Expr::Date(_))
            | (Expr::Bool(_), Expr::Bool(_))
            | (Expr::Null, Expr::Null) => operand1.partial_cmp(&operand2),
            _ => None,
        };
        ordering.map(|o| Expr::Bool(check(o))).ok_or(ExprError::Value)
    }

    /// An empty cell as the empty value of `other`'s type
    fn fill_empty(value: Expr, other: &Expr) -> Expr {
        match (value, other) {
            (Expr::Null, Expr::Number(_)) => Expr::Number(0.0),
            (Expr::Null, Expr::String(_)) => Expr::String(String::new()),
            (Expr::Null, Expr::Bool(_)) => Expr::Bool(false),
            (value, _) => value,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ExprError, Formula};

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
        assert_eq!(value, Expr::Error(ExprError::DivisionByZero));

        let formula = Formula::new("=F.DIV(0,0)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Error(ExprError::DivisionByZero));

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
        assert_eq!(formula.parse().unwrap(), Expr::Error(ExprError::Value));

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

        let formula = Formula::new("=F.EQ('a','A')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Bool(true));

        let formula = Formula::new("=F.NE('Straße','STRASSE')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Bool(true));

        let formula = Formula::new("=F.NE('Ä','ä')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Bool(false));

        let formula = Formula::new("=F.LT('a','B')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Bool(true));

        let formula = Formula::new("=F.GTE('b','A')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Bool(true));

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
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Error(ExprError::DivisionByZero));

        // the first error wins
        let formula = Formula::new("=F.ADD(F.ADD('a',1),F.DIV(1,0))").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Error(ExprError::Value));

        let formula = Formula::new("=F.EQ(NA(),1)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Error(ExprError::NotAvailable));
    }
}
