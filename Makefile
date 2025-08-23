run:
	cargo run --bin=zerv

setup_dev:
	pre-commit install
	cargo install cargo-tarpaulin

update:
	rustup update
	cargo update

lint:
	cargo check
	cargo fmt -- --check || (cargo fmt && exit 1)
	cargo clippy --all-targets --all-features -- -D warnings
	npx prettier --write "**/*.{ts,tsx,css,json,yaml,yml,md}"

# Quick testing without external dependencies (Docker, etc.)
test_easy:
	cargo test

# Full test suite with coverage - requires cargo-tarpaulin
test:
	RUST_BACKTRACE=1 cargo tarpaulin \
		--features test-utils \
		--out Xml \
		--out Html \
		--out Lcov \
		--output-dir coverage \
		--include-tests \
		--exclude-files src/main.rs \
		-- --include-ignored

open_coverage:
	open coverage/tarpaulin-report.html
