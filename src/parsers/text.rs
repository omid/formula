use pest::iterators::Pair;
use crate::{Expr, parse_value};
use anyhow::Result;
use crate::error::Error;
use crate::{Rule};

pub fn parse_left(pair: Pair<Rule>) -> Result<Expr> {
    const RULE: &str = "left";
    let mut args = pair.into_inner();
    let text = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let num_chars = args.next().map(|v| parse_value(v)).unwrap_or(Ok(Expr::Number(1.0)))?;

    let text = match (text, num_chars) {
        (Expr::String(text), Expr::Number(chars)) => {
            if chars < 0.0 {
                return Err(Error::Parser(RULE).into());
            }
            text.chars().take(chars as usize).collect()
        }
        _ => return Err(Error::Parser(RULE).into()),
    };
    Ok(Expr::String(text))
}

pub fn parse_leftb(pair: Pair<Rule>) -> Result<Expr> {
    const RULE: &str = "leftb";
    let mut args = pair.into_inner();
    let text = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let num_bytes = args.next().map(|v| parse_value(v)).unwrap_or(Ok(Expr::Number(1.0)))?;

    let text = match (text, num_bytes) {
        (Expr::String(text), Expr::Number(bytes)) => {
            if bytes < 0.0 {
                return Err(Error::Parser(RULE).into());
            }

            let text = text.bytes().take(bytes as usize).collect::<Vec<_>>();
            String::from_utf8_lossy(&text).to_string()
        }
        _ => return Err(Error::Parser(RULE).into()),
    };
    Ok(Expr::String(text))
}

pub fn parse_right(pair: Pair<Rule>) -> Result<Expr> {
    const RULE: &str = "right";
    let mut args = pair.into_inner();
    let text = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let num_chars = args.next().map(|v| parse_value(v)).unwrap_or(Ok(Expr::Number(1.0)))?;

    let text = match (text, num_chars) {
        (Expr::String(text), Expr::Number(chars)) => {
            if chars < 0.0 {
                return Err(Error::Parser(RULE).into());
            }

            let chars = chars as usize;
            let start = text.chars().count() - chars;
            let text = text.chars();
            text.skip(start).collect()
        }
        _ => return Err(Error::Parser(RULE).into()),
    };
    Ok(Expr::String(text))
}

pub fn parse_rightb(pair: Pair<Rule>) -> Result<Expr> {
    const RULE: &str = "rightb";
    let mut args = pair.into_inner();
    let text = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let num_bytes = args.next().map(|v| parse_value(v)).unwrap_or(Ok(Expr::Number(1.0)))?;

    let text = match (text, num_bytes) {
        (Expr::String(text), Expr::Number(bytes)) => {
            if bytes < 0.0 {
                return Err(Error::Parser(RULE).into());
            }

            let bytes = bytes as usize;
            let text = text.bytes();
            let start = text.len() - bytes;
            let text = text.skip(start).collect::<Vec<_>>();
            String::from_utf8_lossy(&text).to_string()
        }
        _ => return Err(Error::Parser(RULE).into()),
    };
    Ok(Expr::String(text))
}

pub fn parse_mid(pair: Pair<Rule>) -> Result<Expr> {
    const RULE: &str = "mid";
    let mut args = pair.into_inner();
    let text = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let start = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let len = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;

    let text = match (text, start, len) {
        (Expr::String(text), Expr::Number(start), Expr::Number(len)) => {
            if start < 0.0 {
                return Err(Error::Parser(RULE).into());
            }
            text.chars().skip((start as usize) - 1).take(len as usize).collect()
        }
        _ => return Err(Error::Parser(RULE).into()),
    };
    Ok(Expr::String(text))
}

pub fn parse_midb(pair: Pair<Rule>) -> Result<Expr> {
    const RULE: &str = "midb";
    let mut args = pair.into_inner();
    let text = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let start = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let len = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;

    let text = match (text, start, len) {
        (Expr::String(text), Expr::Number(start), Expr::Number(len)) => {
            if start < 0.0 {
                return Err(Error::Parser(RULE).into());
            }

            let text = text.bytes().skip((start as usize) - 1).take(len as usize).collect::<Vec<_>>();
            String::from_utf8_lossy(&text).to_string()
        }
        _ => return Err(Error::Parser(RULE).into()),
    };
    Ok(Expr::String(text))
}

