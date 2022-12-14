# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
