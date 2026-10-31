/*!
[![Crates.io](https://img.shields.io/crates/v/formula.svg?style=flat)](https://crates.io/crates/formula)
[![npm](https://img.shields.io/npm/v/formula-wasm.svg?style=flat)](https://npmjs.com/package/formula-wasm)
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
- 8 logical functions
- 2 web functions
- plus all arithmetic and comparison operators

### Installation and usage

#### Rust

Add this library to your project with `cargo add formula` or add `formula = "*"` to your `Cargo.toml` file.

Use it similar to the following code:

```rust
use formula::{Formula, Expr, Result};

fn main() -> Result<()> {
    let formula = Formula::new("=UPPER(TRIM('   Hello '))")?;
    let value = formula.parse()?;
    assert_eq!(value, Expr::String("HELLO".to_string()));
    Ok(())
}
```

#### JavaScript

Add this library to your project with `npm install formula-wasm` or add `formula-wasm` to your `package.json` file.

Use it similar to the following code:

```js
import { parse } from 'formula-wasm';

const value = parse('=UPPER(TRIM("   Hello "))');
console.assert(value, "HELLO");
```

### What we do not support, yet:

- We don't support all existing functions in the world, but we would like to add more of them, like Excel functions, Google Sheets functions, and so on
- At the moment, we don't support table data. It means you need to extract table data and pass theirs values to this library
- We do not support simple formulas like `1+1` or as argument like `AND(1>3, 1<3)` or `SUM(2-1, 2)`. Instead, you can use our `F.` functions like `AND(F.GT(1, 3), F.LT(1, 3))` or `SUM(F.SUB(2, 1), 2)`
- We still do not support parentheses to change the order of operations, but you can use our `F.` functions. So for example instead of `2*(1+1)`, you should use `F.MUL(2, F.ADD(1, 1))`

### Contributing

We would love to have your contribution! Please read our [contributing guidelines](CONTRIBUTING.md) to get started.

### Inspired by

- [formulajs](https://github.com/formulajs/formulajs)
- [hyperformula](https://github.com/handsontable/hyperformula)

### License

This project is licensed under the MIT license. See the [LICENSE](LICENSE.md) file for more info.
*/

pub mod error;
mod parsers;

use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use parsers::Halt;
use pest::{Parser, iterators::Pair};
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
#[derive(Debug, PartialEq, Clone, PartialOrd)]
pub enum Expr {
    Date(NaiveDate),
    Datetime(DateTime<Utc>),
    Time(NaiveTime),
    Number(f64),
    String(String),
    Bool(bool),
    Array(Vec<Expr>),
    /// An empty value
    Null,
    /// A spreadsheet error like `#DIV/0!`.
    /// Like Excel, it passes through the functions that use it, until `IFERROR`, `IFNA` or the result.
    Error(ExprError),
}

/// A spreadsheet error value, it displays like in Excel, e.g. `#DIV/0!`
///
/// ```rust
/// use formula::{Expr, ExprError, Formula};
///
/// let value = Formula::new("=F.DIV(1, 0)").unwrap().parse().unwrap();
/// assert_eq!(value, Expr::Error(ExprError::DivisionByZero));
/// assert_eq!(ExprError::DivisionByZero.to_string(), "#DIV/0!");
/// ```
#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd)]
#[non_exhaustive]
pub enum ExprError {
    /// `#DIV/0!`
    DivisionByZero,
    /// `#NUM!`, e.g. the square root of a negative number or a result that is too large
    Number,
    /// `#N/A`
    NotAvailable,
}

impl std::fmt::Display for ExprError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let error = match self {
            Self::DivisionByZero => "#DIV/0!",
            Self::Number => "#NUM!",
            Self::NotAvailable => "#N/A",
        };
        f.write_str(error)
    }
}

pub type Result<T> = std::result::Result<T, error::Error>;

/// Excel's limit of nested functions
const MAX_NESTING: usize = 64;