pub fn parse_char(pair: Pair<Rule>) -> Result<Expr> {
    const RULE: &str = "char";
    let mut args = pair.into_inner();
    let number = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;

    let char = match number {
        Expr::Number(number) => (number as u8 as char).to_string(),
        _ => return Err(Error::Parser(RULE).into()),
    };
    Ok(Expr::String(char))
}

pub fn parse_code(pair: Pair<Rule>) -> Result<Expr> {
    const RULE: &str = "code";
    let mut args = pair.into_inner();
    let text = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;

    let code = match text {
        Expr::String(text) => text.chars().take(1).map(|c| c as u8).collect::<Vec<_>>().first().cloned().ok_or(Error::Parser(RULE))?,
        _ => return Err(Error::Parser(RULE).into()),
    };
    Ok(Expr::Number(code as f64))
}

pub fn parse_concat(pair: Pair<Rule>) -> Result<Expr> {
    const RULE: &str = "concat";
    let mut args = pair.into_inner();
    let text = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let texts = args.map(|v| parse_value(v)).collect::<Result<Vec<_>>>()?;

    let text = match text {
        Expr::String(mut text) => {
            for t in texts {
                match t {
                    Expr::String(t) => text.extend(t.chars()),
                    _ => return Err(Error::Parser(RULE).into()),
                }
            }
            text
        }
        _ => return Err(Error::Parser(RULE).into()),
    };
    Ok(Expr::String(text))
}

pub fn parse_exact(pair: Pair<Rule>) -> Result<Expr> {
    const RULE: &str = "exact";
    let mut args = pair.into_inner();
    let text1 = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let text2 = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let exact = match (text1, text2) {
        (Expr::String(text1), Expr::String(text2)) => text1 == text2,
        _ => return Err(Error::Parser(RULE).into()),
    };
    Ok(Expr::Bool(exact))
}

pub fn parse_find(pair: Pair<Rule>) -> Result<Expr> {
    const RULE: &str = "find";
    let mut args = pair.into_inner();
    let find_text = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let within_text = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let start_num = args.next().map(|v| parse_value(v)).unwrap_or(Ok(Expr::Number(1.0)))?;

    let index = match (find_text, within_text, start_num) {
        (Expr::String(find_text), Expr::String(within_text), Expr::Number(start_num)) => {
            if start_num < 1.0 || start_num > within_text.len() as f64 {
                return Err(Error::Parser(RULE).into());
            }
            let start = (start_num - 1.0) as usize;
            let within_text = within_text.chars().skip(start).map(|c| c as u8).collect::<Vec<_>>();
            let within_text = String::from_utf8_lossy(&within_text).to_string();
            let index = within_text.find(&find_text).ok_or(Error::Parser(RULE))?;
            index as f64 + start as f64 + 1.0
        }
        _ => return Err(Error::Parser(RULE).into()),
    };
    Ok(Expr::Number(index))
}

pub fn parse_findb(pair: Pair<Rule>) -> Result<Expr> {
    const RULE: &str = "findb";
    let mut args = pair.into_inner();
    let find_text = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let within_text = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let start_num = args.next().map(|v| parse_value(v)).unwrap_or(Ok(Expr::Number(1.0)))?;

    let index = match (find_text, within_text, start_num) {
        (Expr::String(find_text), Expr::String(within_text), Expr::Number(start_num)) => {
            if start_num < 1.0 || start_num > within_text.len() as f64 {
                return Err(Error::Parser(RULE).into());
            }
            let start = (start_num - 1.0) as usize;
            let within_text = &within_text[start..];
            let index = within_text.find(&find_text).ok_or(Error::Parser(RULE))?;
            index as f64 + start as f64 + 1.0
        }
        _ => return Err(Error::Parser(RULE).into()),
    };
    Ok(Expr::Number(index))
}

pub fn parse_search(pair: Pair<Rule>) -> Result<Expr> {
    const RULE: &str = "search";
    let mut args = pair.into_inner();
    let search_text = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let within_text = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let start_num = args.next().map(|v| parse_value(v)).unwrap_or(Ok(Expr::Number(1.0)))?;

    let index = match (search_text, within_text, start_num) {
        (Expr::String(search_text), Expr::String(within_text), Expr::Number(start_num)) => {
            if start_num < 1.0 || start_num > within_text.len() as f64 {
                return Err(Error::Parser(RULE).into());
            }
            let search_text = search_text.to_lowercase();
            let within_text = within_text.to_lowercase();
            let start = (start_num - 1.0) as usize;
            let within_text = within_text.chars().skip(start).map(|c| c as u8).collect::<Vec<_>>();
            let within_text = String::from_utf8_lossy(&within_text).to_string();
            let index = within_text.find(&search_text).ok_or(Error::Parser(RULE))?;
            index as f64 + start as f64 + 1.0
        }
        _ => return Err(Error::Parser(RULE).into()),
    };
    Ok(Expr::Number(index))
}

