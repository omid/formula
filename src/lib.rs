mod error;
mod parsers;

use crate::parsers::date_and_time::*;
use anyhow::Result;
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use pest::iterators::Pair;
use pest_derive::Parser;
use crate::parsers::text::*;

#[derive(Parser, Debug)]
#[grammar = "formula.pest"]
pub struct Formula;

#[derive(Debug, PartialEq)]
pub enum Expr {
    Date(NaiveDate),
    Datetime(DateTime<Utc>),
    Time(NaiveTime),
    Number(f64),
    String(String),
    Bool(bool),
}

///
/// # Panics
///
/// Will panic if the function is not implemented yet.
///
pub fn parse_value(pair: Pair<Rule>) -> Result<Expr> {
    let res = match pair.as_rule() {
        // Date and time functions
        Rule::date => parse_date(pair)?,
        Rule::time => parse_time(pair)?,
        Rule::year => parse_year(pair)?,
        Rule::month => parse_month(pair)?,
        Rule::day => parse_day(pair)?,
        Rule::days => parse_days(pair)?,
        Rule::edate => parse_edate(pair)?,
        Rule::eomonth => parse_eomonth(pair)?,
        Rule::datevalue => parse_datevalue(pair)?,
        Rule::timevalue => parse_timevalue(pair)?,
        Rule::hour => parse_hour(pair)?,
        Rule::minute => parse_minute(pair)?,
        Rule::second => parse_second(pair)?,
        Rule::isoweeknum => parse_isoweeknum(pair)?,
        Rule::weeknum => parse_weeknum(pair)?,
        Rule::weekday => parse_weekday(pair)?,
        Rule::now => parse_now(),
        Rule::today => parse_today(),
        // TODO remaining date and time functions
        Rule::networkdays
        | Rule::networkdaysintl
        | Rule::workdays
        | Rule::workdaysintl
        | Rule::yearfrac
        | Rule::days360
        | Rule::datediff => todo!("Not implemented: {:?}", pair.as_rule()),
        // Text functions
        Rule::left => parse_left(pair)?,
        Rule::leftb => parse_leftb(pair)?,
        Rule::right => parse_right(pair)?,
        Rule::rightb => parse_rightb(pair)?,
        Rule::mid => parse_mid(pair)?,
        Rule::midb => parse_midb(pair)?,
        Rule::char | Rule::unichar => parse_char(pair)?,
        Rule::code | Rule::unicode => parse_code(pair)?,
        Rule::concat | Rule::concatenate => parse_concat(pair)?,
        Rule::exact => parse_exact(pair)?,
        Rule::find => parse_find(pair)?,
        Rule::findb => parse_findb(pair)?,
        Rule::search => parse_search(pair)?,
        Rule::searchb => parse_searchb(pair)?,
        Rule::fixed => parse_fixed(pair)?,
        Rule::len => parse_len(pair)?,
        Rule::lenb => parse_lenb(pair)?,
        Rule::lower => parse_lower(pair)?,
        Rule::upper => parse_upper(pair)?,
        Rule::rept => parse_rept(pair)?,
        Rule::replace => parse_replace(pair)?,
        Rule::replaceb => parse_replaceb(pair)?,
        Rule::textjoin => parse_textjoin(pair)?,
        Rule::trim => parse_trim(pair)?,
        Rule::t => parse_t(pair)?,
        Rule::proper => parse_proper(pair)?,
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
