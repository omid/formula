use crate::{error::Error, parse_value, Expr, Rule};
use anyhow::Result;
use chrono::{Datelike, NaiveDate, NaiveTime, Timelike, Utc};
use chronoutil::shift_months;
use pest::iterators::Pair;

pub fn parse_now() -> Expr {
    Expr::Datetime(Utc::now())
}

pub fn parse_today() -> Expr {
    Expr::Date(Utc::now().naive_utc().date())
}

pub fn parse_weekday(pair: Pair<Rule>) -> Result<Expr> {
    const RULE: &str = "weekday";
    let mut args = pair.into_inner();
    let date = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    // TODO support return type, ref: https://support.microsoft.com/en-us/office/weekday-function-60e44483-2ed1-439f-8bd0-e404c190949a
    // let return_type = args.next().map_or_else(Expr::Number(1.0));

    let weekday = match date {
        Expr::Date(date) => date.weekday(),
        _ => return Err(Error::Parser(RULE).into()),
    };
    Ok(Expr::Number(f64::from(weekday as i32)))
}

pub fn parse_weeknum(_pair: Pair<Rule>) -> Result<Expr> {
    // const RULE: &str = "weeknum";
    // let mut args = pair.into_inner();
    // let date = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    // let return_type = args.next().map(|v| parse_value(v)?).unwrap_or(Expr::Number(1.0));
    //
    // let start_week = match return_type {
    //     Expr::Number(16.0) => Weekday::Sat,
    //     Expr::Number(1.0) | Expr::Number(17.0) => Weekday::Sun,
    //     Expr::Number(2.0) | Expr::Number(11.0) | Expr::Number(21.0)  => Weekday::Mon,
    //     Expr::Number(12.0)  => Weekday::Tue,
    //     Expr::Number(13.0)  => Weekday::Wed,
    //     Expr::Number(14.0)  => Weekday::Thu,
    //     Expr::Number(15.0)  => Weekday::Fri,
    //     _ => return Err(Error::Parser(RULE).into()),
    // };
    // let week = match date {
    //     Expr::Date(date) =>  date.week(start_week),
    //     _ => return Err(Error::Parser(RULE).into()),
    // };
    // Ok(Expr::Number(week as i32 as f64))
    todo!()
}

pub fn parse_isoweeknum(pair: Pair<Rule>) -> Result<Expr> {
    const RULE: &str = "isoweeknum";
    let arg = parse_value(pair.into_inner().next().ok_or(Error::Parser(RULE))?)?;
    let second = match arg {
        Expr::Date(date) => date.iso_week().week(),
        Expr::String(date) => datestring_to_naivedate(&date)?.iso_week().week(),
        _ => return Err(Error::Parser(RULE).into()),
    };
    Ok(Expr::Number(f64::from(second)))
}

pub fn parse_second(pair: Pair<Rule>) -> Result<Expr> {
    const RULE: &str = "second";
    let arg = parse_value(pair.into_inner().next().ok_or(Error::Parser(RULE))?)?;
    let second = match arg {
        Expr::Time(time) => time.second(),
        Expr::String(time) => timestring_to_naivetime(&time)?.second(),
        _ => return Err(Error::Parser(RULE).into()),
    };
    Ok(Expr::Number(f64::from(second)))
}

pub fn parse_minute(pair: Pair<Rule>) -> Result<Expr> {
    const RULE: &str = "minute";
    let arg = parse_value(pair.into_inner().next().ok_or(Error::Parser(RULE))?)?;
    let minute = match arg {
        Expr::Time(time) => time.minute(),
        Expr::String(time) => timestring_to_naivetime(&time)?.minute(),
        _ => return Err(Error::Parser(RULE).into()),
    };
    Ok(Expr::Number(f64::from(minute)))
}

