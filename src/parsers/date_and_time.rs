use crate::{error::Error, Expr, Formula, Rule};
use anyhow::Result;
use chrono::{Datelike, NaiveDate, NaiveTime, Timelike, Utc};
use chronoutil::shift_months;
use pest::iterators::Pair;

impl Formula<'_> {
    pub(crate) fn parse_now() -> Expr {
        Expr::Datetime(Utc::now())
    }

    pub(crate) fn parse_today() -> Expr {
        Expr::Date(Utc::now().naive_utc().date())
    }

    pub(crate) fn parse_weekday(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let date = Self::get_formula(&mut args, &rule_name)?;
        // TODO support return type, ref: https://support.microsoft.com/en-us/office/weekday-function-60e44483-2ed1-439f-8bd0-e404c190949a
        // let return_type = Self::get_opt_formula_with_default(&mut args,Expr::Number(1.0))?;

        let weekday = match date {
            Expr::Date(date) => date.weekday(),
            _ => return Err(Error::Parser(rule_name).into()),
        };
        Ok(Expr::Number(f64::from(weekday as i32)))
    }

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
        todo!("Not implemented: {:?}", pair.as_rule())
    }

    pub(crate) fn parse_isoweeknum(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let arg = Self::get_formula(&mut args, &rule_name)?;
        let second = match arg {
            Expr::Date(date) => date.iso_week().week(),
            Expr::String(date) => Self::datestring_to_naivedate(&date)?.iso_week().week(),
            _ => return Err(Error::Parser(rule_name).into()),
        };
        Ok(Expr::Number(f64::from(second)))
    }

    pub(crate) fn parse_second(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let arg = Self::get_formula(&mut args, &rule_name)?;
        let second = match arg {
            Expr::Time(time) => time.second(),
            Expr::String(time) => Self::timestring_to_naivetime(&time)?.second(),
            _ => return Err(Error::Parser(rule_name).into()),
        };
        Ok(Expr::Number(f64::from(second)))
    }

    pub(crate) fn parse_minute(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let arg = Self::get_formula(&mut args, &rule_name)?;
        let minute = match arg {
            Expr::Time(time) => time.minute(),
            Expr::String(time) => Self::timestring_to_naivetime(&time)?.minute(),
            _ => return Err(Error::Parser(rule_name).into()),
        };
        Ok(Expr::Number(f64::from(minute)))
    }

    pub(crate) fn parse_hour(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let arg = Self::get_formula(&mut args, &rule_name)?;
        let hour = match arg {
            Expr::Time(time) => time.hour(),
            Expr::String(time) => Self::timestring_to_naivetime(&time)?.hour(),
            _ => return Err(Error::Parser(rule_name).into()),
        };
        Ok(Expr::Number(f64::from(hour)))
    }

    pub(crate) fn parse_timevalue(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let arg = Self::get_formula(&mut args, &rule_name)?;
        let time = match arg {
            Expr::String(datestring) => Self::timestring_to_naivetime(&datestring)?,
            _ => return Err(Error::Parser(rule_name).into()),
        };
        Ok(Expr::Time(time))
    }

    pub(crate) fn parse_datevalue(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let arg = Self::get_formula(&mut args, &rule_name)?;
        let date = match arg {
            Expr::String(datestring) => Self::datestring_to_naivedate(&datestring)?,
            _ => return Err(Error::Parser(rule_name).into()),
        };
        Ok(Expr::Date(date))
    }

    pub(crate) fn parse_eomonth(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let date = Self::get_formula(&mut args, &rule_name)?;
        let num = Self::get_formula(&mut args, &rule_name)?;
        let date = match (date, num) {
            (Expr::String(date), Expr::Number(num)) => {
                let date = Self::datestring_to_naivedate(&date)?;
                let date = shift_months(date, num as i32);
                let last_day = Self::last_day_of_month(date.year(), date.month());
                NaiveDate::from_ymd(date.year(), date.month(), last_day)
            }
            (Expr::Date(date), Expr::Number(num)) => {
                let date = shift_months(date, num as i32);
                let last_day = Self::last_day_of_month(date.year(), date.month());
                NaiveDate::from_ymd(date.year(), date.month(), last_day)
            }
            _ => return Err(Error::Parser(rule_name).into()),
        };

        Ok(Expr::Date(date))
    }

    pub(crate) fn parse_edate(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let date = Self::get_formula(&mut args, &rule_name)?;
        let num = Self::get_formula(&mut args, &rule_name)?;
        let date = match (date, num) {
            (Expr::String(date), Expr::Number(num)) => {
                let date = Self::datestring_to_naivedate(&date)?;
                shift_months(date, num as i32)
            }
            (Expr::Date(date), Expr::Number(num)) => shift_months(date, num as i32),
            _ => return Err(Error::Parser(rule_name).into()),
        };

        Ok(Expr::Date(date))
    }

    pub(crate) fn parse_days(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let end_date = Self::get_formula(&mut args, &rule_name)?;
        let start_date = Self::get_formula(&mut args, &rule_name)?;
        let days = match (end_date, start_date) {
            (Expr::Date(end_date), Expr::Date(start_date)) => {
                end_date.signed_duration_since(start_date).num_days() as f64
            }
            (Expr::String(end_date), Expr::String(start_date)) => {
                let end_date = Self::datestring_to_naivedate(&end_date)?;
                let start_date = Self::datestring_to_naivedate(&start_date)?;
                end_date.signed_duration_since(start_date).num_days() as f64
            }
            (Expr::Number(end_date), Expr::Number(start_date)) => end_date - start_date,
            _ => return Err(Error::Parser(rule_name).into()),
        };

        Ok(Expr::Number(days as f64))
    }

    pub(crate) fn parse_day(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let arg = Self::get_formula(&mut args, &rule_name)?;
        let day = match arg {
            Expr::Date(date) => date.day(),
            Expr::String(date) => Self::datestring_to_naivedate(&date)?.day(),
            _ => return Err(Error::Parser(rule_name).into()),
        };
        Ok(Expr::Number(f64::from(day)))
    }

    pub(crate) fn parse_month(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let arg = Self::get_formula(&mut args, &rule_name)?;
        let month = match arg {
            Expr::Date(date) => date.month(),
            Expr::String(date) => Self::datestring_to_naivedate(&date)?.month(),
            _ => return Err(Error::Parser(rule_name).into()),
        };
        Ok(Expr::Number(f64::from(month)))
    }

    pub(crate) fn parse_year(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let arg = Self::get_formula(&mut args, &rule_name)?;
        let year = match arg {
            Expr::Date(date) => date.year(),
            Expr::String(date) => Self::datestring_to_naivedate(&date)?.year(),
            _ => return Err(Error::Parser(rule_name).into()),
        };
        Ok(Expr::Number(f64::from(year)))
    }

    pub(crate) fn parse_time(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let hour = Self::get_formula(&mut args, &rule_name)?;
        let minute = Self::get_formula(&mut args, &rule_name)?;
        let second = Self::get_formula(&mut args, &rule_name)?;
        let time = match (hour, minute, second) {
            (Expr::Number(hour), Expr::Number(minute), Expr::Number(second)) => {
                Expr::Time(NaiveTime::from_hms(hour as u32, minute as u32, second as u32))
            }
            _ => return Err(Error::Parser(rule_name).into()),
        };
        Ok(time)
    }

    pub(crate) fn parse_date(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let year = Self::get_formula(&mut args, &rule_name)?;
        let month = Self::get_formula(&mut args, &rule_name)?;
        let day = Self::get_formula(&mut args, &rule_name)?;
        let date = match (year, month, day) {
            (Expr::Number(year), Expr::Number(month), Expr::Number(day)) => {
                Expr::Date(NaiveDate::from_ymd(year as i32, month as u32, day as u32))
            }
            _ => return Err(Error::Parser(rule_name).into()),
        };
        Ok(date)
    }

    // fn Self::datestring_to_naivedate(datestring: &str) -> NaiveDate {
    //     let datestring = datestring.trim().trim_matches('\'').trim_matches('"');
    //
    //     NaiveDate::parse_from_str(datestring, "%m/%d/%Y")
    //         .unwrap_or(NaiveDate::parse_from_str(datestring, "%v").ok_or_else(|| Error::Parser(rule_name.clone()))?)
    // }
    //
    // fn Self::timestring_to_naivetime(timestring: &str) -> NaiveTime {
    //     let timestring = timestring.trim().trim_matches('\'').trim_matches('"');
    //     NaiveTime::parse_from_str(timestring, "%T")
    //         .unwrap_or(NaiveTime::parse_from_str(timestring, "%r").ok_or_else(|| Error::Parser(rule_name.clone()))?)
    // }

    fn datestring_to_naivedate(datestring: &str) -> Result<NaiveDate> {
        let mdy = datestring
            .split('/')
            .map(str::parse)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(NaiveDate::from_ymd(mdy[2] as i32, mdy[0], mdy[1]))
    }

    fn timestring_to_naivetime(timestring: &str) -> Result<NaiveTime> {
        let hms = timestring
            .trim_matches('\'')
            .trim_matches('"')
            .split(':')
            .map(str::parse)
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(NaiveTime::from_hms(hms[0], hms[1], hms[2]))
    }

    fn last_day_of_month(year: i32, month: u32) -> u32 {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
            .unwrap_or_else(|| NaiveDate::from_ymd(year + 1, 1, 1))
            .pred()
            .day()
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
        assert_eq!(value, Expr::Date(NaiveDate::from_ymd(2020, 2, 3)));

        let formula = Formula::new("=DATE(YEAR('1/30/2020'),MONTH('1/30/2020'),DAY('1/30/2020'))".trim()).unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Date(NaiveDate::from_ymd(2020, 1, 30)));

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
        assert_eq!(value, Expr::Date(NaiveDate::from_ymd(2020, 1, 30)));

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
        assert_eq!(value, Expr::Date(NaiveDate::from_ymd(2020, 6, 30)));

        let formula = Formula::new("=EOMONTH('1/20/2020', 5)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Date(NaiveDate::from_ymd(2020, 6, 30)));

        let formula = Formula::new("=HOUR('02:30:00')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(2.0));

        let formula = Formula::new("=MINUTE('02:30:00')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(30.0));

        let formula = Formula::new("=SECOND('02:30:00')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(0.0));

        let formula = Formula::new("=ISOWEEKNUM('1/30/2020')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(5.0));

        let formula = Formula::new("=TIME(2,30,0)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Time(NaiveTime::from_hms(2, 30, 0)));

        let formula = Formula::new("=TIMEVALUE('02:30:00')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Time(NaiveTime::from_hms(2, 30, 0)));

        let formula = Formula::new("=WEEKDAY(DATE(2020,1,1))").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(2.0));

        // let formula = Formula::new("=WEEKDAY(DATE(2020,1,1), 1)").unwrap();
        // let value = formula.parse().unwrap();
        // assert_eq!(value, Expr::Number(5.0));

        // let formula = Formula::new("=WEEKDAY(DATE(2020,1,1), 3)").unwrap();
        // let value = formula.parse().unwrap();
        // assert_eq!(value, Expr::Number(5.0));

        // let formula = Formula::new("=WEEKNUM(DATE(2020,1,1))").unwrap();
        // let value = formula.parse().unwrap();
        // assert_eq!(value, Expr::Number(2.0));
    }
}
