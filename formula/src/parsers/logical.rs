use super::{
    Evaluator, Result,
    helper::{ArgValue, as_bool},
};
use crate::{Expr, ExprError, Rule};
use pest::iterators::Pair;

impl Evaluator<'_> {
    pub(crate) fn parse_and(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.fold_bools(pair, |bool, b| bool && b)
    }

    pub(crate) fn parse_or(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.fold_bools(pair, |bool, b| bool || b)
    }

    pub(crate) fn parse_xor(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.fold_bools(pair, |bool, b| bool != b)
    }

    pub(crate) fn parse_not(&self, pair: Pair<Rule>) -> Result<Expr> {
        let mut args = pair.into_inner();

        Ok(Expr::Bool(!self.get_bool(&mut args)?))
    }

    /// Falls back on any spreadsheet error
    pub(crate) fn parse_iferror(&self, pair: Pair<Rule>) -> Result<Expr> {
        let mut args = pair.into_inner();
        let value = args.next().ok_or(ExprError::Value)?;
        let error_value = args.next().ok_or(ExprError::Value)?;

        match self.parse_pair(value) {
            Err(_) => self.parse_pair(error_value),
            value => value,
        }
    }

    /// Falls back on `#N/A` only
    pub(crate) fn parse_ifna(&self, pair: Pair<Rule>) -> Result<Expr> {
        let mut args = pair.into_inner();
        let value = args.next().ok_or(ExprError::Value)?;
        let na_value = args.next().ok_or(ExprError::Value)?;

        match self.parse_pair(value) {
            Err(ExprError::NotAvailable) => self.parse_pair(na_value),
            value => value,
        }
    }

    #[allow(clippy::unnecessary_wraps, clippy::unused_self)]
    pub(crate) fn parse_na(&self, _pair: Pair<Rule>) -> Result<Expr> {
        Ok(Expr::Error(ExprError::NotAvailable))
    }

    /// Only the selected branch is evaluated
    pub(crate) fn parse_if(&self, pair: Pair<Rule>) -> Result<Expr> {
        let mut args = pair.into_inner();
        let condition = self.get_bool(&mut args)?;
        let true_value = args.next().ok_or(ExprError::Value)?;
        let false_value = args.next();

        if condition {
            self.parse_pair(true_value)
        } else {
            false_value.map_or(Ok(Expr::Bool(false)), |pair| self.parse_pair(pair))
        }
    }

    /// Combines one or more boolean arguments
    fn fold_bools(&self, pair: Pair<Rule>, f: impl Fn(bool, bool) -> bool) -> Result<Expr> {
        let mut result = None;
        self.for_each_value(pair.into_inner(), |value| {
            // like Excel, numbers in references and arrays are booleans, their texts and empty cells don't count
            let bool = match value {
                ArgValue::Single(value) => as_bool(&value)?,
                ArgValue::InRange(Expr::Bool(bool)) => bool,
                ArgValue::InRange(Expr::Number(number)) => number != 0.0,
                ArgValue::InRange(_) => return Ok(()),
            };
            result = Some(result.map_or(bool, |result| f(result, bool)));
            Ok(())
        })?;
        result.map(Expr::Bool).ok_or(ExprError::Value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ExprError, Formula};

    #[test]
    fn test_parse_bool_types() {
        let formula = Formula::new("=AND(TRUE, FALSE(), TRUE)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Bool(false));

        let formula = Formula::new("=AND(TRUE, TRUE(), TRUE)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Bool(true));

        let formula = Formula::new("=OR(TRUE, FALSE(), TRUE)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Bool(true));

        let formula = Formula::new("=OR(TRUE, TRUE(), TRUE)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Bool(true));

        let formula = Formula::new("=XOR(TRUE, FALSE(), FALSE)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Bool(true));

        let formula = Formula::new("=XOR(TRUE, TRUE())").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Bool(false));

        let formula = Formula::new("=OR(TRUE, TRUE(), NOT(FALSE()))").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Bool(true));

        let formula = Formula::new("=NA()").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Error(ExprError::NotAvailable));

        let formula = Formula::new("=IFNA(NA(), 'not found')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("not found".to_string()));

        let formula = Formula::new("=IFNA(F.ADD(NA(), 1), 'not found')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("not found".to_string()));

        let formula = Formula::new("=IFNA(F.DIV(1, 0), 'not found')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Error(ExprError::DivisionByZero));

        let formula = Formula::new("=IFNA(F.ADD('a', 1), 'not found')").unwrap();
        assert_eq!(formula.parse().unwrap(), Expr::Error(ExprError::Value));

        let formula = Formula::new("=IFNA(5, 'not found')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(5.0));

        let formula = Formula::new("=IFERROR(NOT('a'), 'err')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("err".to_string()));

        let formula = Formula::new("=IFERROR(F.DIV(1, 0), 'err')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("err".to_string()));

        let formula = Formula::new("=IFERROR(SQRT(F.SUB(0, F.DIV(1, 0))), 'err')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("err".to_string()));

        let formula = Formula::new("=IFERROR(NA(), 'err')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("err".to_string()));

        let formula = Formula::new("=NOT(F.DIV(1, 0))").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Error(ExprError::DivisionByZero));

        let formula = Formula::new("=IFERROR(5, 'err')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(5.0));

        let formula = Formula::new("=IFERROR(ROUND(1, 0), 'err')");
        assert!(matches!(formula, Err(crate::error::Error::NotImplemented(_))));

        let formula = Formula::new("=IF(TRUE, 'true', 'false')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("true".to_string()));

        let formula = Formula::new("=IF(TRUE, 1, F.ADD('a', 1))").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(1.0));

        let formula = Formula::new("=IF(FALSE, 1)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Bool(false));

        let formula = Formula::new("=AND(TRUE, 1)").unwrap();
        assert_eq!(formula.parse().unwrap(), Expr::Error(ExprError::Value));

        let formula = Formula::new("=IF(1, 'a', 'b')").unwrap();
        assert_eq!(formula.parse().unwrap(), Expr::Error(ExprError::Value));

        let formula = Formula::new("=IF(FALSE, F.ADD('a', 1), 2)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(2.0));

        let formula = Formula::new("=IFERROR(F.ADD('a', 1), F.DIV(1, 0))").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Error(ExprError::DivisionByZero));

        let formula = Formula::new("=IFERROR(F.ADD('a', 1), F.ADD('a', 1))").unwrap();
        assert_eq!(formula.parse().unwrap(), Expr::Error(ExprError::Value));

        let formula = Formula::new("=XOR(TRUE, 'a')").unwrap();
        assert_eq!(formula.parse().unwrap(), Expr::Error(ExprError::Value));
    }
}
