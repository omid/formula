use formula::{Expr, Formula};
use std::fmt::Write;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn parse(str: &str) -> Result<JsValue, JsError> {
    let formula = Formula::new(str).map_err(|e| JsError::new(&e.to_string()))?;
    let value = formula.parse().map_err(|e| JsError::new(&e.to_string()))?;

    let value = match value {
        Expr::Date(v) => v.to_string().into(),
        Expr::Datetime(v) => v.to_string().into(),
        Expr::Time(v) => v.to_string().into(),
        Expr::Number(v) => v.into(),
        Expr::String(v) => v.into(),
        Expr::Bool(v) => v.into(),
        Expr::Array(v) => array_to_string(Expr::Array(v)).into(),
        Expr::Null => JsValue::null(),
        Expr::Error(v) => v.to_string().into(),
    };

    Ok(value)
}

/// Serializes the value as JSON
fn array_to_string(arr: Expr) -> String {
    match arr {
        Expr::Array(v) => {
            let strings = v.into_iter().map(array_to_string).collect::<Vec<_>>();
            format!("[{}]", strings.join(","))
        }
        Expr::String(v) => json_string(&v),
        Expr::Date(v) => format!("\"{v}\""),
        Expr::Datetime(v) => format!("\"{v}\""),
        Expr::Time(v) => format!("\"{v}\""),
        // JSON has no NaN or infinity
        Expr::Number(v) if !v.is_finite() => "null".to_string(),
        Expr::Number(v) => v.to_string(),
        Expr::Bool(v) => v.to_string(),
        Expr::Null => "null".to_string(),
        Expr::Error(v) => json_string(&v.to_string()),
    }
}

fn json_string(v: &str) -> String {
    let mut string = String::with_capacity(v.len() + 2);
    string.push('"');
    for c in v.chars() {
        match c {
            '"' => string.push_str("\\\""),
            '\\' => string.push_str("\\\\"),
            c if c < ' ' => {
                let _ = write!(string, "\\u{:04x}", u32::from(c));
            }
            c => string.push(c),
        }
    }
    string.push('"');
    string
}

#[cfg(test)]
mod tests {
    use super::*;
    use formula::ExprError;

    #[test]
    fn test_array_to_string() {
        let array = Expr::Array(vec![
            Expr::String("a\"b\\c\n".to_string()),
            Expr::Number(1.5),
            Expr::Number(f64::NAN),
            Expr::Array(vec![
                Expr::Bool(true),
                Expr::Null,
                Expr::Error(ExprError::DivisionByZero),
            ]),
        ]);
        assert_eq!(
            array_to_string(array),
            r##"["a\"b\\c\u000a",1.5,null,[true,null,"#DIV/0!"]]"##
        );
    }
}
