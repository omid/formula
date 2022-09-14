use crate::{error::Error, Expr, Formula, Result, Rule};
use pest::iterators::Pair;

impl Formula<'_> {
    pub(crate) fn parse_add(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let operand1 = Self::get_formula(&mut args, &rule_name)?;
        let operand2 = Self::get_formula(&mut args, &rule_name)?;

        let res = match (operand1, operand2) {
            (Expr::Number(operand1), Expr::Number(operand2)) => operand1 + operand2,
            _ => return Err(Error::Parser(rule_name)),
        };
        Ok(Expr::Number(res))
    }

    pub(crate) fn parse_sub(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let operand1 = Self::get_formula(&mut args, &rule_name)?;
        let operand2 = Self::get_formula(&mut args, &rule_name)?;

        let res = match (operand1, operand2) {
            (Expr::Number(operand1), Expr::Number(operand2)) => operand1 - operand2,
            _ => return Err(Error::Parser(rule_name)),
        };
        Ok(Expr::Number(res))
    }

    pub(crate) fn parse_mul(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let operand1 = Self::get_formula(&mut args, &rule_name)?;
        let operand2 = Self::get_formula(&mut args, &rule_name)?;

        let res = match (operand1, operand2) {
            (Expr::Number(operand1), Expr::Number(operand2)) => operand1 * operand2,
            _ => return Err(Error::Parser(rule_name)),
        };
        Ok(Expr::Number(res))
    }

    pub(crate) fn parse_div(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let operand1 = Self::get_formula(&mut args, &rule_name)?;
        let operand2 = Self::get_formula(&mut args, &rule_name)?;

        let res = match (operand1, operand2) {
            (Expr::Number(operand1), Expr::Number(operand2)) => {
                if operand2 == 0.0 {
                    Expr::Null
                } else {
                    Expr::Number(operand1 / operand2)
                }
            }
            _ => return Err(Error::Parser(rule_name)),
        };
        Ok(res)
    }

    pub(crate) fn parse_pow(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let operand1 = Self::get_formula(&mut args, &rule_name)?;
        let operand2 = Self::get_formula(&mut args, &rule_name)?;

        let res = match (operand1, operand2) {
            (Expr::Number(operand1), Expr::Number(operand2)) => operand1.powf(operand2),
            _ => return Err(Error::Parser(rule_name)),
        };
        Ok(Expr::Number(res))
    }

    pub(crate) fn parse_eq(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let operand1 = Self::get_formula(&mut args, &rule_name)?;
        let operand2 = Self::get_formula(&mut args, &rule_name)?;

        let res = match (operand1, operand2) {
            (Expr::Number(operand1), Expr::Number(operand2)) => operand1 == operand2,
            (Expr::String(operand1), Expr::String(operand2)) => operand1 == operand2,
            (Expr::Time(operand1), Expr::Time(operand2)) => operand1 == operand2,
            (Expr::Datetime(operand1), Expr::Datetime(operand2)) => operand1 == operand2,
            (Expr::Date(operand1), Expr::Date(operand2)) => operand1 == operand2,
            (Expr::Bool(operand1), Expr::Bool(operand2)) => operand1 == operand2,
            _ => return Err(Error::Parser(rule_name)),
        };
        Ok(Expr::Bool(res))
    }

    pub(crate) fn parse_ne(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let operand1 = Self::get_formula(&mut args, &rule_name)?;
        let operand2 = Self::get_formula(&mut args, &rule_name)?;

        let res = match (operand1, operand2) {
            (Expr::Number(operand1), Expr::Number(operand2)) => (operand1 - operand2).abs() > 0.000_000_1,
            (Expr::String(operand1), Expr::String(operand2)) => operand1 != operand2,
            (Expr::Time(operand1), Expr::Time(operand2)) => operand1 != operand2,
            (Expr::Datetime(operand1), Expr::Datetime(operand2)) => operand1 != operand2,
            (Expr::Date(operand1), Expr::Date(operand2)) => operand1 != operand2,
            (Expr::Bool(operand1), Expr::Bool(operand2)) => operand1 != operand2,
            _ => return Err(Error::Parser(rule_name)),
        };
        Ok(Expr::Bool(res))
    }

    pub(crate) fn parse_gt(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let operand1 = Self::get_formula(&mut args, &rule_name)?;
        let operand2 = Self::get_formula(&mut args, &rule_name)?;

        let res = match (operand1, operand2) {
            (Expr::Number(operand1), Expr::Number(operand2)) => operand1 > operand2,
            (Expr::String(operand1), Expr::String(operand2)) => operand1 > operand2,
            (Expr::Time(operand1), Expr::Time(operand2)) => operand1 > operand2,
            (Expr::Datetime(operand1), Expr::Datetime(operand2)) => operand1 > operand2,
            (Expr::Date(operand1), Expr::Date(operand2)) => operand1 > operand2,
            _ => return Err(Error::Parser(rule_name)),
        };
        Ok(Expr::Bool(res))
    }

    pub(crate) fn parse_lt(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let operand1 = Self::get_formula(&mut args, &rule_name)?;
        let operand2 = Self::get_formula(&mut args, &rule_name)?;

        let res = match (operand1, operand2) {
            (Expr::Number(operand1), Expr::Number(operand2)) => operand1 < operand2,
            (Expr::String(operand1), Expr::String(operand2)) => operand1 < operand2,
            (Expr::Time(operand1), Expr::Time(operand2)) => operand1 < operand2,
            (Expr::Datetime(operand1), Expr::Datetime(operand2)) => operand1 < operand2,
            (Expr::Date(operand1), Expr::Date(operand2)) => operand1 < operand2,
            _ => return Err(Error::Parser(rule_name)),
        };
        Ok(Expr::Bool(res))
    }

    pub(crate) fn parse_gte(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let operand1 = Self::get_formula(&mut args, &rule_name)?;
        let operand2 = Self::get_formula(&mut args, &rule_name)?;

        let res = match (operand1, operand2) {
            (Expr::Number(operand1), Expr::Number(operand2)) => operand1 >= operand2,
            (Expr::String(operand1), Expr::String(operand2)) => operand1 >= operand2,
            (Expr::Time(operand1), Expr::Time(operand2)) => operand1 >= operand2,
            (Expr::Datetime(operand1), Expr::Datetime(operand2)) => operand1 >= operand2,
            (Expr::Date(operand1), Expr::Date(operand2)) => operand1 >= operand2,
            _ => return Err(Error::Parser(rule_name)),
        };
        Ok(Expr::Bool(res))
    }

    pub(crate) fn parse_lte(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let operand1 = Self::get_formula(&mut args, &rule_name)?;
        let operand2 = Self::get_formula(&mut args, &rule_name)?;

        let res = match (operand1, operand2) {
            (Expr::Number(operand1), Expr::Number(operand2)) => operand1 <= operand2,
            (Expr::String(operand1), Expr::String(operand2)) => operand1 <= operand2,
            (Expr::Time(operand1), Expr::Time(operand2)) => operand1 <= operand2,
            (Expr::Datetime(operand1), Expr::Datetime(operand2)) => operand1 <= operand2,
            (Expr::Date(operand1), Expr::Date(operand2)) => operand1 <= operand2,
            _ => return Err(Error::Parser(rule_name)),
        };
        Ok(Expr::Bool(res))
    }

    pub(crate) fn parse_percent(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let operand = Self::get_formula(&mut args, &rule_name)?;

        let res = match operand {
            Expr::Number(operand) => operand / 100.0,
            _ => return Err(Error::Parser(rule_name)),
        };
        Ok(Expr::Number(res))
    }

    pub(crate) fn parse_negate(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let operand = Self::get_formula(&mut args, &rule_name)?;

        let res = match operand {
            Expr::Number(operand) => -operand,
            _ => return Err(Error::Parser(rule_name)),
        };
        Ok(Expr::Number(res))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Formula;

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_parse_web_types() {
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

        let formula = Formula::new("=F.MUL(5,6)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(30.0));

        let formula = Formula::new("=F.POW(5,2)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(25.0));

        let formula = Formula::new("=F.EQ(5,2)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Bool(false));

        let formula = Formula::new("=F.NE('a','a')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Bool(false));

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
    }
}
