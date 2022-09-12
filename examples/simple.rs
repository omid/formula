use formula::{Formula, Expr, Result};

fn main() -> Result<()> {
    let formula = Formula::new("=UPPER(TRIM('   Hello '))")?;
    let value = formula.parse()?;
    assert_eq!(value, Expr::String("HELLO".to_string()));
    Ok(())
}