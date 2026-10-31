use super::{helper::parser_error, Halt, Result};
use crate::{error::Error, Expr, ExprError, Formula, Rule};
use pest::iterators::Pair;

impl Formula<'_> {
    pub(crate) fn parse_and(pair: Pair<Rule>) -> Result<Expr> {
        Self::fold_bools(pair, |bool, b| bool && b)
    }

    pub(crate) fn parse_or(pair: Pair<Rule>) -> Result<Expr> {
        Self::fold_bools(pair, |bool, b| bool || b)
    }

    pub(crate) fn parse_xor(pair: Pair<Rule>) -> Result<Expr> {
        Self::fold_bools(pair, |bool, b| bool != b)
    }

    pub(crate) fn parse_not(pair: Pair<Rule>) -> Result<Expr> {
        let rule = pair.as_rule();
        let mut args = pair.into_inner();

        match Self::get_formula(&mut args, rule)? {
            Expr::Bool(b) => Ok(Expr::Bool(!b)),
            _ => Err(parser_error(rule)),
        }
    }

    /// Falls back on spreadsheet errors and on invalid formulas.
    /// Functions that are not implemented yet still return their error.
    pub(crate) fn parse_iferror(pair: Pair<Rule>) -> Result<Expr> {
        let rule = pair.as_rule();
        let mut args = pair.into_inner();
        let value = args.next().ok_or_else(|| parser_error(rule))?;
        let error_value = args.next().ok_or_else(|| parser_error(rule))?;

        match Self::parse_pair(value) {
            Err(Halt::Value(_) | Halt::Error(Error::Parser(_))) => Self::parse_pair(error_value),
            value => value,
        }
    }

    /// Falls back on `#N/A` only
    pub(crate) fn parse_ifna(pair: Pair<Rule>) -> Result<Expr> {
        let rule = pair.as_rule();
        let mut args = pair.into_inner();
        let value = args.next().ok_or_else(|| parser_error(rule))?;
        let na_value = args.next().ok_or_else(|| parser_error(rule))?;

        match Self::parse_pair(value) {
            Err(Halt::Value(ExprError::NotAvailable)) => Self::parse_pair(na_value),
            value => value,
        }
    }

    #[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
    pub(crate) fn parse_na(_pair: Pair<Rule>) -> Result<Expr> {
        Ok(Expr::Error(ExprError::NotAvailable))
    }

    /// Only the selected branch is evaluated
    pub(crate) fn parse_if(pair: Pair<Rule>) -> Result<Expr> {
        let rule = pair.as_rule();
        let mut args = pair.into_inner();
        let condition = Self::get_formula(&mut args, rule)?;
        let true_value = args.next().ok_or_else(|| parser_error(rule))?;
        let false_value = args.next();

        match condition {
            Expr::Bool(true) => Self::parse_pair(true_value),
            Expr::Bool(false) => false_value.map_or(Ok(Expr::Bool(false)), Self::parse_pair),
            _ => Err(parser_error(rule)),
        }
    }

    /// Combines one or more boolean arguments
    fn fold_bools(pair: Pair<Rule>, f: impl Fn(bool, bool) -> bool) -> Result<Expr> {
        let rule = pair.as_rule();
        let mut bools = pair.into_inner().map(|arg| match Self::parse_pair(arg)? {
            Expr::Bool(b) => Ok(b),
            _ => Err(parser_error(rule)),
        });
        let first = bools.next().ok_or_else(|| parser_error(rule))??;

        bools.try_fold(first, |bool, b| Ok(f(bool, b?))).map(Expr::Bool)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ExprError, Formula};

    #[test]
    #[allow(clippy::too_many_lines)]
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
        assert!(formula.parse().is_err());

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

        let formula = Formula::new("=IFERROR(ROUND(1, 0), 'err')").unwrap();
        assert!(formula.parse().is_err());

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
        assert!(formula.parse().is_err());

        let formula = Formula::new("=IF(1, 'a', 'b')").unwrap();
        assert!(formula.parse().is_err());

        let formula = Formula::new("=IF(FALSE, F.ADD('a', 1), 2)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(2.0));

        let formula = Formula::new("=IFERROR(F.ADD('a', 1), F.DIV(1, 0))").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Error(ExprError::DivisionByZero));

        let formula = Formula::new("=IFERROR(F.ADD('a', 1), F.ADD('a', 1))").unwrap();
        assert!(formula.parse().is_err());

        let formula = Formula::new("=XOR(TRUE, 'a')").unwrap();
        assert!(formula.parse().is_err());
    }
}