pub fn parse_searchb(pair: Pair<Rule>) -> Result<Expr> {
    const RULE: &str = "searchb";
    let mut args = pair.into_inner();
    let search_text = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let within_text = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let start_num = args.next().map(|v| parse_value(v)).unwrap_or(Ok(Expr::Number(1.0)))?;

    let index = match (search_text, within_text, start_num) {
        (Expr::String(search_text), Expr::String(within_text), Expr::Number(start_num)) => {
            if start_num < 1.0 || start_num > within_text.len() as f64 {
                return Err(Error::Parser(RULE).into());
            }
            let search_text = search_text.to_lowercase();
            let within_text = within_text.to_lowercase();
            let start = (start_num - 1.0) as usize;
            let within_text = &within_text[start..];
            let index = within_text.find(&search_text).ok_or(Error::Parser(RULE))?;
            index as f64 + start as f64 + 1.0
        }
        _ => return Err(Error::Parser(RULE).into()),
    };
    Ok(Expr::Number(index))
}

pub fn parse_fixed(pair: Pair<Rule>) -> Result<Expr> {
    const RULE: &str = "fixed";
    let mut args = pair.into_inner();
    let number = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let decimals = args.next().map(|v| parse_value(v)).unwrap_or(Ok(Expr::Number(2.0)))?;
    let no_commas = args.next().map(|v| parse_value(v)).unwrap_or(Ok(Expr::Bool(false)))?;
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
                let num_and_decimal = text.split('.').map(|s| s.to_string()).collect::<Vec<_>>();
                let t = num_and_decimal[0].chars().rev().enumerate().map(|(i, c)| {
                    [c.to_string(),
                        if i % 3 == 0 && i != 0 {
                            ",".to_string()
                        } else {
                            "".to_string()
                        }
                    ]
                }).collect::<Vec<_>>().into_iter().rev().flatten().collect::<String>();

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
        _ => return Err(Error::Parser(RULE).into()),
    };
    Ok(Expr::String(text))
}

pub fn parse_len(pair: Pair<Rule>) -> Result<Expr> {
    const RULE: &str = "len";
    let mut args = pair.into_inner();
    let text = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let len = match text {
        Expr::String(text) => text.chars().count() as f64,
        _ => return Err(Error::Parser(RULE).into()),
    };
    Ok(Expr::Number(len))
}

pub fn parse_lenb(pair: Pair<Rule>) -> Result<Expr> {
    const RULE: &str = "lenb";
    let mut args = pair.into_inner();
    let text = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let len = match text {
        Expr::String(text) => text.len() as f64,
        _ => return Err(Error::Parser(RULE).into()),
    };
    Ok(Expr::Number(len))
}

pub fn parse_lower(pair: Pair<Rule>) -> Result<Expr> {
    const RULE: &str = "lower";
    let mut args = pair.into_inner();
    let text = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let text = match text {
        Expr::String(text) => text.to_lowercase(),
        _ => return Err(Error::Parser(RULE).into()),
    };
    Ok(Expr::String(text))
}

pub fn parse_upper(pair: Pair<Rule>) -> Result<Expr> {
    const RULE: &str = "upper";
    let mut args = pair.into_inner();
    let text = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let text = match text {
        Expr::String(text) => text.to_uppercase(),
        _ => return Err(Error::Parser(RULE).into()),
    };
    Ok(Expr::String(text))
}

pub fn parse_rept(pair: Pair<Rule>) -> Result<Expr> {
    const RULE: &str = "rept";
    let mut args = pair.into_inner();
    let text = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let rept = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let text = match (text, rept) {
        (Expr::String(text), Expr::Number(rept)) => {
            if rept < 0.0 {
                return Err(Error::Parser(RULE).into());
            }
            text.repeat(rept as usize)
        }
        _ => return Err(Error::Parser(RULE).into()),
    };
    Ok(Expr::String(text))
}

