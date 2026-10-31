use super::{
    Result,
    helper::{number, parser_error},
};
use crate::{Expr, ExprError, Formula, Rule};
use pest::iterators::Pair;
use std::f64::consts::PI;

impl Formula<'_> {
    pub(crate) fn parse_abs(pair: Pair<Rule>) -> Result<Expr> {
        Self::parse_unary_number(pair, f64::abs)
    }

    pub(crate) fn parse_acos(pair: Pair<Rule>) -> Result<Expr> {
        Self::parse_unary_number(pair, f64::acos)
    }

    pub(crate) fn parse_acosh(pair: Pair<Rule>) -> Result<Expr> {
        Self::parse_unary_number(pair, f64::acosh)
    }

    pub(crate) fn parse_cos(pair: Pair<Rule>) -> Result<Expr> {
        Self::parse_unary_number(pair, f64::cos)
    }

    pub(crate) fn parse_cosh(pair: Pair<Rule>) -> Result<Expr> {
        Self::parse_unary_number(pair, f64::cosh)
    }

    pub(crate) fn parse_asin(pair: Pair<Rule>) -> Result<Expr> {
        Self::parse_unary_number(pair, f64::asin)
    }

    pub(crate) fn parse_asinh(pair: Pair<Rule>) -> Result<Expr> {
        Self::parse_unary_number(pair, f64::asinh)
    }

    pub(crate) fn parse_sin(pair: Pair<Rule>) -> Result<Expr> {
        Self::parse_unary_number(pair, f64::sin)
    }

    pub(crate) fn parse_sinh(pair: Pair<Rule>) -> Result<Expr> {
        Self::parse_unary_number(pair, f64::sinh)
    }

    pub(crate) fn parse_atan(pair: Pair<Rule>) -> Result<Expr> {
        Self::parse_unary_number(pair, f64::atan)
    }

    pub(crate) fn parse_atan2(pair: Pair<Rule>) -> Result<Expr> {
        Self::parse_binary_number(pair, |operand1, operand2| number(operand1.atan2(operand2)))
    }

    pub(crate) fn parse_atanh(pair: Pair<Rule>) -> Result<Expr> {
        Self::parse_unary_number(pair, f64::atanh)
    }

    pub(crate) fn parse_tan(pair: Pair<Rule>) -> Result<Expr> {
        Self::parse_unary_number(pair, f64::tan)
    }

    pub(crate) fn parse_tanh(pair: Pair<Rule>) -> Result<Expr> {
        Self::parse_unary_number(pair, f64::tanh)
    }

    #[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
    pub(crate) fn parse_pi(_pair: Pair<Rule>) -> Result<Expr> {
        Ok(Expr::Number(PI))
    }

    /// Like Excel, the result has the sign of the divisor
    pub(crate) fn parse_mod(pair: Pair<Rule>) -> Result<Expr> {
        Self::parse_binary_number(pair, |operand1, operand2| {
            if operand2 == 0.0 {
                Expr::Error(ExprError::DivisionByZero)
            } else {
                number(operand1 - operand2 * (operand1 / operand2).floor())
            }
        })
    }

    pub(crate) fn parse_log(pair: Pair<Rule>) -> Result<Expr> {
        Self::parse_binary_number(pair, |operand1, operand2| number(operand1.log(operand2)))
    }

    pub(crate) fn parse_log10(pair: Pair<Rule>) -> Result<Expr> {
        Self::parse_unary_number(pair, f64::log10)
    }

    pub(crate) fn parse_ln(pair: Pair<Rule>) -> Result<Expr> {
        Self::parse_unary_number(pair, f64::ln)
    }

    pub(crate) fn parse_sqrt(pair: Pair<Rule>) -> Result<Expr> {
        Self::parse_unary_number(pair, f64::sqrt)
    }

    pub(crate) fn parse_sqrtpi(pair: Pair<Rule>) -> Result<Expr> {
        Self::parse_unary_number(pair, |operand| (operand * PI).sqrt())
    }

    #[allow(clippy::needless_pass_by_value)]
    pub(crate) fn parse_rand(pair: Pair<Rule>) -> Result<Expr> {
        // 53 random bits scaled into [0, 1)
        let bits = getrandom::u64().map_err(|_| parser_error(pair.as_rule()))?;
        Ok(Expr::Number((bits >> 11) as f64 / (1u64 << 53) as f64))
    }

    pub(crate) fn parse_sign(pair: Pair<Rule>) -> Result<Expr> {
        Self::parse_unary_number(pair, |operand| if operand == 0.0 { 0.0 } else { operand.signum() })
    }

    pub(crate) fn parse_exp(pair: Pair<Rule>) -> Result<Expr> {
        Self::parse_unary_number(pair, f64::exp)
    }

    pub(crate) fn parse_sum(pair: Pair<Rule>) -> Result<Expr> {
        let rule = pair.as_rule();
        pair.into_inner()
            .try_fold(0.0, |sum, arg| match Self::parse_pair(arg)? {
                Expr::Number(operand) => Ok(sum + operand),
                _ => Err(parser_error(rule)),
            })
            .map(number)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ExprError, Formula};

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_parse_math_types() {
        let formula = Formula::new("=ABS(-1)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(1.0));

        let formula = Formula::new("=SIGN(LOG10(SQRT(SUM(1,2,3,3))))").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(1.0));

        let formula = Formula::new("=MOD(-3, 2)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(1.0));

        let formula = Formula::new("=MOD(3, -2)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(-1.0));

        let formula = Formula::new("=MOD(3, 0)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Error(ExprError::DivisionByZero));

        let formula = Formula::new("=SQRT(-1)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Error(ExprError::Number));

        let formula = Formula::new("=LN(0)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Error(ExprError::Number));

        let formula = Formula::new("=SUM(1, 'a')").unwrap();
        assert!(formula.parse().is_err());

        let formula = Formula::new("=RAND()").unwrap();
        let value = formula.parse().unwrap();
        assert!(matches!(value, Expr::Number(n) if (0.0..1.0).contains(&n)));

        let formula = Formula::new("=F.POW(10, 400)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Error(ExprError::Number));

        let formula = Formula::new("=SUM(1e308, 1e308)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Error(ExprError::Number));

        let formula = Formula::new("=MOD(5.5, 2)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(1.5));

        let formula = Formula::new("=SIGN(-0.5)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(-1.0));

        let formula = Formula::new("=SIGN(0)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(0.0));

        let formula = Formula::new("=SQRTPI(-1)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Error(ExprError::Number));

        let formula = Formula::new("=LOG(8, 2)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(3.0));
    }
}
