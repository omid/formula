use formula::{Cells, Expr, ExprError, Formula, Size};
use js_sys::Array;
use std::fmt::Write;
use wasm_bindgen::prelude::*;

/// Evaluates a formula, its references read the optional `rows`,
/// e.g. `parse('=SUM(A1:B2)', [[1, 2], [3, null]])` is 6
#[wasm_bindgen]
pub fn parse(str: &str, rows: Option<Array>) -> Result<JsValue, JsError> {
    let formula = Formula::new(str).map_err(|e| JsError::new(&e.to_string()))?;
    let value = match rows {
        Some(rows) => formula.parse_with(&JsRows::new(rows)?),
        None => formula.parse(),
    }
    .map_err(|e| JsError::new(&e.to_string()))?;

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

/// Rows of values from JavaScript, their cells are only converted when the formula reads them
struct JsRows {
    rows: Array,
    size: Size,
}

impl JsRows {
    fn new(rows: Array) -> Result<Self, JsError> {
        let mut columns = 0;
        for row in rows.iter() {
            let row = row
                .dyn_ref::<Array>()
                .ok_or_else(|| JsError::new("Each row must be an array of values"))?;
            columns = columns.max(row.length());
        }
        let size = Size {
            rows: rows.length(),
            columns,
        };
        Ok(Self { rows, size })
    }
}

impl Cells for JsRows {
    fn size(&self, sheet: Option<&str>) -> Option<Size> {
        sheet.is_none().then_some(self.size)
    }

    /// Numbers, texts and booleans, `null` and `undefined` are empty cells, other values are `#VALUE!`
    fn cell(&self, _sheet: Option<&str>, row: u32, column: u32) -> Option<Expr> {
        let row = self.rows.get(row.checked_sub(1)?).dyn_into::<Array>().ok()?;
        let value = row.get(column.checked_sub(1)?);

        if let Some(number) = value.as_f64() {
            Some(Expr::Number(number))
        } else if let Some(text) = value.as_string() {
            Some(Expr::String(text))
        } else if let Some(bool) = value.as_bool() {
            Some(Expr::Bool(bool))
        } else if value.is_null() || value.is_undefined() {
            None
        } else {
            Some(Expr::Error(ExprError::Value))
        }
    }
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

/// Run in Node with wasm-bindgen's test runner, see `make test_wasm`
#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    /// Rows of values from JSON, which has no `undefined`
    fn rows(json: &str) -> Option<Array> {
        Some(js_sys::JSON::parse(json).unwrap().unchecked_into())
    }

    fn error_message(error: JsError) -> String {
        JsValue::from(error).unchecked_into::<js_sys::Error>().message().into()
    }

    #[wasm_bindgen_test]
    fn test_parse_without_rows() {
        assert_eq!(parse("=UPPER('hi')", None).unwrap(), "HI");
        assert_eq!(parse("=F.ADD(1, 2)", None).unwrap(), 3.0);
        assert_eq!(parse("=TRUE", None).unwrap(), true);
        assert_eq!(parse("=A1", None).unwrap(), JsValue::NULL);
        assert_eq!(parse("=F.DIV(1, 0)", None).unwrap(), "#DIV/0!");
        assert_eq!(parse("=DATE(2020, 1, 2)", None).unwrap(), "2020-01-02");
        assert_eq!(parse("=TIME(1, 2, 3)", None).unwrap(), "01:02:03");
        assert_eq!(
            parse("={1, 'a'; TRUE, NA()}", None).unwrap(),
            r##"[[1,"a"],[true,"#N/A"]]"##
        );

        // the clock and random numbers work in WASM
        assert!(parse("=YEAR(NOW())", None).unwrap().as_f64().unwrap() >= 2026.0);
        let random = parse("=RAND()", None).unwrap().as_f64().unwrap();
        assert!((0.0..1.0).contains(&random));
    }

    #[wasm_bindgen_test]
    fn test_parse_with_rows() {
        assert_eq!(parse("=SUM(A1:B2)", rows("[[1, 2], [3, null]]")).unwrap(), 6.0);
        assert_eq!(parse("=A1", rows(r#"[["x"]]"#)).unwrap(), "x");
        assert_eq!(parse("=AND(A1:B1)", rows("[[true, 1]]")).unwrap(), true);
        assert_eq!(parse("=A1", rows("[[]]")).unwrap(), JsValue::NULL);
        assert_eq!(parse("=A1", rows("[]")).unwrap(), JsValue::NULL);
        assert_eq!(
            parse(
                r#"=TEXTJOIN(", ", TRUE, A:A)"#,
                rows(r#"[["apples"], [null], ["pears"]]"#)
            )
            .unwrap(),
            "apples, pears"
        );

        // ranges are JSON
        assert_eq!(parse("=A1:B1", rows(r#"[[1, "a"]]"#)).unwrap(), r#"[1,"a"]"#);
        assert_eq!(parse("=A1:A2", rows("[[1], [null]]")).unwrap(), "[[1],[null]]");

        // other values, and sheets
        assert_eq!(parse("=A1", rows("[[{}]]")).unwrap(), "#VALUE!");
        assert_eq!(parse("=A1", rows("[[[1]]]")).unwrap(), "#VALUE!");
        assert_eq!(parse("=Sheet2!A1", rows("[[1]]")).unwrap(), "#REF!");
    }

    #[wasm_bindgen_test]
    fn test_parse_undefined_cells() {
        let row = Array::new();
        row.push(&JsValue::from(1));
        row.push(&JsValue::UNDEFINED);
        let rows = Array::of1(&row);

        assert_eq!(parse("=B1", Some(rows.clone())).unwrap(), JsValue::NULL);
        assert_eq!(parse("=SUM(A1:B1)", Some(rows)).unwrap(), 1.0);
    }

    #[wasm_bindgen_test]
    fn test_parse_reads_only_the_cells_it_needs() {
        // B1 counts how often it's read
        let rows = js_sys::eval(
            "globalThis.reads = 0;
            const row = [1, 2];
            Object.defineProperty(row, 1, { get() { globalThis.reads++; return 2; } });
            [row]",
        )
        .unwrap();
        let rows = Some(rows.unchecked_into::<Array>());
        let reads = || js_sys::eval("globalThis.reads").unwrap();

        assert_eq!(parse("=A1", rows.clone()).unwrap(), 1.0);
        assert_eq!(reads(), 0.0);
        assert_eq!(parse("=SUM(A1:B1)", rows).unwrap(), 3.0);
        assert_eq!(reads(), 1.0);
    }

    #[wasm_bindgen_test]
    fn test_parse_errors() {
        let error = parse("=A1", rows("[1]")).unwrap_err();
        assert_eq!(error_message(error), "Each row must be an array of values");

        let error = parse("=XFE1", None).unwrap_err();
        assert!(error_message(error).contains("XFE1"));

        let error = parse("=WEEKNUM(1)", None).unwrap_err();
        assert!(error_message(error).contains("weeknum"));
    }
}
