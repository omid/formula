use super::{Evaluator, Result};
use crate::{Expr, Rule};
use pest::iterators::Pair;

impl Evaluator<'_> {
    #[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps, clippy::unused_self)]
    pub(crate) fn parse_num(&self, pair: Pair<Rule>) -> Result<Expr> {
        let number = pair.as_str().trim().parse().unwrap();
        Ok(Expr::Number(number))
    }

    #[allow(clippy::unnecessary_wraps, clippy::unused_self)]
    pub(crate) fn parse_string(&self, pair: Pair<Rule>) -> Result<Expr> {
        Ok(Expr::String(Self::unescape(pair.into_inner().as_str())))
    }

    /// Decodes the escape sequences that the grammar accepts, in a single pass
    fn unescape(escaped: &str) -> String {
        let mut string = String::with_capacity(escaped.len());
        let mut chars = escaped.chars();
        while let Some(c) = chars.next() {
            if c != '\\' {
                string.push(c);
                continue;
            }
            match chars.next() {
                Some('b') => string.push('\u{0008}'),
                Some('f') => string.push('\u{000C}'),
                Some('n') => string.push('\n'),
                Some('r') => string.push('\r'),
                Some('t') => string.push('\t'),
                Some('u') => {
                    let code = chars.by_ref().take(4).collect::<String>();
                    let c = u32::from_str_radix(&code, 16).ok().and_then(char::from_u32);
                    string.push(c.unwrap_or(char::REPLACEMENT_CHARACTER));
                }
                // quotes and backslash
                Some(c) => string.push(c),
                None => {}
            }
        }
        string
    }

    #[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps, clippy::unused_self)]
    pub(crate) fn parse_true(&self, _pair: Pair<Rule>) -> Result<Expr> {
        Ok(Expr::Bool(true))
    }

    #[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps, clippy::unused_self)]
    pub(crate) fn parse_false(&self, _pair: Pair<Rule>) -> Result<Expr> {
        Ok(Expr::Bool(false))
    }

    #[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
    pub(crate) fn parse_array(&self, pair: Pair<Rule>) -> Result<Expr> {
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
                    // An error element doesn't make the whole array an error
                    row.push(self.parse_pair(ipair).unwrap_or_else(Expr::Error));
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
    use crate::{ExprError, Formula};

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

        let formula = Formula::new("='\\'TEST\\\"'").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("'TEST\"".to_string()));

        let formula = Formula::new(r"='a\\b\nc\u0041'").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("a\\b\ncA".to_string()));

        let formula = Formula::new("=\"it's\"").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("it's".to_string()));

        assert!(Formula::new("='it's'").is_err());
        assert!(Formula::new("=F.ADD(1,2) garbage").is_err());
        assert!(Formula::new("=1+1").is_err());

        let formula = Formula::new("={1, F.DIV(1, 0)}").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(
            value,
            Expr::Array(vec![Expr::Number(1.0), Expr::Error(ExprError::DivisionByZero)])
        );

        let formula = Formula::new("={F.ADD('a', 1); 2}").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(
            value,
            Expr::Array(vec![
                Expr::Array(vec![Expr::Error(ExprError::Value)]),
                Expr::Array(vec![Expr::Number(2.0)])
            ])
        );

        let formula = Formula::new("={}").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Array(vec![]));

        let formula = Formula::new("={'TEST', 1}").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(
            value,
            Expr::Array(vec![Expr::String("TEST".to_string()), Expr::Number(1.0)]),
        );

        let formula = Formula::new("={'TEST', SUM(1,2); 2, TRUE}").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(
            value,
            Expr::Array(vec![
                Expr::Array(vec![Expr::String("TEST".to_string()), Expr::Number(3.0)]),
                Expr::Array(vec![Expr::Number(2.0), Expr::Bool(true)]),
            ])
        );

        let formula = Formula::new("=\"say \\\"hi\\\"\"").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("say \"hi\"".to_string()));

        let formula = Formula::new("='\\uD800'").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("\u{FFFD}".to_string()));

        let formula = Formula::new("=''").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String(String::new()));

        let formula = Formula::new("= 1 ").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(1.0));
    }
}
