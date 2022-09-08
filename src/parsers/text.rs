use crate::{error::Error, Expr, Formula, Rule};
use anyhow::Result;
use pest::iterators::Pair;

impl Formula<'_> {
    pub(crate) fn parse_left(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let text = Self::get_formula(&mut args, &rule_name)?;
        let num_chars = Self::get_opt_formula_with_default(&mut args, Expr::Number(1.0))?;

        let text = match (text, num_chars) {
            (Expr::String(text), Expr::Number(chars)) => {
                if chars < 0.0 {
                    return Err(Error::Parser(rule_name).into());
                }
                text.chars().take(chars as usize).collect()
            }
            _ => return Err(Error::Parser(rule_name).into()),
        };
        Ok(Expr::String(text))
    }

    pub(crate) fn parse_leftb(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let text = Self::get_formula(&mut args, &rule_name)?;
        let num_bytes = Self::get_opt_formula_with_default(&mut args, Expr::Number(1.0))?;

        let text = match (text, num_bytes) {
            (Expr::String(text), Expr::Number(bytes)) => {
                if bytes < 0.0 {
                    return Err(Error::Parser(rule_name).into());
                }

                let text = text.bytes().take(bytes as usize).collect::<Vec<_>>();
                String::from_utf8_lossy(&text).to_string()
            }
            _ => return Err(Error::Parser(rule_name).into()),
        };
        Ok(Expr::String(text))
    }

    pub(crate) fn parse_right(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let text = Self::get_formula(&mut args, &rule_name)?;
        let num_chars = Self::get_opt_formula_with_default(&mut args, Expr::Number(1.0))?;

        let text = match (text, num_chars) {
            (Expr::String(text), Expr::Number(chars)) => {
                if chars < 0.0 {
                    return Err(Error::Parser(rule_name).into());
                }

                let chars = chars as usize;
                let start = text.chars().count() - chars;
                let text = text.chars();
                text.skip(start).collect()
            }
            _ => return Err(Error::Parser(rule_name).into()),
        };
        Ok(Expr::String(text))
    }

    pub(crate) fn parse_rightb(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let text = Self::get_formula(&mut args, &rule_name)?;
        let num_bytes = Self::get_opt_formula_with_default(&mut args, Expr::Number(1.0))?;

        let text = match (text, num_bytes) {
            (Expr::String(text), Expr::Number(bytes)) => {
                if bytes < 0.0 {
                    return Err(Error::Parser(rule_name).into());
                }

                let bytes = bytes as usize;
                let text = text.bytes();
                let start = text.len() - bytes;
                let text = text.skip(start).collect::<Vec<_>>();
                String::from_utf8_lossy(&text).to_string()
            }
            _ => return Err(Error::Parser(rule_name).into()),
        };
        Ok(Expr::String(text))
    }

    pub(crate) fn parse_mid(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let text = Self::get_formula(&mut args, &rule_name)?;
        let start = Self::get_formula(&mut args, &rule_name)?;
        let len = Self::get_formula(&mut args, &rule_name)?;

        let text = match (text, start, len) {
            (Expr::String(text), Expr::Number(start), Expr::Number(len)) => {
                if start < 0.0 {
                    return Err(Error::Parser(rule_name).into());
                }
                text.chars().skip((start as usize) - 1).take(len as usize).collect()
            }
            _ => return Err(Error::Parser(rule_name).into()),
        };
        Ok(Expr::String(text))
    }

    pub(crate) fn parse_midb(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let text = Self::get_formula(&mut args, &rule_name)?;
        let start = Self::get_formula(&mut args, &rule_name)?;
        let len = Self::get_formula(&mut args, &rule_name)?;

        let text = match (text, start, len) {
            (Expr::String(text), Expr::Number(start), Expr::Number(len)) => {
                if start < 0.0 {
                    return Err(Error::Parser(rule_name).into());
                }

                let text = text
                    .bytes()
                    .skip((start as usize) - 1)
                    .take(len as usize)
                    .collect::<Vec<_>>();
                String::from_utf8_lossy(&text).to_string()
            }
            _ => return Err(Error::Parser(rule_name).into()),
        };
        Ok(Expr::String(text))
    }

    pub(crate) fn parse_char(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let number = Self::get_formula(&mut args, &rule_name)?;

        let char = match number {
            Expr::Number(number) => (number as u8 as char).to_string(),
            _ => return Err(Error::Parser(rule_name).into()),
        };
        Ok(Expr::String(char))
    }

    pub(crate) fn parse_code(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let text = Self::get_formula(&mut args, &rule_name)?;

        let code = match text {
            Expr::String(text) => text
                .chars()
                .take(1)
                .map(|c| c as u8)
                .collect::<Vec<_>>()
                .first()
                .copied()
                .ok_or_else(|| Error::Parser(rule_name.clone()))?,
            _ => return Err(Error::Parser(rule_name).into()),
        };
        Ok(Expr::Number(f64::from(code)))
    }

    pub(crate) fn parse_concat(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let text = Self::get_formula(&mut args, &rule_name)?;
        let texts = args.map(Self::parse_pair).collect::<Result<Vec<_>>>()?;

        let text = match text {
            Expr::String(mut text) => {
                for t in texts {
                    match t {
                        Expr::String(t) => text.push_str(&t),
                        _ => return Err(Error::Parser(rule_name).into()),
                    }
                }
                text
            }
            _ => return Err(Error::Parser(rule_name).into()),
        };
        Ok(Expr::String(text))
    }

    pub(crate) fn parse_exact(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let text1 = Self::get_formula(&mut args, &rule_name)?;
        let text2 = Self::get_formula(&mut args, &rule_name)?;
        let exact = match (text1, text2) {
            (Expr::String(text1), Expr::String(text2)) => text1 == text2,
            _ => return Err(Error::Parser(rule_name).into()),
        };
        Ok(Expr::Bool(exact))
    }

    pub(crate) fn parse_find(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let find_text = Self::get_formula(&mut args, &rule_name)?;
        let within_text = Self::get_formula(&mut args, &rule_name)?;
        let start_num = Self::get_opt_formula_with_default(&mut args, Expr::Number(1.0))?;

        let index = match (find_text, within_text, start_num) {
            (Expr::String(find_text), Expr::String(within_text), Expr::Number(start_num)) => {
                if start_num < 1.0 || start_num > within_text.len() as f64 {
                    return Err(Error::Parser(rule_name).into());
                }
                let start = (start_num - 1.0) as usize;
                let within_text = within_text.chars().skip(start).map(|c| c as u8).collect::<Vec<_>>();
                let within_text = String::from_utf8_lossy(&within_text).to_string();
                let index = within_text
                    .find(&find_text)
                    .ok_or_else(|| Error::Parser(rule_name.clone()))?;
                index as f64 + start as f64 + 1.0
            }
            _ => return Err(Error::Parser(rule_name).into()),
        };
        Ok(Expr::Number(index))
    }

    pub(crate) fn parse_findb(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let find_text = Self::get_formula(&mut args, &rule_name)?;
        let within_text = Self::get_formula(&mut args, &rule_name)?;
        let start_num = Self::get_opt_formula_with_default(&mut args, Expr::Number(1.0))?;

        let index = match (find_text, within_text, start_num) {
            (Expr::String(find_text), Expr::String(within_text), Expr::Number(start_num)) => {
                if start_num < 1.0 || start_num > within_text.len() as f64 {
                    return Err(Error::Parser(rule_name).into());
                }
                let start = (start_num - 1.0) as usize;
                let within_text = &within_text[start..];
                let index = within_text
                    .find(&find_text)
                    .ok_or_else(|| Error::Parser(rule_name.clone()))?;
                index as f64 + start as f64 + 1.0
            }
            _ => return Err(Error::Parser(rule_name).into()),
        };
        Ok(Expr::Number(index))
    }

    pub(crate) fn parse_search(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let search_text = Self::get_formula(&mut args, &rule_name)?;
        let within_text = Self::get_formula(&mut args, &rule_name)?;
        let start_num = Self::get_opt_formula_with_default(&mut args, Expr::Number(1.0))?;

        let index = match (search_text, within_text, start_num) {
            (Expr::String(search_text), Expr::String(within_text), Expr::Number(start_num)) => {
                if start_num < 1.0 || start_num > within_text.len() as f64 {
                    return Err(Error::Parser(rule_name).into());
                }
                let search_text = search_text.to_lowercase();
                let within_text = within_text.to_lowercase();
                let start = (start_num - 1.0) as usize;
                let within_text = within_text.chars().skip(start).map(|c| c as u8).collect::<Vec<_>>();
                let within_text = String::from_utf8_lossy(&within_text).to_string();
                let index = within_text
                    .find(&search_text)
                    .ok_or_else(|| Error::Parser(rule_name.clone()))?;
                index as f64 + start as f64 + 1.0
            }
            _ => return Err(Error::Parser(rule_name).into()),
        };
        Ok(Expr::Number(index))
    }

    pub(crate) fn parse_searchb(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let search_text = Self::get_formula(&mut args, &rule_name)?;
        let within_text = Self::get_formula(&mut args, &rule_name)?;
        let start_num = Self::get_opt_formula_with_default(&mut args, Expr::Number(1.0))?;

        let index = match (search_text, within_text, start_num) {
            (Expr::String(search_text), Expr::String(within_text), Expr::Number(start_num)) => {
                if start_num < 1.0 || start_num > within_text.len() as f64 {
                    return Err(Error::Parser(rule_name).into());
                }
                let search_text = search_text.to_lowercase();
                let within_text = within_text.to_lowercase();
                let start = (start_num - 1.0) as usize;
                let within_text = &within_text[start..];
                let index = within_text
                    .find(&search_text)
                    .ok_or_else(|| Error::Parser(rule_name.clone()))?;
                index as f64 + start as f64 + 1.0
            }
            _ => return Err(Error::Parser(rule_name).into()),
        };
        Ok(Expr::Number(index))
    }

    pub(crate) fn parse_fixed(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let number = Self::get_formula(&mut args, &rule_name)?;
        let decimals = Self::get_opt_formula_with_default(&mut args, Expr::Number(2.0))?;
        let no_commas = Self::get_opt_formula_with_default(&mut args, Expr::Bool(false))?;
        let text = match (number, decimals, no_commas) {
            (Expr::Number(number), Expr::Number(mut decimals), Expr::Bool(no_commas)) => {
                let number = if decimals < 0.0 {
                    let number = (number / 10.0f64.powf(decimals.abs())).floor() * 10.0f64.powf(decimals.abs());
                    decimals = 0.0;
                    number
                } else {
                    number
                };

                let decimals = decimals as usize;
                let mut text = format!("{:.*}", decimals, number);

                if !no_commas {
                    // separate text with commas
                    let num_and_decimal = text.split('.').map(std::string::ToString::to_string).collect::<Vec<_>>();
                    let t = num_and_decimal[0]
                        .chars()
                        .rev()
                        .enumerate()
                        .map(|(i, c)| {
                            [
                                c.to_string(),
                                if i % 3 == 0 && i != 0 {
                                    ",".to_string()
                                } else {
                                    String::new()
                                },
                            ]
                        })
                        .collect::<Vec<_>>()
                        .into_iter()
                        .rev()
                        .flatten()
                        .collect::<String>();

                    match num_and_decimal.get(1) {
                        Some(decimals) => {
                            text = format!("{}.{}", t, decimals);
                        }
                        None => {
                            text = t;
                        }
                    }
                }
                text
            }
            _ => return Err(Error::Parser(rule_name).into()),
        };
        Ok(Expr::String(text))
    }

    pub(crate) fn parse_len(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let text = Self::get_formula(&mut args, &rule_name)?;
        let len = match text {
            Expr::String(text) => text.chars().count() as f64,
            _ => return Err(Error::Parser(rule_name).into()),
        };
        Ok(Expr::Number(len))
    }

    pub(crate) fn parse_lenb(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let text = Self::get_formula(&mut args, &rule_name)?;
        let len = match text {
            Expr::String(text) => text.len() as f64,
            _ => return Err(Error::Parser(rule_name).into()),
        };
        Ok(Expr::Number(len))
    }

    pub(crate) fn parse_lower(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let text = Self::get_formula(&mut args, &rule_name)?;
        let text = match text {
            Expr::String(text) => text.to_lowercase(),
            _ => return Err(Error::Parser(rule_name).into()),
        };
        Ok(Expr::String(text))
    }

    pub(crate) fn parse_upper(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let text = Self::get_formula(&mut args, &rule_name)?;
        let text = match text {
            Expr::String(text) => text.to_uppercase(),
            _ => return Err(Error::Parser(rule_name).into()),
        };
        Ok(Expr::String(text))
    }

    pub(crate) fn parse_rept(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let text = Self::get_formula(&mut args, &rule_name)?;
        let rept = Self::get_formula(&mut args, &rule_name)?;
        let text = match (text, rept) {
            (Expr::String(text), Expr::Number(rept)) => {
                if rept < 0.0 {
                    return Err(Error::Parser(rule_name).into());
                }
                text.repeat(rept as usize)
            }
            _ => return Err(Error::Parser(rule_name).into()),
        };
        Ok(Expr::String(text))
    }

    pub(crate) fn parse_replace(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let text = Self::get_formula(&mut args, &rule_name)?;
        let start = Self::get_formula(&mut args, &rule_name)?;
        let len = Self::get_formula(&mut args, &rule_name)?;
        let new_text = Self::get_formula(&mut args, &rule_name)?;

        let text = match (text, start, len, new_text) {
            (Expr::String(text), Expr::Number(start), Expr::Number(len), Expr::String(new_text)) => {
                if start < 0.0 {
                    return Err(Error::Parser(rule_name).into());
                }
                let start_text = text.chars().take((start as usize) - 1).collect::<String>();
                let end_text = text.chars().skip((start + len) as usize - 1).collect::<String>();
                format!("{}{}{}", start_text, new_text, end_text)
            }
            _ => return Err(Error::Parser(rule_name).into()),
        };
        Ok(Expr::String(text))
    }

    pub(crate) fn parse_replaceb(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let text = Self::get_formula(&mut args, &rule_name)?;
        let start = Self::get_formula(&mut args, &rule_name)?;
        let len = Self::get_formula(&mut args, &rule_name)?;
        let new_text = Self::get_formula(&mut args, &rule_name)?;

        let text = match (text, start, len, new_text) {
            (Expr::String(text), Expr::Number(start), Expr::Number(len), Expr::String(new_text)) => {
                if start < 0.0 {
                    return Err(Error::Parser(rule_name).into());
                }
                let start_text = text.bytes().take((start as usize) - 1).collect::<Vec<_>>();
                let end_text = text.bytes().skip((start + len) as usize - 1).collect::<Vec<_>>();
                format!(
                    "{}{}{}",
                    String::from_utf8_lossy(&start_text),
                    new_text,
                    String::from_utf8_lossy(&end_text)
                )
            }
            _ => return Err(Error::Parser(rule_name).into()),
        };
        Ok(Expr::String(text))
    }

    pub(crate) fn parse_textjoin(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let delim = Self::get_formula(&mut args, &rule_name)?;
        let ignore_empty = Self::get_formula(&mut args, &rule_name)?;
        let text = Self::get_formula(&mut args, &rule_name)?;
        let texts = args.map(Self::parse_pair).collect::<Result<Vec<_>>>()?;

        let text = match (delim, ignore_empty, text) {
            (Expr::String(delim), Expr::Bool(ignore_empty), Expr::String(mut text)) => {
                for t in texts {
                    match t {
                        Expr::String(t) => {
                            if ignore_empty && t.is_empty() {
                                continue;
                            }
                            text.push_str(&delim);
                            text.push_str(&t);
                        }
                        _ => return Err(Error::Parser(rule_name).into()),
                    }
                }
                text
            }
            _ => return Err(Error::Parser(rule_name).into()),
        };
        Ok(Expr::String(text))
    }

    pub(crate) fn parse_trim(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let text = Self::get_formula(&mut args, &rule_name)?;
        let text = match text {
            Expr::String(text) => text.trim().to_string(),
            _ => return Err(Error::Parser(rule_name).into()),
        };
        Ok(Expr::String(text))
    }

    pub(crate) fn parse_t(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let text = Self::get_formula(&mut args, &rule_name)?;
        let text = match text {
            Expr::String(text) => text,
            _ => String::new(),
        };
        Ok(Expr::String(text))
    }

    pub(crate) fn parse_proper(pair: Pair<Rule>) -> Result<Expr> {
        let rule_name = format!("{:?}", &pair.as_rule());
        let mut args = pair.into_inner();
        let text = Self::get_formula(&mut args, &rule_name)?;
        let text = match text {
            Expr::String(text) => text
                .split_whitespace()
                .map(str::chars)
                .map(|mut c| {
                    c.next()
                        .into_iter()
                        .flat_map(char::to_uppercase)
                        .chain(c.flat_map(char::to_lowercase))
                })
                .map(Iterator::collect::<String>)
                .collect::<Vec<_>>()
                .join(" "),
            _ => return Err(Error::Parser(rule_name).into()),
        };
        Ok(Expr::String(text))
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

        let formula = Formula::new("=MID('Fluid Flow', 7, 20)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("Flow".to_string()));

        let formula = Formula::new("=MID('Fluid Flow', 20, 5)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String(String::new()));

        let formula = Formula::new("=MIDB('Fluid Flow', 7, 20)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("Flow".to_string()));

        let formula = Formula::new("=CODE('A')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::Number(65.0));

        let formula = Formula::new("=CHAR(65)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("A".to_string()));

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

        let formula = Formula::new("=FIXED(123456.673, 2)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("123,456.67".to_string()));

        let formula = Formula::new("=FIXED(123456.673, -2)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("123,400".to_string()));

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

        let formula = Formula::new("=REPLACE('abcdefghijk', 6, 5, '*')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("abcde*k".to_string()));

        let formula = Formula::new("=REPLACEB('123456', 1, 3, '@')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("@456".to_string()));

        let formula = Formula::new("=UPPER(TRIM('   Hello '))").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("HELLO".to_string()));

        let formula = Formula::new("=T('Hello')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("Hello".to_string()));

        let formula = Formula::new("=T(true)").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String(String::new()));

        let formula = Formula::new("=PROPER('this is a TITLE')").unwrap();
        let value = formula.parse().unwrap();
        assert_eq!(value, Expr::String("This Is A Title".to_string()));
    }
}
