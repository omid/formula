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
- 26 math functions
- 7 logical functions
- 2 web functions
- plus all arithmetic and comparison operators

## Installation and usage

Add this library to your project with `cargo add formula` or add `formula = "*"` to your `Cargo.toml` file.

Use it similar to the following code:

```rust
use formula::{Formula, Expr, error::Error, Result};

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

use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

#[derive(Parser, Debug)]
#[grammar = "formula.pest"]
struct FormulaInner;

/// `Formula`, is the main struct and entry point of this library.
#[derive(Debug)]
pub struct Formula<'a> {
    pairs: Pair<'a, Rule>,
}

/// `Expr` is the result of parsing a formula.
///
/// There is a difference between Excel and this library here.
/// We don't have a `#N/A`, `#VALUE!`, `#DIV/0!`, `#NUM!`, `#NULL!` error types, instead it will return `Expr::Null`.
#[derive(Debug, PartialEq)]
pub enum Expr {
    Date(NaiveDate),
    Datetime(DateTime<Utc>),
    Time(NaiveTime),
    Number(f64),
    String(String),
    Bool(bool),
    Array(Vec<Expr>),
    Null,
}

pub type Result<T> = std::result::Result<T, error::Error>;

impl<'a> Formula<'a> {
    /// To interpret and prepare a new formula, you need to call the `new` method, like the code below:
    ///
    /// ```rust
    /// use formula::{Formula, Expr, error::Error, Result};
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
        let pairs = FormulaInner::parse(Rule::root, formula)
            .map_err(|_| error::Error::Parser("root".to_string()))?
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
            Rule::now => Self::parse_now(pair)?,
            Rule::today => Self::parse_today(pair)?,
            // TODO remaining date and time functions
            Rule::networkdays
            | Rule::networkdaysintl
            | Rule::workdays
            | Rule::workdaysintl
            | Rule::yearfrac
            | Rule::days360
            | Rule::datediff => return Err(error::Error::NotImplemented(format!("{:?}", pair.as_rule()))),

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
            | Rule::bahttext => return Err(error::Error::NotImplemented(format!("{:?}", pair.as_rule()))),
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
            | Rule::ifs => return Err(error::Error::NotImplemented(format!("{:?}", pair.as_rule()))),
            // Math functions
            Rule::abs => Self::parse_abs(pair)?,
            Rule::acos => Self::parse_acos(pair)?,
            Rule::acosh => Self::parse_acosh(pair)?,
            Rule::asin => Self::parse_asin(pair)?,
            Rule::asinh => Self::parse_asinh(pair)?,
            Rule::cos => Self::parse_cos(pair)?,
            Rule::cosh => Self::parse_cosh(pair)?,
            Rule::sin => Self::parse_sin(pair)?,
            Rule::sinh => Self::parse_sinh(pair)?,
            Rule::tan => Self::parse_tan(pair)?,
            Rule::tanh => Self::parse_tanh(pair)?,
            Rule::atan => Self::parse_atan(pair)?,
            Rule::atan2 => Self::parse_atan2(pair)?,
            Rule::atanh => Self::parse_atanh(pair)?,
            Rule::pi => Self::parse_pi(pair)?,
            Rule::power => Self::parse_pow(pair)?,
            Rule::mod_ => Self::parse_mod(pair)?,
            Rule::log => Self::parse_log(pair)?,
            Rule::log10 => Self::parse_log10(pair)?,
            Rule::ln => Self::parse_ln(pair)?,
            Rule::sqrt => Self::parse_sqrt(pair)?,
            Rule::sqrtpi => Self::parse_sqrtpi(pair)?,
            Rule::rand => Self::parse_rand(pair)?,
            Rule::sign => Self::parse_sign(pair)?,
            Rule::exp => Self::parse_exp(pair)?,
            Rule::sum => Self::parse_sum(pair)?,

            // TODO remaining text functions
            Rule::ceiling
            | Rule::round
            | Rule::floor
            | Rule::acot
            | Rule::acoth
            | Rule::aggregate
            | Rule::arabic
            | Rule::base
            | Rule::ceiling_math
            | Rule::ceiling_precise
            | Rule::combin
            | Rule::combina
            | Rule::cot
            | Rule::coth
            | Rule::csc
            | Rule::csch
            | Rule::decimal
            | Rule::degrees
            | Rule::even
            | Rule::fact
            | Rule::factdouble
            | Rule::floor_math
            | Rule::floor_precise
            | Rule::gcd
            | Rule::int
            | Rule::iso_ceiling
            | Rule::lcm
            | Rule::mdeterm
            | Rule::minverse
            | Rule::mmult
            | Rule::mround
            | Rule::multinomial
            | Rule::munit
            | Rule::odd
            | Rule::product
            | Rule::quotient
            | Rule::radians
            | Rule::randarray
            | Rule::randbetween
            | Rule::roman
            | Rule::rounddown
            | Rule::roundup
            | Rule::sec
            | Rule::sech
            | Rule::sequence
            | Rule::seriessum
            | Rule::subtotal
            | Rule::sumif
            | Rule::sumifs
            | Rule::sumproduct
            | Rule::sumsq
            | Rule::sumx2my2
            | Rule::sumx2py2
            | Rule::sumxmy2
            | Rule::trunc => return Err(error::Error::NotImplemented(format!("{:?}", pair.as_rule()))),

            // Statistical functions
            // Web functions
            Rule::encodeurl => Self::parse_encodeurl(pair)?,
            Rule::filterxml => Self::parse_filterxml(pair)?,
            Rule::webservice => Self::parse_webservice(pair)?,

            // Basic types
            Rule::num => Self::parse_num(pair)?,
            Rule::string => Self::parse_string(pair)?,
            Rule::bool_true => Self::parse_true(pair)?,
            Rule::bool_false => Self::parse_false(pair)?,
            Rule::array => Self::parse_array(pair)?,
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
            | Rule::array_row_sep
            | Rule::array_col_sep
            | Rule::basic_types
            | Rule::operators
            | Rule::datetime_functions
            | Rule::text_functions
            | Rule::logical_functions
            | Rule::math_functions
            | Rule::web_functions
            | Rule::WHITESPACE => {
                unreachable!()
            }
        };

        Ok(res)
    }
}
