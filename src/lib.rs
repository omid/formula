/*!
[![Crates.io](https://img.shields.io/crates/v/formula.svg)](https://crates.io/crates/formula)
[![Workflow Status](https://github.com/omid/formula/workflows/main/badge.svg)](https://github.com/omid/formula/actions?query=workflow%3A%22ci%22)

<p align="center">
  <strong style="font-size: 50px"><em>Formula</em></strong>
</p>

<p align="center">
  <strong>A parser and evaluator of spreadsheet-like formulas</strong>
</p>

It's in its early stages, and we are trying to add more functions and features soon.

So far we have the following features:

- 18 date time functions
- 26 text functions

## Installation and usage

Add this library to your project with `cargo add formula` or add `formula = "*"` to your `Cargo.toml` file.

Use it similar to the following code:

```rust
use formula::{Formula, Expr, error::Error};
use anyhow::Result;

fn main() -> Result<()> {
    let formula = Formula::new("UPPER(TRIM('   Hello '))")?;
    let value = formula.parse().unwrap();
    assert_eq!(value, Expr::String("HELLO".to_string()));
    Ok(())
}
```

## What we do not support:

- We would like to add more functions, like Excel functions, Google Sheets functions, and more
- At the moment, we don't support table data, so you need to pass values to the formula as arguments by yourself
- We do not support simple formulas like `1+1` or as argument like `AND(1>3, 1<3)` or `SUM(2-1, 2)`, yet

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
pub struct Formula<'a> {
    pairs: Pair<'a, Rule>,
}

#[derive(Debug, PartialEq)]
pub enum Expr {
    Date(NaiveDate),
    Datetime(DateTime<Utc>),
    Time(NaiveTime),
    Number(f64),
    String(String),
    Bool(bool),
}

impl<'a> Formula<'a> {
    pub fn new(formula: &'a str) -> Result<Self> {
        let pairs = FormulaInner::parse(Rule::formula, formula)?
            .next()
            .ok_or(error::Error::Parser("No formula found"))?;
        Ok(Self { pairs })
    }

    ///
    /// # Panics
    ///
    /// Will panic if the function is not implemented yet.
    ///
    pub fn parse(self) -> Result<Expr> {
        Self::parse_pair(self.pairs)
    }

    fn parse_pair(pair: Pair<Rule>) -> Result<Expr> {
        let res = match pair.as_rule() {
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
            | Rule::datediff => todo!("Not implemented: {:?}", pair.as_rule()),
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
            | Rule::bahttext => todo!("Not implemented: {:?}", pair.as_rule()),
            // Engineering functions
            // Financial functions
            // Logical functions
            // Math functions
            // Statistical functions
            // Web functions

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
            | Rule::OP
            | Rule::F
            | Rule::CF
            | Rule::CP
            | Rule::C
            | Rule::Q
            | Rule::inner
            | Rule::char_
            | Rule::basic_types
            | Rule::datetime_functions
            | Rule::text_functions
            | Rule::WHITESPACE => {
                unreachable!()
            }
        };

        Ok(res)
    }
}
