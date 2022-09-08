use crate::{error::Error, Expr, Formula, Rule};
use anyhow::Result;
use pest::iterators::Pair;

impl Formula<'_> {
    pub(crate) fn parse_and(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let bool = Self::get_formula(&mut args, &rule_name)?;
        let bools = args.map(Self::parse_pair).collect::<Result<Vec<_>>>()?;

        let bool = match bool {
            Expr::Bool(mut bool) => {
                for b in bools {
                    match b {
                        Expr::Bool(b) => bool = bool && b,
                        _ => return Err(Error::Parser(rule_name).into()),
                    }
                }
                bool
            }
            _ => return Err(Error::Parser(rule_name).into()),
        };
        Ok(Expr::Bool(bool))
    }

    pub(crate) fn parse_or(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let bool = Self::get_formula(&mut args, &rule_name)?;
        let bools = args.map(Self::parse_pair).collect::<Result<Vec<_>>>()?;

        let bool = match bool {
            Expr::Bool(mut bool) => {
                for b in bools {
                    match b {
                        Expr::Bool(b) => bool = bool || b,
                        _ => return Err(Error::Parser(rule_name).into()),
                    }
                }
                bool
            }
            _ => return Err(Error::Parser(rule_name).into()),
        };
        Ok(Expr::Bool(bool))
    }

    pub(crate) fn parse_xor(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let bool = Self::get_formula(&mut args, &rule_name)?;
        let bools = args.map(Self::parse_pair).collect::<Result<Vec<_>>>()?;

        let bool = match bool {
            Expr::Bool(mut bool) => {
                for b in bools {
                    match b {
                        Expr::Bool(b) => bool = bool != b,
                        _ => return Err(Error::Parser(rule_name).into()),
                    }
                }
                bool
            }
            _ => return Err(Error::Parser(rule_name).into()),
        };

        Ok(Expr::Bool(bool))
    }

    pub(crate) fn parse_not(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let bool = Self::get_formula(&mut args, &rule_name)?;

        let bool = match bool {
            Expr::Bool(b) => !b,
            _ => return Err(Error::Parser(rule_name).into()),
        };
        Ok(Expr::Bool(bool))
    }

    pub(crate) fn parse_iferror(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let value = args.next();
        let error_value = args.next().ok_or_else(|| Error::Parser(rule_name.clone()))?;

        let value = value.unwrap_or_else(|| error_value.clone());
        let value = Self::parse_pair(value)
            .or_else(|_| Self::parse_pair(error_value))
            .map_err(|_| Error::Parser(rule_name))?;

        Ok(value)
    }

    pub(crate) fn parse_ifna(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let value = Self::get_formula(&mut args, &rule_name)?;
        let na_value = Self::get_formula(&mut args, &rule_name)?;

        let value = match value {
            Expr::Null => na_value,
            _ => return Err(Error::Parser(rule_name).into()),
        };

        Ok(value)
    }

    pub(crate) fn parse_if(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let value = Self::get_formula(&mut args, &rule_name)?;
        let true_value = Self::get_opt_formula_with_default(&mut args, Expr::Number(0.0))?;
        let false_value = Self::get_opt_formula_with_default(&mut args, Expr::Number(0.0))?;

        let value = match value {
            Expr::Bool(b) => {
                if b {
                    true_value
                } else {
                    false_value
                }
            }
            _ => return Err(Error::Parser(rule_name).into()),
        };

        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Formula;

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

        // let formula = Formula::new("=IFNA(x, 'not found')").unwrap();
        // let value = formula.parse().unwrap();
        // assert_eq!(value, Expr::String("not found".to_string()));

        let formula = Formula::new("=IFERROR(NOT('a'), 'err')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("err".to_string()));

        let formula = Formula::new("=IF(TRUE, 'true', 'false')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("true".to_string()));
    }
}
