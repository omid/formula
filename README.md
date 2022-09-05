# formula

[![Crates.io](https://img.shields.io/crates/v/formula.svg)](https://crates.io/crates/formula)
[![Workflow Status](https://github.com/omid/formula/workflows/main/badge.svg)](https://github.com/omid/formula/actions?query=workflow%3A%22ci%22)

<p align="center">
  <strong style="font-size: 50px"><em>Formula</em></strong>
</p>

<p align="center">
  <strong>A parser and evaluator of spreadsheet-like formulas</strong>
</p>

It's in its early stages, and we are trying to add more functions and features soon.

So far we have the following features:

- 18 date time functions
- 26 text functions

### Installation and usage

Add this library to your project with `cargo add formula` or add `formula = "*"` to your `Cargo.toml` file.

Use it similar to the following code:

```rust
use formula::{Formula, Expr, E};

fn main() -> Result<(), FormulaError> {
    let formula = Formula::new("UPPER(TRIM('   Hello '))")?;
    let value = formula.parse().unwrap();
    assert_eq!(value, Expr::String("HELLO".to_string()));
    Ok(())
}
```

### What we do not support:

- We would like to add more functions, like Excel functions, Google Sheets functions, and more
- At the moment, we don't support table data, so you need to pass values to the formula as arguments by yourself
- We do not support simple formulas like `1+1` or as argument like `AND(1>3, 1<3)` or `SUM(2-1, 2)`, yet

### Contributing

We would love to have your contribution! Please read our [contributing guidelines](CONTRIBUTING.md) to get started.

### License

This project is licensed under the MIT license. See the [LICENSE](LICENSE.md) file for more info.
