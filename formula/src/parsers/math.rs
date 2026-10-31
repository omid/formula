use super::{
    Evaluator, Result,
    helper::{ArgValue, as_number, number},
};
use crate::{Expr, ExprError, Rule};
use pest::iterators::Pair;
use std::f64::consts::PI;

impl Evaluator<'_> {
    pub(crate) fn parse_abs(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_unary_number(pair, f64::abs)
    }

    pub(crate) fn parse_acos(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_unary_number(pair, f64::acos)
    }

    pub(crate) fn parse_acosh(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_unary_number(pair, f64::acosh)
    }

    pub(crate) fn parse_cos(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_unary_number(pair, f64::cos)
    }

    pub(crate) fn parse_cosh(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_unary_number(pair, f64::cosh)
    }

    pub(crate) fn parse_asin(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_unary_number(pair, f64::asin)
    }

    pub(crate) fn parse_asinh(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_unary_number(pair, f64::asinh)
    }

    pub(crate) fn parse_sin(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_unary_number(pair, f64::sin)
    }

    pub(crate) fn parse_sinh(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_unary_number(pair, f64::sinh)
    }

    pub(crate) fn parse_atan(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_unary_number(pair, f64::atan)
    }

    /// Like Excel, the arguments are the x and then the y coordinate, and the angle of `(0, 0)` is `#DIV/0!`
    pub(crate) fn parse_atan2(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_binary_number(pair, |x, y| {
            if x == 0.0 && y == 0.0 {
                Expr::Error(ExprError::DivisionByZero)
            } else {
                number(y.atan2(x))
            }
        })
    }

    pub(crate) fn parse_atanh(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_unary_number(pair, f64::atanh)
    }

    pub(crate) fn parse_tan(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_unary_number(pair, f64::tan)
    }

    pub(crate) fn parse_tanh(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_unary_number(pair, f64::tanh)
    }

    #[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps, clippy::unused_self)]
    pub(crate) fn parse_pi(&self, _pair: Pair<Rule>) -> Result<Expr> {
        Ok(Expr::Number(PI))
    }

    /// Like Excel, the result has the sign of the divisor
    pub(crate) fn parse_mod(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_binary_number(pair, |operand1, operand2| {
            if operand2 == 0.0 {
                Expr::Error(ExprError::DivisionByZero)
            } else {
                number(operand1 - operand2 * (operand1 / operand2).floor())
            }
        })
    }

    pub(crate) fn parse_log(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_binary_number(pair, |operand1, operand2| number(operand1.log(operand2)))
    }

    pub(crate) fn parse_log10(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_unary_number(pair, f64::log10)
    }

    pub(crate) fn parse_ln(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_unary_number(pair, f64::ln)
    }

    pub(crate) fn parse_sqrt(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_unary_number(pair, f64::sqrt)
    }

    pub(crate) fn parse_sqrtpi(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_unary_number(pair, |operand| (operand * PI).sqrt())
    }

    #[allow(clippy::needless_pass_by_value, clippy::unused_self)]
    pub(crate) fn parse_rand(&self, _pair: Pair<Rule>) -> Result<Expr> {
        // 53 random bits scaled into [0, 1)
        let bits = getrandom::u64().map_err(|_| ExprError::Value)?;
        Ok(Expr::Number((bits >> 11) as f64 / (1u64 << 53) as f64))
    }

    pub(crate) fn parse_sign(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_unary_number(pair, |operand| if operand == 0.0 { 0.0 } else { operand.signum() })
    }

    pub(crate) fn parse_exp(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_unary_number(pair, f64::exp)
    }

    /// Like Excel, the texts, booleans and empty cells of references and arrays don't count
    pub(crate) fn parse_sum(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.range_args(pair.into_inner())?
            .into_iter()
            .try_fold(0.0, |sum, value| match value {
                ArgValue::Single(value) => Ok(sum + as_number(value)?),
                ArgValue::InRange(Expr::Number(number)) => Ok(sum + number),
                ArgValue::InRange(_) => Ok(sum),
            })
            .map(number)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ExprError, Formula};

    #[test]
    fn test_parse_trigonometric_types() {
        type Function = fn(f64) -> f64;
        let functions: [(&str, Function, f64); 13] = [
            ("SIN", f64::sin, 0.5),
            ("COS", f64::cos, 0.5),
            ("TAN", f64::tan, 0.5),
            ("ASIN", f64::asin, 0.5),
            ("ACOS", f64::acos, 0.5),
            ("ATAN", f64::atan, 0.5),
            ("SINH", f64::sinh, 0.5),
            ("COSH", f64::cosh, 0.5),
            ("TANH", f64::tanh, 0.5),
            ("ASINH", f64::asinh, 0.5),
            ("ACOSH", f64::acosh, 1.5),
            ("ATANH", f64::atanh, 0.5),
            ("EXP", f64::exp, 0.5),
        ];
        for (name, f, x) in functions {
            let formula = format!("={name}({x})");
            let value = Formula::new(&formula).unwrap().parse().unwrap();
            assert_eq!(value, Expr::Number(f(x)), "{name}");
        }

        let formula = Formula::new("=PI()").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(PI));

        let formula = Formula::new("=SIN(F.DIV(PI(), 2))").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(1.0));

        // outside of their domain
        for formula in ["=ASIN(2)", "=ACOS(-2)", "=ACOSH(0.5)", "=ATANH(1)", "=EXP(1000)"] {
            let value = Formula::new(formula).unwrap().parse().unwrap();
            assert_eq!(value, Expr::Error(ExprError::Number), "{formula}");
        }

        let formula = Formula::new("=COS('a')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Error(ExprError::Value));
    }

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
        assert_eq!(formula.parse().unwrap(), Expr::Error(ExprError::Value));

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

        let formula = Formula::new("=ATAN2(1, 0)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(0.0));

        let formula = Formula::new("=ATAN2(0, 1)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(std::f64::consts::FRAC_PI_2));

        let formula = Formula::new("=ATAN2(-1, -1)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(-3.0 * std::f64::consts::FRAC_PI_4));

        let formula = Formula::new("=ATAN2(-1, 0)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(std::f64::consts::PI));

        let formula = Formula::new("=ATAN2(0, 0)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Error(ExprError::DivisionByZero));
    }
}
