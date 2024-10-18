test:
	@cargo test --all -- --nocapture
	@for example in examples/*.rs; do \
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

check_nightly: check
	@cargo +nightly clippy --fix --allow-dirty --allow-staged

check_strictly:
	@cargo +nightly clippy --fix --allow-dirty --allow-staged --all-features --all-targets -- -W clippy::all -W clippy::pedantic -W clippy::cargo -A clippy::missing_errors_doc -A clippy::extra_unused_lifetimes -A clippy::cast_sign_loss -A clippy::cast_possible_truncation -A clippy::missing-panics-doc -A clippy::module_name_repetitions -A clippy::cast_precision_loss -A clippy::cast_possible_wrap -A clippy::used_underscore_binding -A clippy::multiple_crate_versions -A clippy::option_option -A clippy::let_underscore_drop

check_very_strictly:
	@cargo +nightly clippy --fix --allow-dirty --allow-staged --all-features --all-targets -- -W clippy::all -W clippy::pedantic -W clippy::cargo -A clippy::cast_sign_loss -A clippy::cast_possible_truncation -A clippy::cast_precision_loss

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
