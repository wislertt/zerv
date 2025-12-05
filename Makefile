run:
	cargo run --bin=zerv

setup_dev:
	pre-commit install
	cargo install cargo-tarpaulin

update:
	rustup update
	cargo update

lint:
	npx prettier --write "**/*.{ts,tsx,css,json,yaml,yml,md}"
	cargo +nightly check --tests
	cargo +nightly fmt -- --check || (cargo +nightly fmt && exit 1)
	cargo +nightly clippy --all-targets --all-features -- -D warnings

_test:
	RUST_BACKTRACE=1 RUST_LOG=cargo_tarpaulin=off cargo tarpaulin \
		--features test-utils \
		--out Xml --out Html --out Lcov \
		--output-dir coverage \
		--include-tests \
		--exclude-files 'src/main.rs' \
		--exclude-files '**/tests/**' \
		--exclude-files 'src/test_utils/git/native.rs' \
		-- --quiet

# Quick testing without Docker tests
test_easy:
	ZERV_TEST_NATIVE_GIT=false ZERV_TEST_DOCKER=false $(MAKE) _test

# Full test suite with Docker tests
test:
	ZERV_TEST_NATIVE_GIT=false ZERV_TEST_DOCKER=true $(MAKE) _test

open_coverage:
	open coverage/tarpaulin-report.html

.PHONY: docs
docs:
	cargo xtask generate-docs

extract_mermaid_svgs:
	@./scripts/extract_mermaid_from_markers.sh
