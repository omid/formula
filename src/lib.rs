/*!
[![Crates.io](https://img.shields.io/crates/v/formula.svg)](https://crates.io/crates/formula)
[![Workflow Status](https://github.com/omid/formula/workflows/ci/badge.svg)](https://github.com/omid/formula/actions?query=workflow%3A%22ci%22)

<h1 align="center"><em>Formula</em></h1>

<h3 align="center">
  A parser and evaluator of spreadsheet-like formulas
</h3>

Formula is in its early stages and is not ready for production use.

So far we have the following features:

- 18 date time functions
- 26 text functions
- 7 logical functions
- 2 web functions
- plus all arithmetic and comparison operators

## Installation and usage

Add this library to your project with `cargo add formula` or add `formula = "*"` to your `Cargo.toml` file.

Use it similar to the following code:

```rust
use formula::{Formula, Expr, error::Error};
use anyhow::Result;

fn main() -> Result<()> {
    let formula = Formula::new("=UPPER(TRIM('   Hello '))")?;
    let value = formula.parse()?;
    assert_eq!(value, Expr::String("HELLO".to_string()));
    Ok(())
}
```

## What we do not support, yet:

- Support of functions are so limited at the moment, but we would like to add more of them, like Excel functions, Google Sheets functions, and so on
- At the moment, we don't support table data, so you need to pass values to the formula as arguments by yourself
- We do not support simple formulas like `1+1` or as argument like `AND(1>3, 1<3)` or `SUM(2-1, 2)`. Instead, you can use our `F.` functions like `AND(F.GT(1, 3), F.LT(1, 3))` or `SUM(F.SUB(2, 1), 2)`
- We still do not support parentheses to change the order of operations, but you can use our `F.` functions. So for example instead of `2*(1+1)`, you should use `F.MUL(2, F.ADD(1, 1))`

## Contributing

We would love to have your contribution! Please read our [contributing guidelines](CONTRIBUTING.md) to get started.

## License

This project is licensed under the MIT license. See the [LICENSE](LICENSE.md) file for more info.
*/

pub mod error;
mod parsers;

use anyhow::Result;
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

#[derive(Parser, Debug)]
#[grammar = "formula.pest"]
struct FormulaInner;

#[derive(Debug)]
/// `Formula`, is the main struct and entry point of this library.
pub struct Formula<'a> {
    pairs: Pair<'a, Rule>,
}

#[derive(Debug, PartialEq)]
/// `Expr` is the result of parsing a formula.
///
/// There is a difference between Excel and this library here.
/// We don't have a `#N/A`, `#VALUE!`, `#DIV/0!`, `#NUM!`, `#NULL!` error types, instead it will return `Expr::Null`.
pub enum Expr {
    Date(NaiveDate),
    Datetime(DateTime<Utc>),
    Time(NaiveTime),
    Number(f64),
    String(String),
    Bool(bool),
    Null,
}

impl<'a> Formula<'a> {
    /// To interpret and prepare a new formula, you need to call the `new` method, like the code below:
    ///
    /// ```rust
    /// use formula::{Formula, Expr, error::Error};
    /// use anyhow::Result;
    ///
    /// fn main() -> Result<()> {
    ///     let formula = Formula::new("=UPPER(TRIM('   Hello '))")?;
    ///     let value = formula.parse()?;
    ///     assert_eq!(value, Expr::String("HELLO".to_string()));
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Errors
    ///
    /// Will return `Err` if the formula is not valid.
    pub fn new(formula: &'a str) -> Result<Self> {
        let pairs = FormulaInner::parse(Rule::root, formula)?
            .next()
            .ok_or_else(|| error::Error::Parser("No formula found".to_string()))?;
        Ok(Self { pairs })
    }

    /// Parse a formula and return the result
    ///
    /// # Errors
    ///
    /// Will return `Err` if the formula is not valid or the functions are not implemented.
    pub fn parse(self) -> Result<Expr> {
        Self::parse_pair(self.pairs)
    }

