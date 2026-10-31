# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- Table data: formulas can read cells through references like `A1`, `$A$1`, `B3:A1`, `A:C`, `2:3`, `Sheet2!A1`
  or `'My sheet'!A1:B2`. `Formula::parse_with` reads them from the new `Cells` trait, which rows of values
  (`Vec<Vec<Expr>>`) implement. A cell is its value and a range is its rows, like array literals
- `formula-wasm`'s `parse` takes optional rows of values, e.g. `parse('=SUM(A1:B2)', [[1, 2], [3, null]])`
- `#REF!` (`ExprError::Reference`) for references to sheets that don't exist
- `Expr` converts from numbers, booleans, texts, dates, times, errors and options, to build table data
- `formula-wasm`'s tests run in Node, with `make test_wasm` and in CI
- A test that 20,000 random formulas never panic, and tests for the functions and branches that had none
- `make coverage` measures the test coverage with `cargo-llvm-cov`
- Benchmarks with `criterion`, `make bench` runs them

### Changed

- `Formula::parse` takes `&self`, so a formula can be evaluated against many tables
- Like Excel, empty cells are 0 in math, an empty text in text functions and FALSE in logical functions
- `SUM`, `AND`, `OR`, `XOR`, `CONCAT`, `CONCATENATE` and `TEXTJOIN` accept ranges and arrays, and skip their values
  of other types like Excel, e.g. `SUM({1, 'a'})` is 1 instead of `#VALUE!`
- `CONCAT`, `CONCATENATE` and `TEXTJOIN` are `#VALUE!` for texts longer than Excel's 32,767 characters
- `SUM`, `AND`, `OR`, `XOR`, `CONCAT` and `TEXTJOIN` read the cells of ranges one by one, without copying the range,
  e.g. `SUM(A:A)` of 100,000 numbers is about 5 times faster. `CONCAT` and `TEXTJOIN` stop at the maximum text length
- `formula-wasm` converts a cell from JavaScript only when the formula reads it, e.g. `=A1` with a 10,000 × 10 table
  is about 20 times faster
- `LEFT`, `RIGHT`, `MID`, `REPLACE` and `TRIM` copy less, they are 25% to 45% faster on long texts

### Fixed

- `ATAN2` takes the x and then the y coordinate like Excel, e.g. `ATAN2(1, 0)` is 0 instead of π/2,
  and `ATAN2(0, 0)` is `#DIV/0!`

## 0.2.0 (2026-10-31)

### Added

- `NA()` function
- Like Excel, formulas can nest functions up to 64 levels deep, deeper formulas return `Error::Nesting`
  instead of overflowing the stack

### Changed

- MSRV is now Rust 1.99, and the crates use the 2024 edition
- Upgraded dependencies, and replaced `rand` with `getrandom`
- Invalid input in many functions returns a spreadsheet error instead of panicking, e.g. `EDATE`, `EOMONTH`, `RIGHT`, `MID`, `REPLACE`, `FINDB`, `SEARCHB`, `YEAR('2020')`, `REPT` with a huge count
- `WEEKNUM` and `FILTERXML` are rejected as not implemented instead of panicking
- Text after a complete formula is an error, e.g. `=1+1` used to return `1`
- Strings can contain the other quote character, e.g. `="it's"`, and escapes like `\n` or `é` are decoded correctly
- `FLOOR.PRECISE`, `SWITCH`, `WORKDAY` and `WORKDAY.INTL` are recognized again, and rejected as not implemented yet
- **Breaking:** spreadsheet errors are `Expr::Error(ExprError)` values (`#DIV/0!`, `#NUM!`, `#N/A`) instead of `Expr::Null`.
  Like Excel, they pass through the functions that use them, until `IFERROR`, `IFNA` or the result
- **Breaking:** like Excel, arguments of the wrong type and invalid values are `#VALUE!` instead of `Error::Parser`,
  e.g. `F.ADD('a', 1)` or `LEFT('a', -1)`. `Formula::parse` doesn't return `Err` anymore
- **Breaking:** `Formula::new` returns `Error::NotImplemented` for functions that are not implemented yet,
  even in a branch that wouldn't be evaluated, instead of `Formula::parse` returning it
- Division by zero is `#DIV/0!`, and results that are not finite numbers (e.g. `SQRT(-1)`, `LN(0)`) are `#NUM!`
- `F.EQ`, `F.NE` and the other comparisons agree with each other, and ignore floating point noise like Excel
- Text comparisons are case-insensitive like Excel, e.g. `F.EQ('a', 'A')` is `TRUE`. `EXACT` stays case-sensitive
- `IF` only evaluates the selected branch, and returns `FALSE` when the false branch is missing
- `IFERROR` catches spreadsheet errors, `IFNA` only catches `#N/A` and returns the value otherwise
- `WEEKDAY` uses Excel's numbering and supports the return type argument, an invalid one is `#NUM!`
- Excel compatible `MOD`, `FIXED`, `TRIM`, `PROPER`, `TEXTJOIN`, `CHAR`, `UNICHAR`, `CODE`, `UNICODE`, `FIND` and `SEARCH`
- Date and time functions accept the result of `NOW()`
- `DATE` and `TIME` roll over like Excel, e.g. `DATE(2020, 13, 1)` is 2021-01-01, `DATE(2020, 1, 0)` is 2019-12-31
  and `TIME(25, 0, 0)` is 01:00. Out of range results are `#NUM!`, also for `EDATE` and `EOMONTH`
- `WEBSERVICE` returns `#VALUE!` on failed requests and HTTP error responses, and is not implemented on WASM
- The `WEBSERVICE` tests use a local HTTP server, so tests don't need internet access
- The WASM package returns valid JSON for arrays

## 0.1.0 (2022-09-20)

### Added

- Initial WASM support and npm package which you can find here: https://www.npmjs.com/package/formula-wasm

## 0.0.3 (2022-09-11)

### Added

- Support of basic mathematics functions
- Excel array type, e.g. `{cell1, cell2; cell3, cell4}`. It's not in use yet, but we may need it in some implementations

### Changed

- Change the Error type from `anyhow::Error` to `formula::error::Error` and remove the dep of anyhow

## 0.0.2 (2022-09-08)

### Added

- Support of basic MS Excel logical functions
- Support of most MS Excel web functions
- Support of arithmetic and comparison operators of MS Excel as functions. Use `AND(F.GT(1, 3), F.LT(1, 3))` instead of `AND(1>3, 1<3)`
- There is a new error type for not yet implemented functions, `Error::NotImplemented`
- There is a new expression variant for null values, `Expr::Null`

## 0.0.1 (2022-09-05)

### Added

- It's the initial release!
- Support of basic variable types, like `String`, `Number`, `Boolean`, `Date`, `Time`, `DateTime` and ...
- Support of basic MS Excel date and time functions
- Support of basic MS Excel text functions
