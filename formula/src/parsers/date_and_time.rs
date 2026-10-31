use super::{Evaluator, Result};
use crate::{Expr, ExprError, Rule};
use chrono::{Datelike, Months, NaiveDate, NaiveTime, TimeDelta, Timelike, Utc, Weekday};
use pest::iterators::Pair;

/// Excel's limit for each argument of `TIME`
const MAX_TIME_PART: f64 = 32_767.0;

impl Evaluator {
    #[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps, clippy::unused_self)]
    pub(crate) fn parse_now(&self, _pair: Pair<Rule>) -> Result<Expr> {
        Ok(Expr::Datetime(Utc::now()))
    }

    #[allow(clippy::needless_pass_by_value, clippy::unnecessary_wraps, clippy::unused_self)]
    pub(crate) fn parse_today(&self, _pair: Pair<Rule>) -> Result<Expr> {
        Ok(Expr::Date(Utc::now().naive_utc().date()))
    }

    /// ref: <https://support.microsoft.com/en-us/office/weekday-function-60e44483-2ed1-439f-8bd0-e404c190949a>
    pub(crate) fn parse_weekday(&self, pair: Pair<Rule>) -> Result<Expr> {
        let mut args = pair.into_inner();
        let date = Self::to_date(self.get_formula(&mut args)?)?;
        let return_type = self.get_opt_formula_with_default(&mut args, Expr::Number(1.0))?;

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
                _ => return Ok(Expr::Error(ExprError::Number)),
            },
            _ => return Err(ExprError::Value),
        };
        Ok(Expr::Number(f64::from(weekday)))
    }

    // TODO a draft, `Formula::new` rejects it as not implemented yet
    // pub(crate) fn parse_weeknum(&self, pair: Pair<Rule>) -> Result<Expr> {
    //     let rule_name = format!("{:?}", &pair.as_rule());
    //     let mut args = pair.into_inner();
    //     let date = self.get_formula(&mut args, &rule_name)?;
    //     let return_type = args.next().map(|v| self.parse_pair(v)?).unwrap_or(Expr::Number(1.0));
    //
    //     let start_week = match return_type {
    //         Expr::Number(16.0) => Weekday::Sat,
    //         Expr::Number(1.0) | Expr::Number(17.0) => Weekday::Sun,
    //         Expr::Number(2.0) | Expr::Number(11.0) | Expr::Number(21.0)  => Weekday::Mon,
    //         Expr::Number(12.0)  => Weekday::Tue,
    //         Expr::Number(13.0)  => Weekday::Wed,
    //         Expr::Number(14.0)  => Weekday::Thu,
    //         Expr::Number(15.0)  => Weekday::Fri,
    //         _ => return Err(Error::Parser(rule_name).into()),
    //     };
    //     let week = match date {
    //         Expr::Date(date) =>  date.week(start_week),
    //         _ => return Err(Error::Parser(rule_name).into()),
    //     };
    //     Ok(Expr::Number(week as i32 as f64))
    // }

    pub(crate) fn parse_isoweeknum(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_date_part(pair, |date| f64::from(date.iso_week().week()))
    }

    pub(crate) fn parse_second(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_time_part(pair, |time| f64::from(time.second()))
    }

    pub(crate) fn parse_minute(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_time_part(pair, |time| f64::from(time.minute()))
    }

    pub(crate) fn parse_hour(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_time_part(pair, |time| f64::from(time.hour()))
    }

    pub(crate) fn parse_timevalue(&self, pair: Pair<Rule>) -> Result<Expr> {
        let mut args = pair.into_inner();

        match self.get_formula(&mut args)? {
            Expr::String(timestring) => Ok(Expr::Time(Self::timestring_to_naivetime(&timestring)?)),
            _ => Err(ExprError::Value),
        }
    }

    pub(crate) fn parse_datevalue(&self, pair: Pair<Rule>) -> Result<Expr> {
        let mut args = pair.into_inner();

        match self.get_formula(&mut args)? {
            Expr::String(datestring) => Ok(Expr::Date(Self::datestring_to_naivedate(&datestring)?)),
            _ => Err(ExprError::Value),
        }
    }

    pub(crate) fn parse_eomonth(&self, pair: Pair<Rule>) -> Result<Expr> {
        let mut args = pair.into_inner();
        let date = Self::to_date(self.get_formula(&mut args)?)?;
        let Expr::Number(num) = self.get_formula(&mut args)? else {
            return Err(ExprError::Value);
        };

        // the day before the first day of the next month
        let date = date
            .with_day(1)
            .and_then(|date| Self::shift_months(date, num))
            .and_then(|date| date.checked_add_months(Months::new(1)))
            .and_then(|date| date.pred_opt());
        Ok(date.map_or(Expr::Error(ExprError::Number), Expr::Date))
    }

    pub(crate) fn parse_edate(&self, pair: Pair<Rule>) -> Result<Expr> {
        let mut args = pair.into_inner();
        let date = Self::to_date(self.get_formula(&mut args)?)?;
        let Expr::Number(num) = self.get_formula(&mut args)? else {
            return Err(ExprError::Value);
        };

        Ok(Self::shift_months(date, num).map_or(Expr::Error(ExprError::Number), Expr::Date))
    }

    pub(crate) fn parse_days(&self, pair: Pair<Rule>) -> Result<Expr> {
        let mut args = pair.into_inner();
        let end_date = self.get_formula(&mut args)?;
        let start_date = self.get_formula(&mut args)?;

        let days = match (end_date, start_date) {
            (Expr::Number(end_date), Expr::Number(start_date)) => end_date - start_date,
            (end_date, start_date) => {
                let end_date = Self::to_date(end_date)?;
                let start_date = Self::to_date(start_date)?;
                end_date.signed_duration_since(start_date).num_days() as f64
            }
        };
        Ok(Expr::Number(days))
    }

    pub(crate) fn parse_day(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_date_part(pair, |date| f64::from(date.day()))
    }

    pub(crate) fn parse_month(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_date_part(pair, |date| f64::from(date.month()))
    }

    pub(crate) fn parse_year(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_date_part(pair, |date| f64::from(date.year()))
    }

    /// Like Excel, the parts overflow into each other (e.g. 90 minutes) and the time wraps around midnight
    pub(crate) fn parse_time(&self, pair: Pair<Rule>) -> Result<Expr> {
        let mut args = pair.into_inner();
        let hour = self.get_formula(&mut args)?;
        let minute = self.get_formula(&mut args)?;
        let second = self.get_formula(&mut args)?;

        let (Expr::Number(hour), Expr::Number(minute), Expr::Number(second)) = (hour, minute, second) else {
            return Err(ExprError::Value);
        };
        let seconds = hour.trunc() * 3600.0 + minute.trunc() * 60.0 + second.trunc();
        if seconds < 0.0 || [hour, minute, second].iter().any(|part| *part > MAX_TIME_PART) {
            return Ok(Expr::Error(ExprError::Number));
        }

        NaiveTime::from_num_seconds_from_midnight_opt((seconds % 86_400.0) as u32, 0)
            .map(Expr::Time)
            .ok_or(ExprError::Value)
    }

    /// Like Excel, months and days overflow into the next or previous year and month,
    /// e.g. `DATE(2020, 13, 1)` is 2021-01-01 and `DATE(2020, 1, 0)` is 2019-12-31
    pub(crate) fn parse_date(&self, pair: Pair<Rule>) -> Result<Expr> {
        let mut args = pair.into_inner();
        let year = self.get_formula(&mut args)?;
        let month = self.get_formula(&mut args)?;
        let day = self.get_formula(&mut args)?;

        let (Expr::Number(year), Expr::Number(month), Expr::Number(day)) = (year, month, day) else {
            return Err(ExprError::Value);
        };
        let date = i32::try_from(year.trunc() as i64)
            .ok()
            .and_then(|year| NaiveDate::from_ymd_opt(year, 1, 1))
            .and_then(|date| Self::shift_months(date, month.trunc() - 1.0))
            .and_then(|date| date.checked_add_signed(TimeDelta::try_days((day.trunc() as i64).saturating_sub(1))?));
        Ok(date.map_or(Expr::Error(ExprError::Number), Expr::Date))
    }

    /// Evaluates a function that takes one date
    fn parse_date_part(&self, pair: Pair<Rule>, part: impl FnOnce(NaiveDate) -> f64) -> Result<Expr> {
        let mut args = pair.into_inner();
        let date = Self::to_date(self.get_formula(&mut args)?)?;
        Ok(Expr::Number(part(date)))
    }

    /// Evaluates a function that takes one time
    fn parse_time_part(&self, pair: Pair<Rule>, part: impl FnOnce(NaiveTime) -> f64) -> Result<Expr> {
        let mut args = pair.into_inner();

        let time = match self.get_formula(&mut args)? {
            Expr::Time(time) => time,
            Expr::Datetime(datetime) => datetime.time(),
            Expr::String(time) => Self::timestring_to_naivetime(&time)?,
            _ => return Err(ExprError::Value),
        };
        Ok(Expr::Number(part(time)))
    }

    /// Accepts a date, a datetime or a `month/day/year` text
    fn to_date(date: Expr) -> Result<NaiveDate> {
        match date {
            Expr::Date(date) => Ok(date),
            Expr::Datetime(datetime) => Ok(datetime.date_naive()),
            Expr::String(date) => Self::datestring_to_naivedate(&date),
            _ => Err(ExprError::Value),
        }
    }

    fn datestring_to_naivedate(datestring: &str) -> Result<NaiveDate> {
        NaiveDate::parse_from_str(datestring, "%m/%d/%Y").map_err(|_| ExprError::Value)
    }

    /// Accepts `hour:minute:second` and `hour:minute`
    fn timestring_to_naivetime(timestring: &str) -> Result<NaiveTime> {
        let timestring = timestring.trim_matches('\'').trim_matches('"');
        NaiveTime::parse_from_str(timestring, "%H:%M:%S")
            .or_else(|_| NaiveTime::parse_from_str(timestring, "%H:%M"))
            .map_err(|_| ExprError::Value)
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
        assert_eq!(formula.parse().unwrap(), Expr::Error(ExprError::Value));

        let formula = Formula::new("=DATEVALUE('1/2/2020/9')").unwrap();
        assert_eq!(formula.parse().unwrap(), Expr::Error(ExprError::Value));

        let formula = Formula::new("=YEAR(NOW())").unwrap();
        let value = formula.parse().unwrap();
        assert!(matches!(value, Expr::Number(_)));

        let formula = Formula::new("=ISOWEEKNUM('1/30/2020')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(5.0));

        let formula = Formula::new("=TIME(2,30,0)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Time(NaiveTime::from_hms_opt(2, 30, 0).unwrap()));

        let formula = Formula::new("=TIME(25,0,0)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Time(NaiveTime::from_hms_opt(1, 0, 0).unwrap()));

        let formula = Formula::new("=TIME(1,90,0)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Time(NaiveTime::from_hms_opt(2, 30, 0).unwrap()));

        let formula = Formula::new("=TIME(1,-30,0.9)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Time(NaiveTime::from_hms_opt(0, 30, 0).unwrap()));

        let formula = Formula::new("=TIME(-1,30,0)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Error(ExprError::Number));

        let formula = Formula::new("=TIME(32768,0,0)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Error(ExprError::Number));

        let formula = Formula::new("=TIME('1',0,0)").unwrap();
        assert_eq!(formula.parse().unwrap(), Expr::Error(ExprError::Value));

        let formula = Formula::new("=DATE(2020,13,1)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Date(NaiveDate::from_ymd_opt(2021, 1, 1).unwrap()));

        let formula = Formula::new("=DATE(2020,0,1)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Date(NaiveDate::from_ymd_opt(2019, 12, 1).unwrap()));

        let formula = Formula::new("=DATE(2020,-13,1)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Date(NaiveDate::from_ymd_opt(2018, 11, 1).unwrap()));

        let formula = Formula::new("=DATE(2020,1,0)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Date(NaiveDate::from_ymd_opt(2019, 12, 31).unwrap()));

        let formula = Formula::new("=DATE(2020,3,-1)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Date(NaiveDate::from_ymd_opt(2020, 2, 28).unwrap()));

        let formula = Formula::new("=DATE(2020,2,30)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Date(NaiveDate::from_ymd_opt(2020, 3, 1).unwrap()));

        let formula = Formula::new("=DATE(2020.9,0.5,1.9)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Date(NaiveDate::from_ymd_opt(2019, 12, 1).unwrap()));

        let formula = Formula::new("=DATE(1e10,1,1)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Error(ExprError::Number));

        let formula = Formula::new("=DATE(2020,1,1e300)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Error(ExprError::Number));

        let formula = Formula::new("=DATE(2020,1,-1e300)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Error(ExprError::Number));

        let formula = Formula::new("=DATE('2020',1,1)").unwrap();
        assert_eq!(formula.parse().unwrap(), Expr::Error(ExprError::Value));

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
        assert_eq!(formula.parse().unwrap(), Expr::Error(ExprError::Number));

        let formula = Formula::new("=NOW()").unwrap();
        formula.parse().unwrap();

        let formula = Formula::new("=WEEKNUM(DATE(2020,1,1))");
        assert!(matches!(formula, Err(crate::error::Error::NotImplemented(_))));

        let formula = Formula::new("=EDATE(DATE(2020,1,15),-1)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Date(NaiveDate::from_ymd_opt(2019, 12, 15).unwrap()));

        let formula = Formula::new("=EDATE(DATE(2020,1,31),1.9)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Date(NaiveDate::from_ymd_opt(2020, 2, 29).unwrap()));

        let formula = Formula::new("=EOMONTH(DATE(2020,1,15),-0.5)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Date(NaiveDate::from_ymd_opt(2020, 1, 31).unwrap()));

        let formula = Formula::new("=EDATE(DATE(2020,1,15),1e20)").unwrap();
        assert_eq!(formula.parse().unwrap(), Expr::Error(ExprError::Number));

        let formula = Formula::new("=EOMONTH(DATE(2020,1,15),-1e20)").unwrap();
        assert_eq!(formula.parse().unwrap(), Expr::Error(ExprError::Number));

        let formula = Formula::new("=EDATE(DATE(2020,1,15),'1')").unwrap();
        assert_eq!(formula.parse().unwrap(), Expr::Error(ExprError::Value));

        let formula = Formula::new("=EOMONTH(DATE(2020,1,15),TRUE)").unwrap();
        assert_eq!(formula.parse().unwrap(), Expr::Error(ExprError::Value));

        let formula = Formula::new("=TIMEVALUE('25:00')").unwrap();
        assert_eq!(formula.parse().unwrap(), Expr::Error(ExprError::Value));

        let formula = Formula::new("=DATEVALUE('13/1/2020')").unwrap();
        assert_eq!(formula.parse().unwrap(), Expr::Error(ExprError::Value));

        let formula = Formula::new("=WEEKDAY('1/1/2020')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(4.0));

        let formula = Formula::new("=DAYS(DATE(2020,1,6), '1/1/2020')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(5.0));

        let formula = Formula::new("=DAYS(DATE(2020,1,6), TRUE)").unwrap();
        assert_eq!(formula.parse().unwrap(), Expr::Error(ExprError::Value));

        let formula = Formula::new("=WEEKDAY(DATE(2020,1,1), 'a')").unwrap();
        assert_eq!(formula.parse().unwrap(), Expr::Error(ExprError::Value));

        let formula = Formula::new("=MONTH(TODAY())").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(f64::from(Utc::now().month())));
    }
}
