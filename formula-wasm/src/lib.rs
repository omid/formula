use formula::{Expr, ExprError, Formula};
use js_sys::Array;
use std::fmt::Write;
use wasm_bindgen::prelude::*;

/// Evaluates a formula, its references read the optional `rows`,
/// e.g. `parse('=SUM(A1:B2)', [[1, 2], [3, null]])` is 6
#[wasm_bindgen]
pub fn parse(str: &str, rows: Option<Array>) -> Result<JsValue, JsError> {
    let formula = Formula::new(str).map_err(|e| JsError::new(&e.to_string()))?;
    let rows = rows
        .map(|rows| rows.iter().map(|row| js_row(&row)).collect::<Result<Vec<_>, _>>())
        .transpose()?
        .unwrap_or_default();
    let value = formula.parse_with(&rows).map_err(|e| JsError::new(&e.to_string()))?;

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

/// A row of values: numbers, texts, booleans, and `null` or `undefined` for empty cells.
/// Other values are `#VALUE!`.
fn js_row(row: &JsValue) -> Result<Vec<Expr>, JsError> {
    let row = row
        .dyn_ref::<Array>()
        .ok_or_else(|| JsError::new("Each row must be an array of values"))?;

    Ok(row
        .iter()
        .map(|value| {
            if let Some(number) = value.as_f64() {
                Expr::Number(number)
            } else if let Some(text) = value.as_string() {
                Expr::String(text)
            } else if let Some(bool) = value.as_bool() {
                Expr::Bool(bool)
            } else if value.is_null() || value.is_undefined() {
                Expr::Null
            } else {
                Expr::Error(ExprError::Value)
            }
        })
        .collect())
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
                Expr::Error(ExprError::Value),
            ]),
        ]);
        assert_eq!(
            array_to_string(array),
            r##"["a\"b\\c\u000a",1.5,null,[true,null,"#DIV/0!","#VALUE!"]]"##
        );
    }
}
