use super::{Evaluator, Result};
use crate::{Expr, ExprError, Rule};
use pest::iterators::{Pair, Pairs};

/// Infinite and NaN results are `#NUM!` errors
pub(crate) fn number(value: f64) -> Expr {
    if value.is_finite() {
        Expr::Number(value)
    } else {
        Expr::Error(ExprError::Number)
    }
}

/// Like Excel, an empty cell is 0 where a number is needed
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn as_number(value: Expr) -> Result<f64> {
    match value {
        Expr::Number(number) => Ok(number),
        Expr::Null => Ok(0.0),
        _ => Err(ExprError::Value),
    }
}

/// Like Excel, an empty cell is an empty text where a text is needed
pub(crate) fn as_text(value: Expr) -> Result<String> {
    match value {
        Expr::String(text) => Ok(text),
        Expr::Null => Ok(String::new()),
        _ => Err(ExprError::Value),
    }
}

/// Like Excel, an empty cell is FALSE where a boolean is needed
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn as_bool(value: Expr) -> Result<bool> {
    match value {
        Expr::Bool(bool) => Ok(bool),
        Expr::Null => Ok(false),
        _ => Err(ExprError::Value),
    }
}

/// A number as text like Excel: at most 15 significant digits, e.g. `0.1 + 0.2` is "0.3",
/// and the scientific notation for very large and very small numbers, e.g. "1E+15"
pub(crate) fn number_to_text(number: f64) -> String {
    let scientific = format!("{number:.14e}");
    let Some((mantissa, exponent)) = scientific.split_once('e') else {
        return number.to_string();
    };
    let exponent = exponent.parse::<i32>().unwrap_or(0);

    if number == 0.0 {
        // also -0
        "0".to_string()
    } else if (-9..15).contains(&exponent) {
        // the shortest text of the rounded number
        scientific.parse::<f64>().unwrap_or(number).to_string()
    } else {
        let mantissa = mantissa.trim_end_matches('0').trim_end_matches('.');
        let sign = if exponent < 0 { '-' } else { '+' };
        format!("{mantissa}E{sign}{:02}", exponent.abs())
    }
}

/// A value of the arguments of functions that take ranges, like `SUM`
pub(crate) enum ArgValue {
    /// An argument's own value, it must have the type that the function needs
    Single(Expr),
    /// A value of a reference or an array, functions skip it if it has another type, like Excel
    InRange(Expr),
}

impl Evaluator<'_> {
    /// The values of the arguments, with the values of references and arrays flattened.
    /// Like Excel, the errors in them propagate.
    pub(crate) fn range_args(&self, args: Pairs<Rule>) -> Result<Vec<ArgValue>> {
        let mut values = Vec::new();
        for arg in args {
            let is_reference = arg.as_rule() == Rule::reference;
            match self.parse_pair(arg)? {
                Expr::Array(array) => Self::flatten(array, &mut values)?,
                value if is_reference => values.push(ArgValue::InRange(value)),
                value => values.push(ArgValue::Single(value)),
            }
        }
        Ok(values)
    }

    fn flatten(array: Vec<Expr>, values: &mut Vec<ArgValue>) -> Result<()> {
        for value in array {
            match value {
                Expr::Array(array) => Self::flatten(array, values)?,
                Expr::Error(error) => return Err(error),
                value => values.push(ArgValue::InRange(value)),
            }
        }
        Ok(())
    }

    pub(crate) fn get_formula(&self, args: &mut Pairs<Rule>) -> Result<Expr> {
        args.next().map_or(Err(ExprError::Value), |pair| self.parse_pair(pair))
    }

    pub(crate) fn get_opt_formula_with_default(&self, args: &mut Pairs<Rule>, default: Expr) -> Result<Expr> {
        args.next().map_or(Ok(default), |pair| self.parse_pair(pair))
    }

    pub(crate) fn get_number(&self, args: &mut Pairs<Rule>) -> Result<f64> {
        as_number(self.get_formula(args)?)
    }

    pub(crate) fn get_text(&self, args: &mut Pairs<Rule>) -> Result<String> {
        as_text(self.get_formula(args)?)
    }

    pub(crate) fn get_bool(&self, args: &mut Pairs<Rule>) -> Result<bool> {
        as_bool(self.get_formula(args)?)
    }

    /// A missing optional argument is `default`
    pub(crate) fn get_opt_number(&self, args: &mut Pairs<Rule>, default: f64) -> Result<f64> {
        as_number(self.get_opt_formula_with_default(args, Expr::Number(default))?)
    }

    /// A missing optional argument is `default`
    pub(crate) fn get_opt_bool(&self, args: &mut Pairs<Rule>, default: bool) -> Result<bool> {
        as_bool(self.get_opt_formula_with_default(args, Expr::Bool(default))?)
    }

    /// Evaluates a function that takes one number
    pub(crate) fn parse_unary_number(&self, pair: Pair<Rule>, f: impl FnOnce(f64) -> f64) -> Result<Expr> {
        let mut args = pair.into_inner();
        let operand = self.get_number(&mut args)?;

        Ok(number(f(operand)))
    }

    /// Evaluates a function that takes two numbers
    pub(crate) fn parse_binary_number(&self, pair: Pair<Rule>, f: impl FnOnce(f64, f64) -> Expr) -> Result<Expr> {
        let mut args = pair.into_inner();
        let operand1 = self.get_number(&mut args)?;
        let operand2 = self.get_number(&mut args)?;

        Ok(f(operand1, operand2))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_to_text() {
        assert_eq!(number_to_text(0.0), "0");
        assert_eq!(number_to_text(-0.0), "0");
        assert_eq!(number_to_text(100.0), "100");
        assert_eq!(number_to_text(-1.5), "-1.5");
        assert_eq!(number_to_text(0.1 + 0.2), "0.3");
        assert_eq!(number_to_text(1.0 / 3.0), "0.333333333333333");
        assert_eq!(number_to_text(0.000_000_001), "0.000000001");
        assert_eq!(number_to_text(123_456_789_012_345.0), "123456789012345");
        assert_eq!(number_to_text(1_234_567_890_123_456.0), "1.23456789012346E+15");
        assert_eq!(number_to_text(1e15), "1E+15");
        assert_eq!(number_to_text(-2.5e20), "-2.5E+20");
        assert_eq!(number_to_text(1e-10), "1E-10");
        assert_eq!(number_to_text(1.5e-300), "1.5E-300");
    }
}
