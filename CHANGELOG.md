# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased

### Added

- `NA()` function

### Changed

- MSRV is now Rust 1.99
- Upgraded dependencies, and replaced `rand` with `getrandom`
- Invalid input in many functions returns an error instead of panicking, e.g. `EDATE`, `EOMONTH`, `RIGHT`, `MID`, `REPLACE`, `FINDB`, `SEARCHB`, `YEAR('2020')`, `REPT` with a huge count
- `WEEKNUM` and `FILTERXML` return `Error::NotImplemented` instead of panicking
- Text after a complete formula is an error, e.g. `=1+1` used to return `1`
- Strings can contain the other quote character, e.g. `="it's"`
- **Breaking:** spreadsheet errors are `Expr::Error(ExprError)` values (`#DIV/0!`, `#NUM!`, `#N/A`) instead of `Expr::Null`.
  Like Excel, they pass through the functions that use them, until `IFERROR`, `IFNA` or the result
- Division by zero is `#DIV/0!`, and results that are not finite numbers (e.g. `SQRT(-1)`, `LN(0)`) are `#NUM!`
- `F.EQ`, `F.NE` and the other comparisons agree with each other, and ignore floating point noise like Excel
- Text comparisons are case-insensitive like Excel, e.g. `F.EQ('a', 'A')` is `TRUE`. `EXACT` stays case-sensitive
- `IF` only evaluates the selected branch, and returns `FALSE` when the false branch is missing
- `IFERROR` catches spreadsheet errors, `IFNA` only catches `#N/A` and returns the value otherwise
- `WEEKDAY` uses Excel's numbering and supports the return type argument
- Excel compatible `MOD`, `FIXED`, `TRIM`, `PROPER`, `TEXTJOIN`, `CHAR`, `UNICHAR`, `CODE`, `UNICODE`, `FIND` and `SEARCH`
- Date and time functions accept the result of `NOW()`
- `DATE` and `TIME` roll over like Excel, e.g. `DATE(2020, 13, 1)` is 2021-01-01, `DATE(2020, 1, 0)` is 2019-12-31
  and `TIME(25, 0, 0)` is 01:00. Out of range results are `#NUM!`
- `WEBSERVICE` returns an error on HTTP error responses, and on WASM
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
