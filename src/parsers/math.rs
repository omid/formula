use crate::{error::Error, Expr, Formula, Result, Rule};
use pest::iterators::Pair;
use rand::Rng;
use std::f64::consts::PI;

impl Formula<'_> {
    pub(crate) fn parse_abs(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let operand = Self::get_formula(&mut args, &rule_name)?;

        let res = match operand {
            Expr::Number(operand) => operand.abs(),
            _ => return Err(Error::Parser(rule_name)),
        };
        Ok(Expr::Number(res))
    }

    pub(crate) fn parse_acos(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let operand = Self::get_formula(&mut args, &rule_name)?;

        let res = match operand {
            Expr::Number(operand) => operand.acos(),
            _ => return Err(Error::Parser(rule_name)),
        };
        Ok(Expr::Number(res))
    }

    pub(crate) fn parse_acosh(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let operand = Self::get_formula(&mut args, &rule_name)?;

        let res = match operand {
            Expr::Number(operand) => operand.acosh(),
            _ => return Err(Error::Parser(rule_name)),
        };
        Ok(Expr::Number(res))
    }

    pub(crate) fn parse_cos(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let operand = Self::get_formula(&mut args, &rule_name)?;

        let res = match operand {
            Expr::Number(operand) => operand.cos(),
            _ => return Err(Error::Parser(rule_name)),
        };
        Ok(Expr::Number(res))
    }

    pub(crate) fn parse_cosh(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let operand = Self::get_formula(&mut args, &rule_name)?;

        let res = match operand {
            Expr::Number(operand) => operand.cosh(),
            _ => return Err(Error::Parser(rule_name)),
        };
        Ok(Expr::Number(res))
    }

    pub(crate) fn parse_asin(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let operand = Self::get_formula(&mut args, &rule_name)?;

        let res = match operand {
            Expr::Number(operand) => operand.asin(),
            _ => return Err(Error::Parser(rule_name)),
        };
        Ok(Expr::Number(res))
    }

    pub(crate) fn parse_asinh(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let operand = Self::get_formula(&mut args, &rule_name)?;

        let res = match operand {
            Expr::Number(operand) => operand.asinh(),
            _ => return Err(Error::Parser(rule_name)),
        };
        Ok(Expr::Number(res))
    }

    pub(crate) fn parse_sin(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let operand = Self::get_formula(&mut args, &rule_name)?;

        let res = match operand {
            Expr::Number(operand) => operand.sin(),
            _ => return Err(Error::Parser(rule_name)),
        };
        Ok(Expr::Number(res))
    }

    pub(crate) fn parse_sinh(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let operand = Self::get_formula(&mut args, &rule_name)?;

        let res = match operand {
            Expr::Number(operand) => operand.sinh(),
            _ => return Err(Error::Parser(rule_name)),
        };
        Ok(Expr::Number(res))
    }

    pub(crate) fn parse_atan(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let operand = Self::get_formula(&mut args, &rule_name)?;

        let res = match operand {
            Expr::Number(operand) => operand.atan(),
            _ => return Err(Error::Parser(rule_name)),
        };
        Ok(Expr::Number(res))
    }

    pub(crate) fn parse_atan2(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let operand1 = Self::get_formula(&mut args, &rule_name)?;
        let operand2 = Self::get_formula(&mut args, &rule_name)?;

        let res = match (operand1, operand2) {
            (Expr::Number(operand1), Expr::Number(operand2)) => operand1.atan2(operand2),
            _ => return Err(Error::Parser(rule_name)),
        };
        Ok(Expr::Number(res))
    }

    pub(crate) fn parse_atanh(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let operand = Self::get_formula(&mut args, &rule_name)?;

        let res = match operand {
            Expr::Number(operand) => operand.atanh(),
            _ => return Err(Error::Parser(rule_name)),
        };
        Ok(Expr::Number(res))
    }

    pub(crate) fn parse_tan(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let operand = Self::get_formula(&mut args, &rule_name)?;

        let res = match operand {
            Expr::Number(operand) => operand.tan(),
            _ => return Err(Error::Parser(rule_name)),
        };
        Ok(Expr::Number(res))
    }

    pub(crate) fn parse_tanh(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let operand = Self::get_formula(&mut args, &rule_name)?;

        let res = match operand {
            Expr::Number(operand) => operand.tanh(),
            _ => return Err(Error::Parser(rule_name)),
        };
        Ok(Expr::Number(res))
    }

    #[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
    pub(crate) fn parse_pi(_pair: Pair<Rule>) -> Result<Expr> {
        Ok(Expr::Number(PI))
    }

    pub(crate) fn parse_mod(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let operand1 = Self::get_formula(&mut args, &rule_name)?;
        let operand2 = Self::get_formula(&mut args, &rule_name)?;

        let res = match (operand1, operand2) {
            (Expr::Number(operand1), Expr::Number(operand2)) => operand1 % operand2,
            _ => return Err(Error::Parser(rule_name)),
        };
        Ok(Expr::Number(res))
    }

    pub(crate) fn parse_log(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let operand1 = Self::get_formula(&mut args, &rule_name)?;
        let operand2 = Self::get_formula(&mut args, &rule_name)?;

        let res = match (operand1, operand2) {
            (Expr::Number(operand1), Expr::Number(operand2)) => operand1.log(operand2),
            _ => return Err(Error::Parser(rule_name)),
        };
        Ok(Expr::Number(res))
    }

    pub(crate) fn parse_log10(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let operand = Self::get_formula(&mut args, &rule_name)?;

        let res = match operand {
            Expr::Number(operand) => operand.log10(),
            _ => return Err(Error::Parser(rule_name)),
        };
        Ok(Expr::Number(res))
    }

    pub(crate) fn parse_ln(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let operand = Self::get_formula(&mut args, &rule_name)?;

        let res = match operand {
            Expr::Number(operand) => operand.ln(),
            _ => return Err(Error::Parser(rule_name)),
        };
        Ok(Expr::Number(res))
    }

    pub(crate) fn parse_sqrt(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let operand = Self::get_formula(&mut args, &rule_name)?;

        let res = match operand {
            Expr::Number(operand) => {
                if operand < 0.0 {
                    Expr::Null
                } else {
                    Expr::Number(operand.sqrt())
                }
            }
            _ => return Err(Error::Parser(rule_name)),
        };
        Ok(res)
    }

    pub(crate) fn parse_sqrtpi(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let operand = Self::get_formula(&mut args, &rule_name)?;

        let res = match operand {
            Expr::Number(operand) => {
                if operand < 0.0 {
                    Expr::Null
                } else {
                    Expr::Number((operand * PI).sqrt())
                }
            }
            _ => return Err(Error::Parser(rule_name)),
        };
        Ok(res)
    }

    #[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
    pub(crate) fn parse_rand(_pair: Pair<Rule>) -> Result<Expr> {
        let mut rng = rand::thread_rng();
        Ok(Expr::Number(rng.gen_range(0.0..1.0)))
    }

    pub(crate) fn parse_sign(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let operand = Self::get_formula(&mut args, &rule_name)?;

        let res = match operand {
            Expr::Number(operand) => {
                if operand == 0.0 {
                    0.0
                } else {
                    operand.signum()
                }
            }
            _ => return Err(Error::Parser(rule_name)),
        };
        Ok(Expr::Number(res))
    }

    pub(crate) fn parse_exp(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let operand = Self::get_formula(&mut args, &rule_name)?;

        let res = match operand {
            Expr::Number(operand) => operand.exp(),
            _ => return Err(Error::Parser(rule_name)),
        };
        Ok(Expr::Number(res))
    }

    pub(crate) fn parse_sum(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let operand = Self::get_formula(&mut args, &rule_name)?;
        let operands = args.map(Self::parse_pair).collect::<Result<Vec<_>>>()?;

        let res = match operand {
            Expr::Number(mut operand) => {
                for o in operands {
                    match o {
                        Expr::Number(o) => operand += o,
                        _ => return Err(Error::Parser(rule_name)),
                    }
                }
                operand
            }
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
    fn test_parse_math_types() {
        let formula = Formula::new("=ABS(-1)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(1.0));

        let formula = Formula::new("=SIGN(LOG10(SQRT(SUM(1,2,3,3))))").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(1.0));
    }
}
