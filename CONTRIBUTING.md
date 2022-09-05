# Contributing

Thanks for contributing!

## Getting Started

### Required software

- [Rust toolchain](https://www.rust-lang.org/en-US/install.html)
- [`cargo-readme`](https://crates.io/crates/cargo-readme) (`cargo install cargo-readme`)
- [`cargo-udeps`](https://crates.io/crates/cargo-udeps) (`cargo install cargo-udeps`)
- [`cargo-outdated`](https://crates.io/crates/cargo-outdated) (`cargo install cargo-outdated`)
- [GNU Make](https://www.gnu.org/software/make/)

## Making Changes

- Before committing changes, make sure to run `make check` to check and format your changes
- Make sure to run `make test` to run the tests
- Add an entry to the `CHANGELOG.md` file
- The `README.md` is generated using `cargo-readme` automatically after the PR is merged into master.
- Pull Requests should be made against master.
