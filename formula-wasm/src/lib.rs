use formula::{Expr, Formula};
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
    };

    Ok(value)
}

fn array_to_string(arr: Expr) -> String {
    match arr {
        Expr::Array(v) => {
            let mut strings = Vec::with_capacity(v.len());
            for i in v {
                strings.push(array_to_string(i));
            }
            format!("[{}]", strings.join(","))
        }
        Expr::String(v) => format!("\"{v}\""),
        Expr::Date(v) => format!("\"{v}\""),
        Expr::Datetime(v) => format!("\"{v}\""),
        Expr::Time(v) => format!("\"{v}\""),
        Expr::Number(v) => v.to_string(),
        Expr::Bool(v) => v.to_string(),
        Expr::Null => "null".to_string(),
    }
}