pub fn parse_hour(pair: Pair<Rule>) -> Result<Expr> {
    const RULE: &str = "hour";
    let arg = parse_value(pair.into_inner().next().ok_or(Error::Parser(RULE))?)?;
    let hour = match arg {
        Expr::Time(time) => time.hour(),
        Expr::String(time) => timestring_to_naivetime(&time)?.hour(),
        _ => return Err(Error::Parser(RULE).into()),
    };
    Ok(Expr::Number(f64::from(hour)))
}

pub fn parse_timevalue(pair: Pair<Rule>) -> Result<Expr> {
    const RULE: &str = "timevalue";
    let arg = parse_value(pair.into_inner().next().ok_or(Error::Parser(RULE))?)?;
    let time = match arg {
        Expr::String(datestring) => timestring_to_naivetime(&datestring)?,
        _ => return Err(Error::Parser(RULE).into()),
    };
    Ok(Expr::Time(time))
}

pub fn parse_datevalue(pair: Pair<Rule>) -> Result<Expr> {
    const RULE: &str = "datevalue";
    let arg = parse_value(pair.into_inner().next().ok_or(Error::Parser(RULE))?)?;
    let date = match arg {
        Expr::String(datestring) => datestring_to_naivedate(&datestring)?,
        _ => return Err(Error::Parser(RULE).into()),
    };
    Ok(Expr::Date(date))
}

pub fn parse_eomonth(pair: Pair<Rule>) -> Result<Expr> {
    const RULE: &str = "eomonth";
    let mut args = pair.into_inner();
    let date = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let num = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let date = match (date, num) {
        (Expr::String(date), Expr::Number(num)) => {
            let date = datestring_to_naivedate(&date)?;
            let date = shift_months(date, num as i32);
            let last_day = last_day_of_month(date.year(), date.month());
            NaiveDate::from_ymd(date.year(), date.month(), last_day)
        }
        (Expr::Date(date), Expr::Number(num)) => {
            let date = shift_months(date, num as i32);
            let last_day = last_day_of_month(date.year(), date.month());
            NaiveDate::from_ymd(date.year(), date.month(), last_day)
        }
        _ => return Err(Error::Parser(RULE).into()),
    };

    Ok(Expr::Date(date))
}

pub fn parse_edate(pair: Pair<Rule>) -> Result<Expr> {
    const RULE: &str = "edate";
    let mut args = pair.into_inner();
    let date = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let num = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let date = match (date, num) {
        (Expr::String(date), Expr::Number(num)) => {
            let date = datestring_to_naivedate(&date)?;
            shift_months(date, num as i32)
        }
        (Expr::Date(date), Expr::Number(num)) => shift_months(date, num as i32),
        _ => return Err(Error::Parser(RULE).into()),
    };

    Ok(Expr::Date(date))
}

pub fn parse_days(pair: Pair<Rule>) -> Result<Expr> {
    const RULE: &str = "days";
    let mut args = pair.into_inner();
    let end_date = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let start_date = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let days = match (end_date, start_date) {
        (Expr::Date(end_date), Expr::Date(start_date)) => end_date.signed_duration_since(start_date).num_days() as f64,
        (Expr::String(end_date), Expr::String(start_date)) => {
            let end_date = datestring_to_naivedate(&end_date)?;
            let start_date = datestring_to_naivedate(&start_date)?;
            end_date.signed_duration_since(start_date).num_days() as f64
        }
        (Expr::Number(end_date), Expr::Number(start_date)) => end_date - start_date,
        _ => return Err(Error::Parser(RULE).into()),
    };

    Ok(Expr::Number(days as f64))
}

pub fn parse_day(pair: Pair<Rule>) -> Result<Expr> {
    const RULE: &str = "day";
    let arg = parse_value(pair.into_inner().next().ok_or(Error::Parser(RULE))?)?;
    let day = match arg {
        Expr::Date(date) => date.day(),
        Expr::String(date) => datestring_to_naivedate(&date)?.day(),
        _ => return Err(Error::Parser(RULE).into()),
    };
    Ok(Expr::Number(f64::from(day)))
}

