run:
	cargo run --bin=zerv

setup_dev:
	pre-commit install
	cargo install cargo-tarpaulin

update:
	rustup update
	cargo update

lint:
	cargo fmt -- --check || (cargo fmt && exit 1)
	cargo clippy -- -D warnings
	prettier --write "**/*.{ts,tsx,css,json,yaml,yml,md}"

test:
	cargo tarpaulin \
		--out Xml \
		--out Html \
		--output-dir coverage \
		--include-tests \
		--exclude-files src/main.rs

open_coverage:
	open coverage/tarpaulin-report.html