/// Rejects formulas that are nested deeper than Excel allows,
/// before the recursive parser and evaluator could overflow the stack
fn check_nesting(formula: &str) -> Result<()> {
    let mut depth = 0_usize;
    let mut quote = None;
    let mut chars = formula.chars();
    while let Some(c) = chars.next() {
        match (quote, c) {
            // skip escaped characters inside strings
            (Some(_), '\\') => {
                chars.next();
            }
            (Some(q), c) if c == q => quote = None,
            (Some(_), _) => {}
            (None, '\'' | '"') => quote = Some(c),
            (None, '(' | '{') => {
                depth += 1;
                if depth > MAX_NESTING {
                    return Err(error::Error::Nesting(MAX_NESTING));
                }
            }
            (None, ')' | '}') => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    Ok(())
}

impl<'a> Formula<'a> {
    /// To interpret and prepare a new formula, you need to call the `new` method, like the code below:
    ///
    /// ```rust
    /// use formula::{Formula, Expr, Result};
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
    /// Will return `Err` if the formula is not valid, or if it nests functions more than 64 levels deep like Excel.
    pub fn new(formula: &'a str) -> Result<Self> {
        check_nesting(formula)?;
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
    /// Spreadsheet errors like `#DIV/0!` are a valid result, `Ok(Expr::Error(_))`.
    pub fn parse(self) -> Result<Expr> {
        match Self::parse_pair(self.pairs) {
            Ok(value) => Ok(value),
            Err(Halt::Value(error)) => Ok(Expr::Error(error)),
            Err(Halt::Error(error)) => Err(error),
        }
    }

    #[allow(clippy::too_many_lines)]
    fn parse_pair(pair: Pair<Rule>) -> parsers::Result<Expr> {
        // Picking the function first and calling it once keeps this frame small,
        // so deeply nested formulas don't overflow the stack in debug builds
        #[allow(clippy::match_same_arms)]
        let parse: fn(Pair<Rule>) -> parsers::Result<Expr> = match pair.as_rule() {
            // Operators
            Rule::add => Self::parse_add,
            Rule::sub => Self::parse_sub,
            Rule::mul => Self::parse_mul,
            Rule::div => Self::parse_div,
            Rule::pow => Self::parse_pow,
            Rule::eq => Self::parse_eq,
            Rule::ne => Self::parse_ne,
            Rule::gt => Self::parse_gt,
            Rule::lt => Self::parse_lt,
            Rule::gte => Self::parse_gte,
            Rule::lte => Self::parse_lte,
            Rule::percent => Self::parse_percent,
            Rule::negate => Self::parse_negate,

            // Date and time functions
            Rule::date => Self::parse_date,
            Rule::time => Self::parse_time,
            Rule::year => Self::parse_year,
            Rule::month => Self::parse_month,
            Rule::day => Self::parse_day,
            Rule::days => Self::parse_days,
            Rule::edate => Self::parse_edate,
            Rule::eomonth => Self::parse_eomonth,
            Rule::datevalue => Self::parse_datevalue,
            Rule::timevalue => Self::parse_timevalue,
            Rule::hour => Self::parse_hour,
            Rule::minute => Self::parse_minute,
            Rule::second => Self::parse_second,
            Rule::isoweeknum => Self::parse_isoweeknum,
            Rule::weeknum => Self::parse_weeknum,
            Rule::weekday => Self::parse_weekday,
            Rule::now => Self::parse_now,
            Rule::today => Self::parse_today,
            // TODO remaining date and time functions
            Rule::networkdays
            | Rule::networkdaysintl
            | Rule::workday
            | Rule::workdayintl
            | Rule::yearfrac
            | Rule::days360
            | Rule::datediff => return Err(error::Error::NotImplemented(format!("{:?}", pair.as_rule())).into()),

            // Text functions
            Rule::left => Self::parse_left,
            Rule::leftb => Self::parse_leftb,
            Rule::right => Self::parse_right,
            Rule::rightb => Self::parse_rightb,
            Rule::mid => Self::parse_mid,
            Rule::midb => Self::parse_midb,
            Rule::char | Rule::unichar => Self::parse_char,
            Rule::code | Rule::unicode => Self::parse_code,
            Rule::concat | Rule::concatenate => Self::parse_concat,
            Rule::exact => Self::parse_exact,
            Rule::find => Self::parse_find,
            Rule::findb => Self::parse_findb,
            Rule::search => Self::parse_search,
            Rule::searchb => Self::parse_searchb,
            Rule::fixed => Self::parse_fixed,
            Rule::len => Self::parse_len,
            Rule::lenb => Self::parse_lenb,
            Rule::lower => Self::parse_lower,
            Rule::upper => Self::parse_upper,
            Rule::rept => Self::parse_rept,
            Rule::replace => Self::parse_replace,
            Rule::replaceb => Self::parse_replaceb,
            Rule::textjoin => Self::parse_textjoin,
            Rule::trim => Self::parse_trim,
            Rule::t => Self::parse_t,
            Rule::proper => Self::parse_proper,
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
            Rule::and => Self::parse_and,
            Rule::or => Self::parse_or,
            Rule::xor => Self::parse_xor,
            Rule::not => Self::parse_not,
            Rule::if_ => Self::parse_if,
            Rule::ifna => Self::parse_ifna,
            Rule::iferror => Self::parse_iferror,
            Rule::na => Self::parse_na,
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
            Rule::abs => Self::parse_abs,
            Rule::acos => Self::parse_acos,
            Rule::acosh => Self::parse_acosh,
            Rule::asin => Self::parse_asin,
            Rule::asinh => Self::parse_asinh,
            Rule::cos => Self::parse_cos,
            Rule::cosh => Self::parse_cosh,
            Rule::sin => Self::parse_sin,
            Rule::sinh => Self::parse_sinh,
            Rule::tan => Self::parse_tan,
            Rule::tanh => Self::parse_tanh,
            Rule::atan => Self::parse_atan,
            Rule::atan2 => Self::parse_atan2,
            Rule::atanh => Self::parse_atanh,
            Rule::pi => Self::parse_pi,
            Rule::power => Self::parse_pow,
            Rule::mod_ => Self::parse_mod,
            Rule::log => Self::parse_log,
            Rule::log10 => Self::parse_log10,
            Rule::ln => Self::parse_ln,
            Rule::sqrt => Self::parse_sqrt,
            Rule::sqrtpi => Self::parse_sqrtpi,
            Rule::rand => Self::parse_rand,
            Rule::sign => Self::parse_sign,
            Rule::exp => Self::parse_exp,
            Rule::sum => Self::parse_sum,

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
            | Rule::trunc => return Err(error::Error::NotImplemented(format!("{:?}", pair.as_rule())).into()),

            // Statistical functions
            // Web functions
            Rule::encodeurl => Self::parse_encodeurl,
            Rule::filterxml => Self::parse_filterxml,
            Rule::webservice => Self::parse_webservice,

            // Basic types
            Rule::num => Self::parse_num,
            Rule::string => Self::parse_string,
            Rule::bool_true => Self::parse_true,
            Rule::bool_false => Self::parse_false,
            Rule::array => Self::parse_array,
            Rule::formula
            | Rule::root
            | Rule::OP
            | Rule::F
            | Rule::CF
            | Rule::CP
            | Rule::C
            | Rule::Q
            | Rule::inner_single
            | Rule::inner_double
            | Rule::escape
            | Rule::array_row_sep
            | Rule::array_col_sep
            | Rule::basic_types
            | Rule::operators
            | Rule::datetime_functions
            | Rule::text_functions
            | Rule::logical_functions
            | Rule::math_functions
            | Rule::web_functions
            | Rule::WHITESPACE
            | Rule::EOI => {
                unreachable!()
            }
        };
        let res = parse(pair)?;

        // Spreadsheet errors stop the evaluation until a function handles them
        match res {
            Expr::Error(error) => Err(Halt::Value(error)),
            res => Ok(res),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nesting_limit() {
        let nested = |levels: usize| format!("={}-1{}", "ABS(".repeat(levels), ")".repeat(levels));

        let formula = nested(MAX_NESTING);
        let value = Formula::new(&formula).unwrap().parse().unwrap();
        assert_eq!(value, Expr::Number(1.0));

        for levels in [MAX_NESTING + 1, 100_000] {
            let formula = nested(levels);
            assert!(matches!(
                Formula::new(&formula),
                Err(error::Error::Nesting(MAX_NESTING))
            ));
        }

        let formula = format!("={}1{}", "{".repeat(MAX_NESTING + 1), "}".repeat(MAX_NESTING + 1));
        assert!(matches!(Formula::new(&formula), Err(error::Error::Nesting(_))));

        // brackets inside strings don't count
        let formula = format!("=LEN('{}\\'{}')", "(".repeat(100), "{".repeat(100));
        let value = Formula::new(&formula).unwrap().parse().unwrap();
        assert_eq!(value, Expr::Number(201.0));
    }

    #[test]
    fn test_not_implemented_functions_are_recognized() {
        for formula in [
            "=FLOOR.PRECISE(1)",
            "=SWITCH(1, 1, 'a')",
            "=SWITCH(1, 1, 'a', 2, 'b', 'c')",
            "=WORKDAY(DATE(2020,1,1), 5)",
            "=WORKDAY.INTL(DATE(2020,1,1), 5)",
        ] {
            let formula = Formula::new(formula).unwrap();
            assert!(matches!(formula.parse(), Err(error::Error::NotImplemented(_))));
        }
    }
}
