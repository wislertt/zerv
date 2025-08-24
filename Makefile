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

# Internal helper - no environment variables set
_test:
	RUST_BACKTRACE=1 cargo tarpaulin \
		--features test-utils \
		--out Xml --out Html --out Lcov \
		--output-dir coverage \
		--include-tests --exclude-files src/main.rs

# Quick testing without Docker tests
test_easy:
	ZERV_TEST_NATIVE_GIT=false ZERV_TEST_DOCKER=false $(MAKE) _test

# Full test suite with Docker tests
test:
	ZERV_TEST_NATIVE_GIT=false ZERV_TEST_DOCKER=true $(MAKE) _test

open_coverage:
	open coverage/tarpaulin-report.html
