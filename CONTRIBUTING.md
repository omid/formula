# Contributing

Thanks for contributing!

## Getting Started

### Required software

- [Rust toolchain](https://www.rust-lang.org/en-US/install.html)
- [`cargo-readme`](https://crates.io/crates/cargo-readme) (`cargo install cargo-readme`)
- [`cargo-udeps`](https://crates.io/crates/cargo-udeps) (`cargo install cargo-udeps`)
- [`cargo-outdated`](https://crates.io/crates/cargo-outdated) (`cargo install cargo-outdated`)
- [`cargo-llvm-cov`](https://crates.io/crates/cargo-llvm-cov) (`cargo install cargo-llvm-cov`), for `make coverage`
- [Node.js](https://nodejs.org/), the `wasm32-unknown-unknown` target (`rustup target add wasm32-unknown-unknown`)
  and [`wasm-bindgen-cli`](https://crates.io/crates/wasm-bindgen-cli) with the same version as the `wasm-bindgen` crate
  (`cargo install wasm-bindgen-cli --version $(cargo pkgid wasm-bindgen | cut -d@ -f2)`), for `make test_wasm`
- [GNU Make](https://www.gnu.org/software/make/)

## Making Changes

- Before committing changes, make sure to run `make check` to check and format your changes
- Make sure to run `make test` to run the tests, and `make test_wasm` if you change `formula-wasm`
- Run `make coverage` to see the test coverage and the lines that the tests miss
- If you change how formulas are evaluated, compare `make bench` before and after your change, see the `Makefile`
- Add an entry to the `CHANGELOG.md` file
- The `README.md` is generated using `cargo-readme` automatically after the PR is merged into main.
- Pull Requests should be made against main.