    #[allow(clippy::too_many_lines)]
    fn parse_pair(pair: Pair<Rule>) -> Result<Expr> {
        #[allow(clippy::match_same_arms)]
        let res = match pair.as_rule() {
            // Operators
            Rule::add => Self::parse_add(pair)?,
            Rule::sub => Self::parse_sub(pair)?,
            Rule::mul => Self::parse_mul(pair)?,
            Rule::div => Self::parse_div(pair)?,
            Rule::pow => Self::parse_pow(pair)?,
            Rule::eq => Self::parse_eq(pair)?,
            Rule::ne => Self::parse_ne(pair)?,
            Rule::gt => Self::parse_gt(pair)?,
            Rule::lt => Self::parse_lt(pair)?,
            Rule::gte => Self::parse_gte(pair)?,
            Rule::lte => Self::parse_lte(pair)?,
            Rule::percent => Self::parse_percent(pair)?,
            Rule::negate => Self::parse_negate(pair)?,

            // Date and time functions
            Rule::date => Self::parse_date(pair)?,
            Rule::time => Self::parse_time(pair)?,
            Rule::year => Self::parse_year(pair)?,
            Rule::month => Self::parse_month(pair)?,
            Rule::day => Self::parse_day(pair)?,
            Rule::days => Self::parse_days(pair)?,
            Rule::edate => Self::parse_edate(pair)?,
            Rule::eomonth => Self::parse_eomonth(pair)?,
            Rule::datevalue => Self::parse_datevalue(pair)?,
            Rule::timevalue => Self::parse_timevalue(pair)?,
            Rule::hour => Self::parse_hour(pair)?,
            Rule::minute => Self::parse_minute(pair)?,
            Rule::second => Self::parse_second(pair)?,
            Rule::isoweeknum => Self::parse_isoweeknum(pair)?,
            Rule::weeknum => Self::parse_weeknum(pair)?,
            Rule::weekday => Self::parse_weekday(pair)?,
            Rule::now => Self::parse_now(),
            Rule::today => Self::parse_today(),
            // TODO remaining date and time functions
            Rule::networkdays
            | Rule::networkdaysintl
            | Rule::workdays
            | Rule::workdaysintl
            | Rule::yearfrac
            | Rule::days360
            | Rule::datediff => return Err(error::Error::NotImplemented(format!("{:?}", pair.as_rule())).into()),

            // Text functions
            Rule::left => Self::parse_left(pair)?,
            Rule::leftb => Self::parse_leftb(pair)?,
            Rule::right => Self::parse_right(pair)?,
            Rule::rightb => Self::parse_rightb(pair)?,
            Rule::mid => Self::parse_mid(pair)?,
            Rule::midb => Self::parse_midb(pair)?,
            Rule::char | Rule::unichar => Self::parse_char(pair)?,
            Rule::code | Rule::unicode => Self::parse_code(pair)?,
            Rule::concat | Rule::concatenate => Self::parse_concat(pair)?,
            Rule::exact => Self::parse_exact(pair)?,
            Rule::find => Self::parse_find(pair)?,
            Rule::findb => Self::parse_findb(pair)?,
            Rule::search => Self::parse_search(pair)?,
            Rule::searchb => Self::parse_searchb(pair)?,
            Rule::fixed => Self::parse_fixed(pair)?,
            Rule::len => Self::parse_len(pair)?,
            Rule::lenb => Self::parse_lenb(pair)?,
            Rule::lower => Self::parse_lower(pair)?,
            Rule::upper => Self::parse_upper(pair)?,
            Rule::rept => Self::parse_rept(pair)?,
            Rule::replace => Self::parse_replace(pair)?,
            Rule::replaceb => Self::parse_replaceb(pair)?,
            Rule::textjoin => Self::parse_textjoin(pair)?,
            Rule::trim => Self::parse_trim(pair)?,
            Rule::t => Self::parse_t(pair)?,
            Rule::proper => Self::parse_proper(pair)?,
            // TODO remaining text functions
            Rule::arraytotext
            | Rule::asc
            | Rule::clean
            | Rule::dbcs
            | Rule::dollar
            | Rule::jis
            | Rule::text
            | Rule::numbervalue
            | Rule::phonetic
            | Rule::substitute
            | Rule::textafter
            | Rule::textbefore
            | Rule::textsplit
            | Rule::value
            | Rule::valuetotext
            | Rule::bahttext => return Err(error::Error::NotImplemented(format!("{:?}", pair.as_rule())).into()),
            // Engineering functions
            // Financial functions
            // Logical functions
            Rule::and => Self::parse_and(pair)?,
            Rule::or => Self::parse_or(pair)?,
            Rule::xor => Self::parse_xor(pair)?,
            Rule::not => Self::parse_not(pair)?,
            Rule::if_ => Self::parse_if(pair)?,
            Rule::ifna => Self::parse_ifna(pair)?,
            Rule::iferror => Self::parse_iferror(pair)?,
            // TODO remaining text functions
            Rule::let_
            | Rule::bycol
            | Rule::byrow
            | Rule::makearray
            | Rule::reduce
            | Rule::scan
            | Rule::map
            | Rule::lambda
            | Rule::switch
            | Rule::ifs => return Err(error::Error::NotImplemented(format!("{:?}", pair.as_rule())).into()),
            // Math functions
            // Statistical functions
            // Web functions
            Rule::encodeurl => Self::parse_encodeurl(pair)?,
            Rule::filterxml => Self::parse_filterxml(pair)?,
            Rule::webservice => Self::parse_webservice(pair)?,

            // Basic types
            Rule::num => {
                let number = pair.as_str().trim().parse().unwrap();
                Expr::Number(number)
            }
            Rule::string => {
                let string = pair.into_inner().as_str().to_string();
                Expr::String(string)
            }
            Rule::bool_true => Expr::Bool(true),
            Rule::bool_false => Expr::Bool(false),
            Rule::formula
            | Rule::root
            | Rule::OP
            | Rule::F
            | Rule::CF
            | Rule::CP
            | Rule::C
            | Rule::Q
            | Rule::inner
            | Rule::char_
            | Rule::basic_types
            | Rule::operators
            | Rule::datetime_functions
            | Rule::text_functions
            | Rule::logical_functions
            | Rule::web_functions
            | Rule::WHITESPACE => {
                unreachable!()
            }
        };

        Ok(res)
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
    }
}
