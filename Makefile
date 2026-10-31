# The version of the formula crate, or the new one for `make bump VERSION=x.y.z`
VERSION ?= $(shell sed -n 's/^version = "\(.*\)"/\1/p' formula/Cargo.toml)

test:
	@cargo test --all -- --nocapture
	@for example in formula/examples/*.rs; do \
         cargo run --example "$$(basename "$${example%.rs}")" -- $$args; \
     done
	@cargo doc --no-deps --all-features --examples

check:
	@cargo +nightly fmt --all
	@cargo clippy --fix --allow-dirty --allow-staged --all-targets -- -D warnings -A clippy::extra_unused_lifetimes
	@cargo update --dry-run
	@cargo outdated -wR
	@cargo +nightly udeps --all-targets
	@cd formula && cargo readme --no-title --no-license > ../README.md

check_nightly:
	@cargo +nightly clippy --fix --allow-dirty --allow-staged

check_strictly:
	@cargo +nightly clippy --fix --allow-dirty --allow-staged --all-features --all-targets -- -W clippy::all -W clippy::pedantic -W clippy::cargo -A clippy::missing_errors_doc -A clippy::extra_unused_lifetimes -A clippy::cast_sign_loss -A clippy::cast_possible_truncation -A clippy::missing-panics-doc -A clippy::module_name_repetitions -A clippy::cast_precision_loss -A clippy::cast_possible_wrap -A clippy::used_underscore_binding -A clippy::multiple_crate_versions -A clippy::option_option -A let_underscore_drop

check_very_strictly:
	@cargo +nightly clippy --fix --allow-dirty --allow-staged --all-features --all-targets -- -W clippy::all -W clippy::pedantic -W clippy::cargo -A clippy::cast_sign_loss -A clippy::cast_possible_truncation -A clippy::cast_precision_loss

# Runs formula-wasm's tests in Node, with wasm-bindgen-cli of the same version as the wasm-bindgen crate
test_wasm:
	@CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER=wasm-bindgen-test-runner cargo test -p formula-wasm --target wasm32-unknown-unknown

wasm_example:
	@cd formula-wasm; \
		npm install; \
		npm run serve

wasm_pack_and_publish:
	@cd formula-wasm; \
		cargo build --release; \
		wasm-pack build --release; \
		wasm-pack pack; \
		wasm-pack publish

# Sets the version of both crates and dates the unreleased changes in the changelog, e.g. `make bump VERSION=0.2.0`
bump:
	@test "$(origin VERSION)" = "command line" || { echo "Usage: make bump VERSION=x.y.z"; exit 1; }
	@grep -q '^## Unreleased$$' CHANGELOG.md || { echo "CHANGELOG.md has no Unreleased section"; exit 1; }
	@perl -pi -e 's/^version = ".*"/version = "$(VERSION)"/' formula/Cargo.toml formula-wasm/Cargo.toml
	@perl -pi -e 's/^formula = \{ version = ".*" \}/formula = { version = "$(VERSION)" }/' formula-wasm/Cargo.toml
	@perl -pi -e 's/^## Unreleased$$/## $(VERSION) ($(shell date +%F))/' CHANGELOG.md

# Tags the current version and creates its GitHub release, which publishes the crate to crates.io.
# Run it on main after the `make bump` changes are merged.
release:
	@test -z "$$(git status --porcelain)" || { echo "Commit or stash your changes first"; exit 1; }
	@test "$$(git branch --show-current)" = main || { echo "Release from the main branch"; exit 1; }
	@git fetch -q origin main
	@test "$$(git rev-parse HEAD)" = "$$(git rev-parse origin/main)" || { echo "main differs from origin/main"; exit 1; }
	@grep -q '^## $(VERSION) ' CHANGELOG.md || { echo "CHANGELOG.md has no $(VERSION) section, run make bump first"; exit 1; }
	@cargo publish --dry-run -q -p formula
	@git tag v$(VERSION)
	@git push -q origin v$(VERSION)
	@awk '/^## $(VERSION) /{f=1; next} /^## /{f=0} f' CHANGELOG.md \
		| gh release create v$(VERSION) --verify-tag --title v$(VERSION) --notes-file -
	@echo "Released v$(VERSION), publish the npm package with: make wasm_pack_and_publish"
