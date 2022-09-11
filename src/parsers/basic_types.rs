use crate::{Expr, Formula, Result, Rule};
use pest::iterators::Pair;

impl Formula<'_> {
    #[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
    pub(crate) fn parse_num(pair: Pair<Rule>) -> Result<Expr> {
        let number = pair.as_str().trim().parse().unwrap();
        Ok(Expr::Number(number))
    }

    #[allow(clippy::unnecessary_wraps)]
    pub(crate) fn parse_string(pair: Pair<Rule>) -> Result<Expr> {
        let string = pair.into_inner().as_str().to_string();
        Ok(Expr::String(string))
    }

    #[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
    pub(crate) fn parse_true(_pair: Pair<Rule>) -> Result<Expr> {
        Ok(Expr::Bool(true))
    }

    #[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
    pub(crate) fn parse_false(_pair: Pair<Rule>) -> Result<Expr> {
        Ok(Expr::Bool(false))
    }

    #[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
    pub(crate) fn parse_array(pair: Pair<Rule>) -> Result<Expr> {
        let inner = pair.into_inner();

        let mut table = Vec::new();
        let mut row = Vec::new();

        for ipair in inner {
            let rule_type = ipair.as_rule();

            match rule_type {
                Rule::array_col_sep => {}
                Rule::array_row_sep => {
                    table.push(Expr::Array(row));
                    row = Vec::new();
                }
                _ => {
                    row.push(Self::parse_pair(ipair)?);
                }
            }
        }

        if table.is_empty() {
            Ok(Expr::Array(row))
        } else {
            table.push(Expr::Array(row));
            Ok(Expr::Array(table))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Formula;

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_parse_basic_types() {
        let formula = Formula::new("=TRUE").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Bool(true));

        let formula = Formula::new("=TRUE()").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Bool(true));

        let formula = Formula::new("=25").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(25.0));

        let formula = Formula::new("='TEST'").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("TEST".to_string()));

        let formula = Formula::new("={}").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Array(vec![]));

        let formula = Formula::new("={'TEST', 1}").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(
            value,
            Expr::Array(vec![Expr::String("TEST".to_string()), Expr::Number(1.0)]),
        );

        let formula = Formula::new("={'TEST', 1; 2, TRUE}").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(
            value,
            Expr::Array(vec![
                Expr::Array(vec![Expr::String("TEST".to_string()), Expr::Number(1.0)]),
                Expr::Array(vec![Expr::Number(2.0), Expr::Bool(true)]),
            ])
        );
    }
}
