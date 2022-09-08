# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## Unreleased (2022-xx-yy)

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