pub fn parse_replace(pair: Pair<Rule>) -> Result<Expr> {
    const RULE: &str = "replace";
    let mut args = pair.into_inner();
    let text = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let start = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let len = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let new_text = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;

    let text = match (text, start, len, new_text) {
        (Expr::String(text), Expr::Number(start), Expr::Number(len), Expr::String(new_text)) => {
            if start < 0.0 {
                return Err(Error::Parser(RULE).into());
            }
            let start_text = text.chars().take((start as usize) - 1).collect::<String>();
            let end_text = text.chars().skip((start + len) as usize - 1).collect::<String>();
            format!("{}{}{}", start_text, new_text, end_text)
        }
        _ => return Err(Error::Parser(RULE).into()),
    };
    Ok(Expr::String(text))
}

pub fn parse_replaceb(pair: Pair<Rule>) -> Result<Expr> {
    const RULE: &str = "replaceb";
    let mut args = pair.into_inner();
    let text = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let start = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let len = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let new_text = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;

    let text = match (text, start, len, new_text) {
        (Expr::String(text), Expr::Number(start), Expr::Number(len), Expr::String(new_text)) => {
            if start < 0.0 {
                return Err(Error::Parser(RULE).into());
            }
            let start_text = text.bytes().take((start as usize) - 1).collect::<Vec<_>>();
            let end_text = text.bytes().skip((start + len) as usize - 1).collect::<Vec<_>>();
            format!("{}{}{}", String::from_utf8_lossy(&start_text), new_text, String::from_utf8_lossy(&end_text))
        }
        _ => return Err(Error::Parser(RULE).into()),
    };
    Ok(Expr::String(text))
}

pub fn parse_textjoin(pair: Pair<Rule>) -> Result<Expr> {
    const RULE: &str = "textjoin";
    let mut args = pair.into_inner();
    let delim = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let ignore_empty = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let text = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let texts = args.map(|v| parse_value(v)).collect::<Result<Vec<_>>>()?;

    let text = match (delim, ignore_empty, text) {
        (Expr::String(delim), Expr::Bool(ignore_empty), Expr::String(mut text)) => {
            for t in texts {
                match t {
                    Expr::String(t) => {
                        if ignore_empty && t.is_empty() {
                            continue;
                        }
                        text.extend(delim.chars());
                        text.extend(t.chars());
                    },
                    _ => return Err(Error::Parser(RULE).into()),
                }
            }
            text
        }
        _ => return Err(Error::Parser(RULE).into()),
    };
    Ok(Expr::String(text))
}

pub fn parse_trim(pair: Pair<Rule>) -> Result<Expr> {
    const RULE: &str = "trim";
    let mut args = pair.into_inner();
    let text = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let text = match text {
        Expr::String(text) => text.trim().to_string(),
        _ => return Err(Error::Parser(RULE).into()),
    };
    Ok(Expr::String(text))
}

pub fn parse_t(pair: Pair<Rule>) -> Result<Expr> {
    const RULE: &str = "t";
    let mut args = pair.into_inner();
    let text = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let text = match text {
        Expr::String(text) => text,
        _ => "".to_string(),
    };
    Ok(Expr::String(text))
}

