use super::{
    Evaluator, Result,
    helper::{ArgValue, as_text, number_to_text},
};
use crate::{Expr, ExprError, Rule};
use pest::iterators::Pair;

/// Excel's maximum text length
const MAX_TEXT_LEN: usize = 32_767;
/// Excel's maximum number of decimals in `FIXED`
const MAX_FIXED_DECIMALS: f64 = 127.0;

impl Evaluator<'_> {
    pub(crate) fn parse_left(&self, pair: Pair<Rule>) -> Result<Expr> {
        let mut args = pair.into_inner();
        let text = self.get_text(&mut args)?;
        let chars = self.get_opt_number(&mut args, 1.0)?;

        if chars >= 0.0 {
            Ok(Expr::String(text.chars().take(chars as usize).collect()))
        } else {
            Err(ExprError::Value)
        }
    }

    pub(crate) fn parse_leftb(&self, pair: Pair<Rule>) -> Result<Expr> {
        let mut args = pair.into_inner();
        let text = self.get_text(&mut args)?;
        let bytes = self.get_opt_number(&mut args, 1.0)?;

        if bytes >= 0.0 {
            let end = (bytes as usize).min(text.len());
            Ok(Expr::String(
                String::from_utf8_lossy(&text.as_bytes()[..end]).into_owned(),
            ))
        } else {
            Err(ExprError::Value)
        }
    }

    pub(crate) fn parse_right(&self, pair: Pair<Rule>) -> Result<Expr> {
        let mut args = pair.into_inner();
        let text = self.get_text(&mut args)?;
        let chars = self.get_opt_number(&mut args, 1.0)?;

        if chars >= 0.0 {
            let start = text.chars().count().saturating_sub(chars as usize);
            Ok(Expr::String(text.chars().skip(start).collect()))
        } else {
            Err(ExprError::Value)
        }
    }

    pub(crate) fn parse_rightb(&self, pair: Pair<Rule>) -> Result<Expr> {
        let mut args = pair.into_inner();
        let text = self.get_text(&mut args)?;
        let bytes = self.get_opt_number(&mut args, 1.0)?;

        if bytes >= 0.0 {
            let start = text.len().saturating_sub(bytes as usize);
            Ok(Expr::String(
                String::from_utf8_lossy(&text.as_bytes()[start..]).into_owned(),
            ))
        } else {
            Err(ExprError::Value)
        }
    }

    pub(crate) fn parse_mid(&self, pair: Pair<Rule>) -> Result<Expr> {
        let mut args = pair.into_inner();
        let text = self.get_text(&mut args)?;
        let start = self.get_number(&mut args)?;
        let len = self.get_number(&mut args)?;

        if start >= 1.0 && len >= 0.0 {
            Ok(Expr::String(
                text.chars().skip(start as usize - 1).take(len as usize).collect(),
            ))
        } else {
            Err(ExprError::Value)
        }
    }

    pub(crate) fn parse_midb(&self, pair: Pair<Rule>) -> Result<Expr> {
        let mut args = pair.into_inner();
        let text = self.get_text(&mut args)?;
        let start = self.get_number(&mut args)?;
        let len = self.get_number(&mut args)?;

        if start >= 1.0 && len >= 0.0 {
            let start = (start as usize - 1).min(text.len());
            let end = start.saturating_add(len as usize).min(text.len());
            Ok(Expr::String(
                String::from_utf8_lossy(&text.as_bytes()[start..end]).into_owned(),
            ))
        } else {
            Err(ExprError::Value)
        }
    }

    /// `CHAR` covers the single byte range, `UNICHAR` covers every code point
    pub(crate) fn parse_char(&self, pair: Pair<Rule>) -> Result<Expr> {
        let max = if pair.as_rule() == Rule::char {
            255.0
        } else {
            f64::from(u32::from(char::MAX))
        };
        let mut args = pair.into_inner();

        match self.get_number(&mut args)? {
            number if (1.0..=max).contains(&number) => char::from_u32(number as u32)
                .map(|c| Expr::String(c.to_string()))
                .ok_or(ExprError::Value),
            _ => Err(ExprError::Value),
        }
    }

    pub(crate) fn parse_code(&self, pair: Pair<Rule>) -> Result<Expr> {
        let mut args = pair.into_inner();

        self.get_text(&mut args)?
            .chars()
            .next()
            .map(|c| Expr::Number(f64::from(u32::from(c))))
            .ok_or(ExprError::Value)
    }

    pub(crate) fn parse_concat(&self, pair: Pair<Rule>) -> Result<Expr> {
        let texts = self
            .range_args(pair.into_inner())?
            .into_iter()
            .map(Self::range_text)
            .collect::<Result<Vec<_>>>()?;
        Self::long_text(texts.concat())
    }

    pub(crate) fn parse_exact(&self, pair: Pair<Rule>) -> Result<Expr> {
        let mut args = pair.into_inner();
        let text1 = self.get_text(&mut args)?;
        let text2 = self.get_text(&mut args)?;

        Ok(Expr::Bool(text1 == text2))
    }

    pub(crate) fn parse_find(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_find_text(pair, false, Self::find_chars)
    }

    pub(crate) fn parse_findb(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_find_text(pair, false, Self::find_bytes)
    }

    pub(crate) fn parse_search(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_find_text(pair, true, Self::find_chars)
    }

    pub(crate) fn parse_searchb(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_find_text(pair, true, Self::find_bytes)
    }

    pub(crate) fn parse_fixed(&self, pair: Pair<Rule>) -> Result<Expr> {
        let mut args = pair.into_inner();
        let number = self.get_number(&mut args)?;
        let decimals = self.get_opt_number(&mut args, 2.0)?;
        let no_commas = self.get_opt_bool(&mut args, false)?;

        let decimals = match decimals {
            decimals if decimals <= MAX_FIXED_DECIMALS => decimals.trunc().max(-308.0),
            _ => return Err(ExprError::Value),
        };

        // Excel rounds half away from zero, Rust's formatter doesn't
        let factor = 10f64.powf(decimals);
        let rounded = (number * factor).round() / factor;
        let number = match rounded {
            0.0 => 0.0, // avoid "-0"
            rounded if rounded.is_finite() => rounded,
            _ => number,
        };

        let text = format!("{number:.0$}", decimals.max(0.0) as usize);
        Ok(Expr::String(if no_commas {
            text
        } else {
            Self::group_thousands(&text)
        }))
    }

    pub(crate) fn parse_len(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_unary_text(pair, |text| Expr::Number(text.chars().count() as f64))
    }

    pub(crate) fn parse_lenb(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_unary_text(pair, |text| Expr::Number(text.len() as f64))
    }

    pub(crate) fn parse_lower(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_unary_text(pair, |text| Expr::String(text.to_lowercase()))
    }

    pub(crate) fn parse_upper(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_unary_text(pair, |text| Expr::String(text.to_uppercase()))
    }

    pub(crate) fn parse_rept(&self, pair: Pair<Rule>) -> Result<Expr> {
        let mut args = pair.into_inner();
        let text = self.get_text(&mut args)?;
        let rept = self.get_number(&mut args)?;

        if rept >= 0.0 && text.chars().count().saturating_mul(rept as usize) <= MAX_TEXT_LEN {
            Ok(Expr::String(text.repeat(rept as usize)))
        } else {
            Err(ExprError::Value)
        }
    }

    pub(crate) fn parse_replace(&self, pair: Pair<Rule>) -> Result<Expr> {
        let mut args = pair.into_inner();
        let text = self.get_text(&mut args)?;
        let start = self.get_number(&mut args)?;
        let len = self.get_number(&mut args)?;
        let new_text = self.get_text(&mut args)?;

        if start >= 1.0 && len >= 0.0 {
            let mut chars = text.chars();
            let start_text = chars.by_ref().take(start as usize - 1).collect::<String>();
            let end_text = chars.skip(len as usize).collect::<String>();
            Ok(Expr::String(format!("{start_text}{new_text}{end_text}")))
        } else {
            Err(ExprError::Value)
        }
    }

    pub(crate) fn parse_replaceb(&self, pair: Pair<Rule>) -> Result<Expr> {
        let mut args = pair.into_inner();
        let text = self.get_text(&mut args)?;
        let start = self.get_number(&mut args)?;
        let len = self.get_number(&mut args)?;
        let new_text = self.get_text(&mut args)?;

        if start >= 1.0 && len >= 0.0 {
            let bytes = text.as_bytes();
            let start = (start as usize - 1).min(bytes.len());
            let end = start.saturating_add(len as usize).min(bytes.len());
            Ok(Expr::String(format!(
                "{}{}{}",
                String::from_utf8_lossy(&bytes[..start]),
                new_text,
                String::from_utf8_lossy(&bytes[end..])
            )))
        } else {
            Err(ExprError::Value)
        }
    }

    pub(crate) fn parse_textjoin(&self, pair: Pair<Rule>) -> Result<Expr> {
        let mut args = pair.into_inner();
        let delim = self.get_text(&mut args)?;
        let ignore_empty = self.get_bool(&mut args)?;

        let texts = self
            .range_args(args)?
            .into_iter()
            .map(Self::range_text)
            .filter(|t| !(ignore_empty && t.as_ref().is_ok_and(String::is_empty)))
            .collect::<Result<Vec<_>>>()?;
        Self::long_text(texts.join(&delim))
    }

    /// Like Excel, only spaces are removed and the inner runs of spaces become one space
    pub(crate) fn parse_trim(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_unary_text(pair, |text| {
            Expr::String(
                text.split(' ')
                    .filter(|word| !word.is_empty())
                    .collect::<Vec<_>>()
                    .join(" "),
            )
        })
    }

    pub(crate) fn parse_t(&self, pair: Pair<Rule>) -> Result<Expr> {
        let mut args = pair.into_inner();
        let text = match self.get_formula(&mut args)? {
            Expr::String(text) => text,
            _ => String::new(),
        };
        Ok(Expr::String(text))
    }

    /// Like Excel, the first letter and every letter that follows a non-letter are capitalized
    pub(crate) fn parse_proper(&self, pair: Pair<Rule>) -> Result<Expr> {
        self.parse_unary_text(pair, |text| {
            let mut proper = String::with_capacity(text.len());
            let mut after_letter = false;
            for c in text.chars() {
                if after_letter {
                    proper.extend(c.to_lowercase());
                } else {
                    proper.extend(c.to_uppercase());
                }
                after_letter = c.is_alphabetic();
            }
            Expr::String(proper)
        })
    }

    /// Like Excel, the numbers and booleans of references and arrays become texts
    fn range_text(value: ArgValue) -> Result<String> {
        match value {
            ArgValue::InRange(Expr::Number(number)) => Ok(number_to_text(number)),
            ArgValue::InRange(Expr::Bool(bool)) => Ok(if bool { "TRUE" } else { "FALSE" }.to_string()),
            ArgValue::Single(value) | ArgValue::InRange(value) => as_text(value),
        }
    }

    /// Like Excel, texts longer than 32,767 characters are `#VALUE!`
    fn long_text(text: String) -> Result<Expr> {
        if text.chars().count() <= MAX_TEXT_LEN {
            Ok(Expr::String(text))
        } else {
            Err(ExprError::Value)
        }
    }

    /// Evaluates a function that takes one text
    fn parse_unary_text(&self, pair: Pair<Rule>, f: impl FnOnce(&str) -> Expr) -> Result<Expr> {
        let mut args = pair.into_inner();
        let text = self.get_text(&mut args)?;

        Ok(f(&text))
    }

    fn parse_find_text(
        &self,
        pair: Pair<Rule>,
        ignore_case: bool,
        find: fn(&str, &str, f64) -> Option<f64>,
    ) -> Result<Expr> {
        let mut args = pair.into_inner();
        let find_text = self.get_text(&mut args)?;
        let within_text = self.get_text(&mut args)?;
        let start_num = self.get_opt_number(&mut args, 1.0)?;

        let index = if ignore_case {
            find(&within_text.to_lowercase(), &find_text.to_lowercase(), start_num)
        } else {
            find(&within_text, &find_text, start_num)
        };
        index.map(Expr::Number).ok_or(ExprError::Value)
    }

    /// 1-based character position of `needle`, searching from the `start_num`th character
    fn find_chars(haystack: &str, needle: &str, start_num: f64) -> Option<f64> {
        if start_num < 1.0 {
            return None;
        }
        let (start, _) = haystack.char_indices().nth(start_num as usize - 1)?;
        let index = start + haystack[start..].find(needle)?;
        Some((haystack[..index].chars().count() + 1) as f64)
    }

    /// 1-based byte position of `needle`, searching from the `start_num`th byte
    fn find_bytes(haystack: &str, needle: &str, start_num: f64) -> Option<f64> {
        if start_num < 1.0 || start_num > haystack.len() as f64 {
            return None;
        }
        let start = start_num as usize - 1;
        let index = start + haystack.get(start..)?.find(needle)?;
        Some((index + 1) as f64)
    }

    /// Adds thousands separators to a formatted number, e.g. `-1234.5` to `-1,234.5`
    fn group_thousands(number: &str) -> String {
        let (sign, number) = number.strip_prefix('-').map_or(("", number), |number| ("-", number));
        let (integer, fraction) = number.split_once('.').map_or((number, None), |(i, f)| (i, Some(f)));

        let mut text = String::with_capacity(sign.len() + number.len() + integer.len() / 3);
        text.push_str(sign);
        for (i, c) in integer.chars().enumerate() {
            if i != 0 && (integer.len() - i) % 3 == 0 {
                text.push(',');
            }
            text.push(c);
        }
        if let Some(fraction) = fraction {
            text.push('.');
            text.push_str(fraction);
        }
        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Formula;

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_parse_text_types() {
        let formula = Formula::new("=LEFT('hello')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("h".to_string()));

        let formula = Formula::new("=LEFT('hello', 2)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("he".to_string()));

        let formula = Formula::new("=LEFTB('hello')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("h".to_string()));

        let formula = Formula::new("=RIGHT('hello')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("o".to_string()));

        let formula = Formula::new("=RIGHT('hello', 2)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("lo".to_string()));

        let formula = Formula::new("=RIGHTB('hello')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("o".to_string()));

        let formula = Formula::new("=RIGHT('hi', 5)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("hi".to_string()));

        let formula = Formula::new("=RIGHTB('hi', 5)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("hi".to_string()));

        let formula = Formula::new("=MID('Fluid Flow', 7, 20)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("Flow".to_string()));

        let formula = Formula::new("=MID('Fluid Flow', 20, 5)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String(String::new()));

        let formula = Formula::new("=MIDB('Fluid Flow', 7, 20)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("Flow".to_string()));

        let formula = Formula::new("=MID('abc', 0, 1)").unwrap();
        assert_eq!(formula.parse().unwrap(), Expr::Error(ExprError::Value));

        let formula = Formula::new("=MIDB('abc', 0.5, 1)").unwrap();
        assert_eq!(formula.parse().unwrap(), Expr::Error(ExprError::Value));

        let formula = Formula::new("=CODE('A')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(65.0));

        let formula = Formula::new("=CHAR(65)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("A".to_string()));

        let formula = Formula::new("=CHAR(256)").unwrap();
        assert_eq!(formula.parse().unwrap(), Expr::Error(ExprError::Value));

        let formula = Formula::new("=UNICHAR(1575)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("ا".to_string()));

        let formula = Formula::new("=UNICODE('€')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(8364.0));

        let formula = Formula::new("=CONCAT('hello', ' ', 'world')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("hello world".to_string()));

        let formula = Formula::new("=CONCATENATE('hello', ' ', 'world')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("hello world".to_string()));

        let formula = Formula::new("=EXACT('hello', 'hello')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Bool(true));

        let formula = Formula::new("=EXACT('hello', 'Hello')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Bool(false));

        let formula = Formula::new("=FIND('a', 'abca')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(1.0));

        let formula = Formula::new("=FIND('a', 'abca', 2)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(4.0));

        let formula = Formula::new("=SEARCH('a', 'ABCA')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(1.0));

        let formula = Formula::new("=SEARCH('a', 'ABCA', 2)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(4.0));

        let formula = Formula::new("=FIND('b', 'éb')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(2.0));

        let formula = Formula::new("=SEARCH('B', 'éb')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(2.0));

        let formula = Formula::new("=FINDB('b', 'éb')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(3.0));

        let formula = Formula::new("=FINDB('a', 'éa', 2)").unwrap();
        assert_eq!(formula.parse().unwrap(), Expr::Error(ExprError::Value));

        let formula = Formula::new("=SEARCHB('a', 'éa', 2)").unwrap();
        assert_eq!(formula.parse().unwrap(), Expr::Error(ExprError::Value));

        let formula = Formula::new("=FIND('z', 'abc')").unwrap();
        assert_eq!(formula.parse().unwrap(), Expr::Error(ExprError::Value));

        let formula = Formula::new("=FIXED(123456.673, 2)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("123,456.67".to_string()));

        let formula = Formula::new("=FIXED(123456.673, -2)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("123,500".to_string()));

        let formula = Formula::new("=FIXED(-123456, 0)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("-123,456".to_string()));

        let formula = Formula::new("=FIXED(2.5, 0)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("3".to_string()));

        let formula = Formula::new("=FIXED(-0.4, 0)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("0".to_string()));

        let formula = Formula::new("=FIXED(1234.5, 1e10)").unwrap();
        assert_eq!(formula.parse().unwrap(), Expr::Error(ExprError::Value));

        let formula = Formula::new("=FIXED(123456.673, 0)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("123,457".to_string()));

        let formula = Formula::new("=FIXED(123456.673, 2, true)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("123456.67".to_string()));

        let formula = Formula::new("=LEN('hello')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(5.0));

        let formula = Formula::new("=LEN('سلام')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(4.0));

        let formula = Formula::new("=LENB('سلام')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(8.0));

        let formula = Formula::new("=LOWER('Hello')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("hello".to_string()));

        let formula = Formula::new("=UPPER('Hello')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("HELLO".to_string()));

        let formula = Formula::new("=REPT('H', 5)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("HHHHH".to_string()));

        let formula = Formula::new("=REPT('H', 0)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String(String::new()));

        let formula = Formula::new("=REPT('a', 1e18)").unwrap();
        assert_eq!(formula.parse().unwrap(), Expr::Error(ExprError::Value));

        let formula = Formula::new("=REPLACE('abcdefghijk', 6, 5, '*')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("abcde*k".to_string()));

        let formula = Formula::new("=REPLACEB('123456', 1, 3, '@')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("@456".to_string()));

        let formula = Formula::new("=REPLACE('abc', 0, 1, 'x')").unwrap();
        assert_eq!(formula.parse().unwrap(), Expr::Error(ExprError::Value));

        let formula = Formula::new("=REPLACE('abc', 1, -1, 'x')").unwrap();
        assert_eq!(formula.parse().unwrap(), Expr::Error(ExprError::Value));

        let formula = Formula::new("=TEXTJOIN(',', TRUE, '', 'a', '', 'b')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("a,b".to_string()));

        let formula = Formula::new("=TEXTJOIN(',', FALSE, '', 'a')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String(",a".to_string()));

        let formula = Formula::new("=UPPER(TRIM('   Hello '))").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("HELLO".to_string()));

        let formula = Formula::new("=TRIM(' a   b ')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("a b".to_string()));

        let formula = Formula::new("=T('Hello')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("Hello".to_string()));

        let formula = Formula::new("=T(true)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String(String::new()));

        let formula = Formula::new("=PROPER('this is a TITLE')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("This Is A Title".to_string()));

        let formula = Formula::new("=PROPER(\"o'neil 2-way\")").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("O'Neil 2-Way".to_string()));

        let formula = Formula::new("=LEFTB('hi', 5)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("hi".to_string()));

        let formula = Formula::new("=LEFT('hi', -1)").unwrap();
        assert_eq!(formula.parse().unwrap(), Expr::Error(ExprError::Value));

        let formula = Formula::new("=MIDB('abc', 10, 2)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String(String::new()));

        let formula = Formula::new("=MID('abc', 1, -1)").unwrap();
        assert_eq!(formula.parse().unwrap(), Expr::Error(ExprError::Value));

        let formula = Formula::new("=REPLACEB('abc', 10, 1, 'x')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("abcx".to_string()));

        let formula = Formula::new("=REPLACE('abc', 2, 10, 'x')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("ax".to_string()));

        let formula = Formula::new("=FIND('a', 'abc', 4)").unwrap();
        assert_eq!(formula.parse().unwrap(), Expr::Error(ExprError::Value));

        let formula = Formula::new("=FIND('a', 'abc', 0)").unwrap();
        assert_eq!(formula.parse().unwrap(), Expr::Error(ExprError::Value));

        let formula = Formula::new("=FIND('', 'abc', 2)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(2.0));

        let formula = Formula::new("=REPT('a', 32767)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("a".repeat(32_767)));

        let formula = Formula::new("=REPT('a', 32768)").unwrap();
        assert_eq!(formula.parse().unwrap(), Expr::Error(ExprError::Value));

        let formula = Formula::new("=REPT('a', -1)").unwrap();
        assert_eq!(formula.parse().unwrap(), Expr::Error(ExprError::Value));

        let formula = Formula::new("=FIXED(1234567.891, 1)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("1,234,567.9".to_string()));

        let formula = Formula::new("=FIXED(-1234.5, 1, TRUE)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("-1234.5".to_string()));

        let formula = Formula::new("=FIXED(1234.5, -1000)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("0".to_string()));

        let formula = Formula::new("=FIXED(0.125)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("0.13".to_string()));

        let formula = Formula::new("=CHAR(0)").unwrap();
        assert_eq!(formula.parse().unwrap(), Expr::Error(ExprError::Value));

        let formula = Formula::new("=UNICHAR(55296)").unwrap();
        assert_eq!(formula.parse().unwrap(), Expr::Error(ExprError::Value));

        let formula = Formula::new("=CODE('')").unwrap();
        assert_eq!(formula.parse().unwrap(), Expr::Error(ExprError::Value));

        let formula = Formula::new("=CODE('é')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(233.0));

        let formula = Formula::new("=TRIM('\\ta  b\\t')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("\ta b\t".to_string()));

        let formula = Formula::new("=PROPER('  two   spaces ')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("  Two   Spaces ".to_string()));

        let formula = Formula::new("=TEXTJOIN('-', TRUE, '', '')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String(String::new()));
    }
}
