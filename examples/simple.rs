use formula::{Expr, Formula, Result};

fn main() -> Result<()> {
    let formula = Formula::new("=CONCAT(UPPER(TRIM('   Hello ')), ' ', 'world!')")?;
    let value = formula.parse()?;
    assert_eq!(value, Expr::String("HELLO world!".to_string()));
    Ok(())
}
