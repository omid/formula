use super::helper::parser_error;
use crate::{error::Error, Expr, Formula, Result, Rule};
use chrono::{Datelike, Months, NaiveDate, NaiveTime, Timelike, Utc, Weekday};
use pest::iterators::Pair;

impl Formula<'_> {
    #[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
    pub(crate) fn parse_now(_pair: Pair<Rule>) -> Result<Expr> {
        Ok(Expr::Datetime(Utc::now()))
    }

    #[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps)]
    pub(crate) fn parse_today(_pair: Pair<Rule>) -> Result<Expr> {
        Ok(Expr::Date(Utc::now().naive_utc().date()))
    }

    /// ref: <https://support.microsoft.com/en-us/office/weekday-function-60e44483-2ed1-439f-8bd0-e404c190949a>
    pub(crate) fn parse_weekday(pair: Pair<Rule>) -> Result<Expr> {
        let rule = pair.as_rule();
        let mut args = pair.into_inner();
        let date = Self::to_date(Self::get_formula(&mut args, rule)?, rule)?;
        let return_type = Self::get_opt_formula_with_default(&mut args, Expr::Number(1.0))?;

        let weekday = date.weekday();
        let weekday = match return_type {
            Expr::Number(return_type) => match return_type as u32 {
                1 | 17 => weekday.number_from_sunday(),
                2 | 11 => weekday.number_from_monday(),
                3 => weekday.num_days_from_monday(),
                12 => weekday.days_since(Weekday::Tue) + 1,
                13 => weekday.days_since(Weekday::Wed) + 1,
                14 => weekday.days_since(Weekday::Thu) + 1,
                15 => weekday.days_since(Weekday::Fri) + 1,
                16 => weekday.days_since(Weekday::Sat) + 1,
                _ => return Err(parser_error(rule)),
            },
            _ => return Err(parser_error(rule)),
        };
        Ok(Expr::Number(f64::from(weekday)))
    }

    #[allow(clippy::needless_pass_by_value)]
    pub(crate) fn parse_weeknum(pair: Pair<Rule>) -> Result<Expr> {
        // let rule_name = format!("{:?}", &pair.as_rule());
        // let mut args = pair.into_inner();
        // let date = Self::get_formula(&mut args, &rule_name)?;
        // let return_type = args.next().map(|v| Self::parse_pair(v)?).unwrap_or(Expr::Number(1.0));
        //
        // let start_week = match return_type {
        //     Expr::Number(16.0) => Weekday::Sat,
        //     Expr::Number(1.0) | Expr::Number(17.0) => Weekday::Sun,
        //     Expr::Number(2.0) | Expr::Number(11.0) | Expr::Number(21.0)  => Weekday::Mon,
        //     Expr::Number(12.0)  => Weekday::Tue,
        //     Expr::Number(13.0)  => Weekday::Wed,
        //     Expr::Number(14.0)  => Weekday::Thu,
        //     Expr::Number(15.0)  => Weekday::Fri,
        //     _ => return Err(Error::Parser(rule_name).into()),
        // };
        // let week = match date {
        //     Expr::Date(date) =>  date.week(start_week),
        //     _ => return Err(Error::Parser(rule_name).into()),
        // };
        // Ok(Expr::Number(week as i32 as f64))
        Err(Error::NotImplemented(format!("{:?}", pair.as_rule())))
    }

    pub(crate) fn parse_isoweeknum(pair: Pair<Rule>) -> Result<Expr> {
        Self::parse_date_part(pair, |date| f64::from(date.iso_week().week()))
    }

    pub(crate) fn parse_second(pair: Pair<Rule>) -> Result<Expr> {
        Self::parse_time_part(pair, |time| f64::from(time.second()))
    }

    pub(crate) fn parse_minute(pair: Pair<Rule>) -> Result<Expr> {
        Self::parse_time_part(pair, |time| f64::from(time.minute()))
    }

    pub(crate) fn parse_hour(pair: Pair<Rule>) -> Result<Expr> {
        Self::parse_time_part(pair, |time| f64::from(time.hour()))
    }

    pub(crate) fn parse_timevalue(pair: Pair<Rule>) -> Result<Expr> {
        let rule = pair.as_rule();
        let mut args = pair.into_inner();

        match Self::get_formula(&mut args, rule)? {
            Expr::String(timestring) => Ok(Expr::Time(Self::timestring_to_naivetime(&timestring, rule)?)),
            _ => Err(parser_error(rule)),
        }
    }

    pub(crate) fn parse_datevalue(pair: Pair<Rule>) -> Result<Expr> {
        let rule = pair.as_rule();
        let mut args = pair.into_inner();

        match Self::get_formula(&mut args, rule)? {
            Expr::String(datestring) => Ok(Expr::Date(Self::datestring_to_naivedate(&datestring, rule)?)),
            _ => Err(parser_error(rule)),
        }
    }

    pub(crate) fn parse_eomonth(pair: Pair<Rule>) -> Result<Expr> {
        let rule = pair.as_rule();
        let mut args = pair.into_inner();
        let date = Self::to_date(Self::get_formula(&mut args, rule)?, rule)?;
        let num = Self::get_formula(&mut args, rule)?;

        let date = match num {
            // the day before the first day of the next month
            Expr::Number(num) => date
                .with_day(1)
                .and_then(|date| Self::shift_months(date, num))
                .and_then(|date| date.checked_add_months(Months::new(1)))
                .and_then(|date| date.pred_opt()),
            _ => None,
        };
        date.map(Expr::Date).ok_or_else(|| parser_error(rule))
    }

    pub(crate) fn parse_edate(pair: Pair<Rule>) -> Result<Expr> {
        let rule = pair.as_rule();
        let mut args = pair.into_inner();
        let date = Self::to_date(Self::get_formula(&mut args, rule)?, rule)?;
        let num = Self::get_formula(&mut args, rule)?;

        let date = match num {
            Expr::Number(num) => Self::shift_months(date, num),
            _ => None,
        };
        date.map(Expr::Date).ok_or_else(|| parser_error(rule))
    }

    pub(crate) fn parse_days(pair: Pair<Rule>) -> Result<Expr> {
        let rule = pair.as_rule();
        let mut args = pair.into_inner();
        let end_date = Self::get_formula(&mut args, rule)?;
        let start_date = Self::get_formula(&mut args, rule)?;

        let days = match (end_date, start_date) {
            (Expr::Number(end_date), Expr::Number(start_date)) => end_date - start_date,
            (end_date, start_date) => {
                let end_date = Self::to_date(end_date, rule)?;
                let start_date = Self::to_date(start_date, rule)?;
                end_date.signed_duration_since(start_date).num_days() as f64
            }
        };
        Ok(Expr::Number(days))
    }

    pub(crate) fn parse_day(pair: Pair<Rule>) -> Result<Expr> {
        Self::parse_date_part(pair, |date| f64::from(date.day()))
    }

    pub(crate) fn parse_month(pair: Pair<Rule>) -> Result<Expr> {
        Self::parse_date_part(pair, |date| f64::from(date.month()))
    }

    pub(crate) fn parse_year(pair: Pair<Rule>) -> Result<Expr> {
        Self::parse_date_part(pair, |date| f64::from(date.year()))
    }

    pub(crate) fn parse_time(pair: Pair<Rule>) -> Result<Expr> {
        let rule = pair.as_rule();
        let mut args = pair.into_inner();
        let hour = Self::get_formula(&mut args, rule)?;
        let minute = Self::get_formula(&mut args, rule)?;
        let second = Self::get_formula(&mut args, rule)?;

        let time = match (hour, minute, second) {
            (Expr::Number(hour), Expr::Number(minute), Expr::Number(second))
                if hour >= 0.0 && minute >= 0.0 && second >= 0.0 =>
            {
                NaiveTime::from_hms_opt(hour as u32, minute as u32, second as u32)
            }
            _ => None,
        };
        time.map(Expr::Time).ok_or_else(|| parser_error(rule))
    }

    pub(crate) fn parse_date(pair: Pair<Rule>) -> Result<Expr> {
        let rule = pair.as_rule();
        let mut args = pair.into_inner();
        let year = Self::get_formula(&mut args, rule)?;
        let month = Self::get_formula(&mut args, rule)?;
        let day = Self::get_formula(&mut args, rule)?;

        let date = match (year, month, day) {
            (Expr::Number(year), Expr::Number(month), Expr::Number(day)) => {
                NaiveDate::from_ymd_opt(year as i32, month as u32, day as u32)
            }
            _ => None,
        };
        date.map(Expr::Date).ok_or_else(|| parser_error(rule))
    }

    /// Evaluates a function that takes one date
    fn parse_date_part(pair: Pair<Rule>, part: impl FnOnce(NaiveDate) -> f64) -> Result<Expr> {
        let rule = pair.as_rule();
        let mut args = pair.into_inner();
        let date = Self::to_date(Self::get_formula(&mut args, rule)?, rule)?;
        Ok(Expr::Number(part(date)))
    }

    /// Evaluates a function that takes one time
    fn parse_time_part(pair: Pair<Rule>, part: impl FnOnce(NaiveTime) -> f64) -> Result<Expr> {
        let rule = pair.as_rule();
        let mut args = pair.into_inner();

        let time = match Self::get_formula(&mut args, rule)? {
            Expr::Time(time) => time,
            Expr::Datetime(datetime) => datetime.time(),
            Expr::String(time) => Self::timestring_to_naivetime(&time, rule)?,
            _ => return Err(parser_error(rule)),
        };
        Ok(Expr::Number(part(time)))
    }

    /// Accepts a date, a datetime or a `month/day/year` text
    fn to_date(date: Expr, rule: Rule) -> Result<NaiveDate> {
        match date {
            Expr::Date(date) => Ok(date),
            Expr::Datetime(datetime) => Ok(datetime.date_naive()),
            Expr::String(date) => Self::datestring_to_naivedate(&date, rule),
            _ => Err(parser_error(rule)),
        }
    }

    fn datestring_to_naivedate(datestring: &str, rule: Rule) -> Result<NaiveDate> {
        NaiveDate::parse_from_str(datestring, "%m/%d/%Y").map_err(|_| parser_error(rule))
    }

    /// Accepts `hour:minute:second` and `hour:minute`
    fn timestring_to_naivetime(timestring: &str, rule: Rule) -> Result<NaiveTime> {
        let timestring = timestring.trim_matches('\'').trim_matches('"');
        NaiveTime::parse_from_str(timestring, "%H:%M:%S")
            .or_else(|_| NaiveTime::parse_from_str(timestring, "%H:%M"))
            .map_err(|_| parser_error(rule))
    }

    /// Moves `date` by whole `months`, keeping the day within the target month like Excel
    fn shift_months(date: NaiveDate, months: f64) -> Option<NaiveDate> {
        let shift = Months::new(u32::try_from(months.trunc().abs() as i64).ok()?);
        if months < 0.0 {
            date.checked_sub_months(shift)
        } else {
            date.checked_add_months(shift)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Formula;

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_parse_date_and_time_types() {
        let formula = Formula::new("=DATE(2020,2,3)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Date(NaiveDate::from_ymd_opt(2020, 2, 3).unwrap()));

        let formula = Formula::new("=DATE(YEAR('1/30/2020'),MONTH('1/30/2020'),DAY('1/30/2020'))".trim()).unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Date(NaiveDate::from_ymd_opt(2020, 1, 30).unwrap()));

        let formula = Formula::new("=YEAR('1/30/2020')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(2020.0));

        let formula = Formula::new("=MONTH('1/30/2020')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(1.0));

        let formula = Formula::new("=DAY('1/30/2020')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(30.0));

        let formula = Formula::new("=DATEVALUE('1/30/2020')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Date(NaiveDate::from_ymd_opt(2020, 1, 30).unwrap()));

        let formula = Formula::new("=DAYS('3/30/2020', '1/30/2020')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(60.0));

        let formula = Formula::new("=DAYS(10, 5)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(5.0));

        let formula = Formula::new("=DAYS(DATE(2020,1,6), DATE(2020,1,1))").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(5.0));

        let formula = Formula::new("=EDATE('1/30/2020', 5)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Date(NaiveDate::from_ymd_opt(2020, 6, 30).unwrap()));

        let formula = Formula::new("=EDATE(DATE(2020,8,25),27)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Date(NaiveDate::from_ymd_opt(2022, 11, 25).unwrap()));

        let formula = Formula::new("=EOMONTH('1/20/2020', 5)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Date(NaiveDate::from_ymd_opt(2020, 6, 30).unwrap()));

        let formula = Formula::new("=EOMONTH(DATE(2020,8,31),27)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Date(NaiveDate::from_ymd_opt(2022, 11, 30).unwrap()));

        let formula = Formula::new("=EOMONTH(DATE(2020,12,15),0)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Date(NaiveDate::from_ymd_opt(2020, 12, 31).unwrap()));

        let formula = Formula::new("=EOMONTH(DATE(2020,1,15),-2)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Date(NaiveDate::from_ymd_opt(2019, 11, 30).unwrap()));

        let formula = Formula::new("=EDATE(DATE(2020,12,15),0)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Date(NaiveDate::from_ymd_opt(2020, 12, 15).unwrap()));

        let formula = Formula::new("=EDATE(DATE(2020,11,15),2)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Date(NaiveDate::from_ymd_opt(2021, 1, 15).unwrap()));

        let formula = Formula::new("=EDATE(DATE(2020,3,31),-1)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Date(NaiveDate::from_ymd_opt(2020, 2, 29).unwrap()));

        let formula = Formula::new("=HOUR('02:30:00')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(2.0));

        let formula = Formula::new("=MINUTE('02:30:00')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(30.0));

        let formula = Formula::new("=SECOND('02:30:00')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(0.0));

        let formula = Formula::new("=HOUR('02:30')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(2.0));

        let formula = Formula::new("=YEAR('2020')").unwrap();
        assert!(formula.parse().is_err());

        let formula = Formula::new("=DATEVALUE('1/2/2020/9')").unwrap();
        assert!(formula.parse().is_err());

        let formula = Formula::new("=YEAR(NOW())").unwrap();
        let value = formula.parse().unwrap();
        assert!(matches!(value, Expr::Number(_)));

        let formula = Formula::new("=ISOWEEKNUM('1/30/2020')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(5.0));

        let formula = Formula::new("=TIME(2,30,0)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Time(NaiveTime::from_hms_opt(2, 30, 0).unwrap()));

        let formula = Formula::new("=TIME(-1,30,0)").unwrap();
        assert!(formula.parse().is_err());

        let formula = Formula::new("=TIMEVALUE('02:30:00')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Time(NaiveTime::from_hms_opt(2, 30, 0).unwrap()));

        // 2020-01-01 is a Wednesday
        let formula = Formula::new("=WEEKDAY(DATE(2020,1,1))").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(4.0));

        let formula = Formula::new("=WEEKDAY(DATE(2020,1,1), 2)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(3.0));

        let formula = Formula::new("=WEEKDAY(DATE(2020,1,1), 3)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(2.0));

        let formula = Formula::new("=WEEKDAY(DATE(2020,1,1), 16)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(5.0));

        let formula = Formula::new("=WEEKDAY(DATE(2020,1,1), 4)").unwrap();
        assert!(formula.parse().is_err());

        let formula = Formula::new("=NOW()").unwrap();
        formula.parse().unwrap();

        let formula = Formula::new("=WEEKNUM(DATE(2020,1,1))").unwrap();
        assert!(matches!(formula.parse(), Err(Error::NotImplemented(_))));
    }
}
