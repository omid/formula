[![Crates.io](https://img.shields.io/crates/v/formula.svg)](https://crates.io/crates/formula)
[![Workflow Status](https://github.com/omid/formula/workflows/ci/badge.svg)](https://github.com/omid/formula/actions?query=workflow%3A%22ci%22)

<h1 align="center"><em>Formula</em></h1>

<h3 align="center">
  A parser and evaluator of spreadsheet-like formulas
</h3>

Formula is in its early stages and is not ready for production use.

So far we have the following features:

- 18 date time functions
- 26 text functions
- 7 logical functions
- 2 web functions
- plus all arithmetic and comparison operators

### Installation and usage

Add this library to your project with `cargo add formula` or add `formula = "*"` to your `Cargo.toml` file.

Use it similar to the following code:

```rust
use formula::{Formula, Expr, error::Error};
use anyhow::Result;

fn main() -> Result<()> {
    let formula = Formula::new("=UPPER(TRIM('   Hello '))")?;
    let value = formula.parse()?;
    assert_eq!(value, Expr::String("HELLO".to_string()));
    Ok(())
}
```

### What we do not support, yet:

- Support of functions are so limited at the moment, but we would like to add more of them, like Excel functions, Google Sheets functions, and so on
- At the moment, we don't support table data, so you need to pass values to the formula as arguments by yourself
- We do not support simple formulas like `1+1` or as argument like `AND(1>3, 1<3)` or `SUM(2-1, 2)`. Instead, you can use our `F.` functions like `AND(F.GT(1, 3), F.LT(1, 3))` or `SUM(F.SUB(2, 1), 2)`
- We still do not support parentheses to change the order of operations, but you can use our `F.` functions. So for example instead of `2*(1+1)`, you should use `F.MUL(2, F.ADD(1, 1))`

### Contributing

We would love to have your contribution! Please read our [contributing guidelines](CONTRIBUTING.md) to get started.

### License

This project is licensed under the MIT license. See the [LICENSE](LICENSE.md) file for more info.

License: MIT