pub fn parse_proper(pair: Pair<Rule>) -> Result<Expr> {
    const RULE: &str = "proper";
    let mut args = pair.into_inner();
    let text = parse_value(args.next().ok_or(Error::Parser(RULE))?)?;
    let text = match text {
        Expr::String(text) => text.split_whitespace()
            .map(|w| w.chars())
            .map(|mut c|
                c.next().into_iter()
                    .flat_map(|c| c.to_uppercase())
                    .chain(c.flat_map(|c| c.to_lowercase())))
            .map(|c| c.collect::<String>()).collect::<Vec<_>>().join(" "),
        _ => return Err(Error::Parser(RULE).into()),
    };
    Ok(Expr::String(text))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Formula;
    use pest::Parser;

    #[test]
    #[allow(clippy::too_many_lines)]
    fn test_parse_text_types() {
        let formula = Formula::parse(Rule::formula, "LEFT('hello')").unwrap().next().unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::String("h".to_string()));

        let formula = Formula::parse(Rule::formula, "LEFT('hello', 2)").unwrap().next().unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::String("he".to_string()));

        let formula = Formula::parse(Rule::formula, "LEFTB('hello')").unwrap().next().unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::String("h".to_string()));

        let formula = Formula::parse(Rule::formula, "RIGHT('hello')").unwrap().next().unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::String("o".to_string()));

        let formula = Formula::parse(Rule::formula, "RIGHT('hello', 2)").unwrap().next().unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::String("lo".to_string()));

        let formula = Formula::parse(Rule::formula, "RIGHTB('hello')").unwrap().next().unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::String("o".to_string()));

        let formula = Formula::parse(Rule::formula, "MID('Fluid Flow', 7, 20)").unwrap().next().unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::String("Flow".to_string()));

        let formula = Formula::parse(Rule::formula, "MID('Fluid Flow', 20, 5)").unwrap().next().unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::String("".to_string()));

        let formula = Formula::parse(Rule::formula, "MIDB('Fluid Flow', 7, 20)").unwrap().next().unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::String("Flow".to_string()));

        let formula = Formula::parse(Rule::formula, "CODE('A')").unwrap().next().unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::Number(65.0));

        let formula = Formula::parse(Rule::formula, "CHAR(65)").unwrap().next().unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::String("A".to_string()));

        let formula = Formula::parse(Rule::formula, "CONCAT('hello', ' ', 'world')").unwrap().next().unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::String("hello world".to_string()));

        let formula = Formula::parse(Rule::formula, "CONCATENATE('hello', ' ', 'world')").unwrap().next().unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::String("hello world".to_string()));

        let formula = Formula::parse(Rule::formula, "EXACT('hello', 'hello')").unwrap().next().unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::Bool(true));

        let formula = Formula::parse(Rule::formula, "EXACT('hello', 'Hello')").unwrap().next().unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::Bool(false));

        let formula = Formula::parse(Rule::formula, "FIND('a', 'abca')").unwrap().next().unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::Number(1.0));

        let formula = Formula::parse(Rule::formula, "FIND('a', 'abca', 2)").unwrap().next().unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::Number(4.0));

        let formula = Formula::parse(Rule::formula, "SEARCH('a', 'ABCA')").unwrap().next().unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::Number(1.0));

        let formula = Formula::parse(Rule::formula, "SEARCH('a', 'ABCA', 2)").unwrap().next().unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::Number(4.0));

        let formula = Formula::parse(Rule::formula, "FIXED(123456.673, 2)").unwrap().next().unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::String("123,456.67".to_string()));

        let formula = Formula::parse(Rule::formula, "FIXED(123456.673, -2)").unwrap().next().unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::String("123,400".to_string()));

        let formula = Formula::parse(Rule::formula, "FIXED(123456.673, 0)").unwrap().next().unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::String("123,457".to_string()));

        let formula = Formula::parse(Rule::formula, "FIXED(123456.673, 2, true)").unwrap().next().unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::String("123456.67".to_string()));

        let formula = Formula::parse(Rule::formula, "LEN('hello')").unwrap().next().unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::Number(5.0));

        let formula = Formula::parse(Rule::formula, "LEN('سلام')").unwrap().next().unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::Number(4.0));

        let formula = Formula::parse(Rule::formula, "LENB('سلام')").unwrap().next().unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::Number(8.0));

        let formula = Formula::parse(Rule::formula, "LOWER('Hello')").unwrap().next().unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::String("hello".to_string()));

        let formula = Formula::parse(Rule::formula, "UPPER('Hello')").unwrap().next().unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::String("HELLO".to_string()));

        let formula = Formula::parse(Rule::formula, "REPT('H', 5)").unwrap().next().unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::String("HHHHH".to_string()));

        let formula = Formula::parse(Rule::formula, "REPT('H', 0)").unwrap().next().unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::String("".to_string()));

        let formula = Formula::parse(Rule::formula, "REPLACE('abcdefghijk', 6, 5, '*')").unwrap().next().unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::String("abcde*k".to_string()));

        let formula = Formula::parse(Rule::formula, "REPLACEB('123456', 1, 3, '@')").unwrap().next().unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::String("@456".to_string()));

        let formula = Formula::parse(Rule::formula, "UPPER(TRIM('   Hello '))").unwrap().next().unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::String("HELLO".to_string()));

        let formula = Formula::parse(Rule::formula, "T('Hello')").unwrap().next().unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::String("Hello".to_string()));

        let formula = Formula::parse(Rule::formula, "T(true)").unwrap().next().unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::String("".to_string()));

        let formula = Formula::parse(Rule::formula, "PROPER('this is a TITLE')").unwrap().next().unwrap();
        let value = parse_value(formula).unwrap();
        assert_eq!(value, Expr::String("This Is A Title".to_string()));
    }
}