pub fn parse_month(pair: Pair<Rule>) -> Result<Expr> {
    const RULE: &str = "month";
    let arg = parse_value(pair.into_inner().next().ok_or(Error::Parser(RULE))?)?;
    let month = match arg {
        Expr::Date(date) => date.month(),
        Expr::String(date) => datestring_to_naivedate(&date)?.month(),
        _ => return Err(Error::Parser(RULE).into()),
    };
    Ok(Expr::Number(f64::from(month)))
}

pub fn parse_year(pair: Pair<Rule>) -> Result<Expr> {
    const RULE: &str = "year";
    let arg = parse_value(pair.into_inner().next().ok_or(Error::Parser(RULE))?)?;
    let year = match arg {
        Expr::Date(date) => date.year(),
        Expr::String(date) => datestring_to_naivedate(&date)?.year(),
        _ => return Err(Error::Parser(RULE).into()),
    };
    Ok(Expr::Number(f64::from(year)))
}

pub fn parse_time(pair: Pair<Rule>) -> Result<Expr> {
    const RULE: &str = "time";
    let mut args = pair.into_inner();
    let hour = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let minute = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let second = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let time = match (hour, minute, second) {
        (Expr::Number(hour), Expr::Number(minute), Expr::Number(second)) => {
            Expr::Time(NaiveTime::from_hms(hour as u32, minute as u32, second as u32))
        }
        _ => return Err(Error::Parser(RULE).into()),
    };
    Ok(time)
}

pub fn parse_date(pair: Pair<Rule>) -> Result<Expr> {
    const RULE: &str = "date";
    let mut args = pair.into_inner();
    let year = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let month = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let day = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let date = match (year, month, day) {
        (Expr::Number(year), Expr::Number(month), Expr::Number(day)) => {
            Expr::Date(NaiveDate::from_ymd(year as i32, month as u32, day as u32))
        }
        _ => return Err(Error::Parser(RULE).into()),
    };
    Ok(date)
}

// fn datestring_to_naivedate(datestring: &str) -> NaiveDate {
//     let datestring = datestring.trim().trim_matches('\'').trim_matches('"');
//
//     NaiveDate::parse_from_str(datestring, "%m/%d/%Y")
//         .unwrap_or(NaiveDate::parse_from_str(datestring, "%v").ok_or(Error::Parser(RULE))?)
// }
//
// fn timestring_to_naivetime(timestring: &str) -> NaiveTime {
//     let timestring = timestring.trim().trim_matches('\'').trim_matches('"');
//     NaiveTime::parse_from_str(timestring, "%T")
//         .unwrap_or(NaiveTime::parse_from_str(timestring, "%r").ok_or(Error::Parser(RULE))?)
// }

fn datestring_to_naivedate(datestring: &str) -> Result<NaiveDate> {
    let mdy = datestring
        .split('/')
        .map(|s| s.parse::<i32>())
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(NaiveDate::from_ymd(mdy[2], mdy[0] as u32, mdy[1] as u32))
}

fn timestring_to_naivetime(timestring: &str) -> Result<NaiveTime> {
    let hms = timestring
        .trim_matches('\'')
        .trim_matches('"')
        .split(':')
        .map(|s| s.parse::<i32>())
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(NaiveTime::from_hms(hms[0] as u32, hms[1] as u32, hms[2] as u32))
}

fn last_day_of_month(year: i32, month: u32) -> u32 {
    NaiveDate::from_ymd_opt(year, month + 1, 1)
        .unwrap_or_else(|| NaiveDate::from_ymd(year + 1, 1, 1))
        .pred()
        .day()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Formula;
    use pest::Parser;

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_parse_date_and_time_types() {
        let formula = Formula::parse(Rule::formula, "DATE(2020,2,3)").unwrap().next().unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::Date(NaiveDate::from_ymd(2020, 2, 3)));

        let formula = Formula::parse(
            Rule::formula,
            "DATE(YEAR('1/30/2020'),MONTH('1/30/2020'),DAY('1/30/2020'))".trim(),
        )
        .unwrap()
        .next()
        .unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::Date(NaiveDate::from_ymd(2020, 1, 30)));

        let formula = Formula::parse(Rule::formula, "YEAR('1/30/2020')")
            .unwrap()
            .next()
            .unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::Number(2020.0));

        let formula = Formula::parse(Rule::formula, "MONTH('1/30/2020')")
            .unwrap()
            .next()
            .unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::Number(1.0));

        let formula = Formula::parse(Rule::formula, "DAY('1/30/2020')")
            .unwrap()
            .next()
            .unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::Number(30.0));

        let formula = Formula::parse(Rule::formula, "DATEVALUE('1/30/2020')")
            .unwrap()
            .next()
            .unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::Date(NaiveDate::from_ymd(2020, 1, 30)));

        let formula = Formula::parse(Rule::formula, "DAYS('3/30/2020', '1/30/2020')")
            .unwrap()
            .next()
            .unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::Number(60.0));

        let formula = Formula::parse(Rule::formula, "DAYS(10, 5)").unwrap().next().unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::Number(5.0));

        let formula = Formula::parse(Rule::formula, "DAYS(DATE(2020,1,6), DATE(2020,1,1))")
            .unwrap()
            .next()
            .unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::Number(5.0));

        let formula = Formula::parse(Rule::formula, "EDATE('1/30/2020', 5)")
            .unwrap()
            .next()
            .unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::Date(NaiveDate::from_ymd(2020, 6, 30)));

        let formula = Formula::parse(Rule::formula, "EOMONTH('1/20/2020', 5)")
            .unwrap()
            .next()
            .unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::Date(NaiveDate::from_ymd(2020, 6, 30)));

        let formula = Formula::parse(Rule::formula, "HOUR('02:30:00')")
            .unwrap()
            .next()
            .unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::Number(2.0));

        let formula = Formula::parse(Rule::formula, "MINUTE('02:30:00')")
            .unwrap()
            .next()
            .unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::Number(30.0));

        let formula = Formula::parse(Rule::formula, "SECOND('02:30:00')")
            .unwrap()
            .next()
            .unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::Number(0.0));

        let formula = Formula::parse(Rule::formula, "ISOWEEKNUM('1/30/2020')")
            .unwrap()
            .next()
            .unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::Number(5.0));

        let formula = Formula::parse(Rule::formula, "TIME(2,30,0)").unwrap().next().unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::Time(NaiveTime::from_hms(2, 30, 0)));

        let formula = Formula::parse(Rule::formula, "TIMEVALUE('02:30:00')")
            .unwrap()
            .next()
            .unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::Time(NaiveTime::from_hms(2, 30, 0)));

        let formula = Formula::parse(Rule::formula, "WEEKDAY(DATE(2020,1,1))")
            .unwrap()
            .next()
            .unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::Number(2.0));

        // let formula = Formula::parse(Rule::formula, "WEEKDAY(DATE(2020,1,1), 1)")
        //     .unwrap()
        //     .next()
        //     .unwrap();
        // let value = parse_value(formula).unwrap();
        // assert_eq!(value, Expr::Number(5.0));

        // let formula = Formula::parse(Rule::formula, "WEEKDAY(DATE(2020,1,1), 3)")
        //     .unwrap()
        //     .next()
        //     .unwrap();
        // let value = parse_value(formula).unwrap();
        // assert_eq!(value, Expr::Number(5.0));

        // let formula = Formula::parse(Rule::formula, "WEEKNUM(DATE(2020,1,1))")
        //     .unwrap()
        //     .next()
        //     .unwrap();
        // let value = parse_value(formula).unwrap();
        // assert_eq!(value, Expr::Number(2.0));
    }
}
