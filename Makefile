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

test_flaky:
	@echo "🚀 Starting flaky test detection with 5 iterations..."
	@echo "=================================================="
	@start_time=$$(date +%s); \
	total_time=0; \
	success_count=0; \
	for i in 1 2 3 4 5; do \
		echo ""; \
		echo "=== Iteration $$i/5 ==="; \
		iter_start=$$(date +%s); \
		echo "⏰ Start time: $$(date)"; \
		echo "🔍 Running tests..."; \
		if ZERV_TEST_NATIVE_GIT=false ZERV_TEST_DOCKER=true $(MAKE) _test 2>&1 | tee /tmp/test_output_$$i.log; then \
			iter_end=$$(date +%s); \
			iter_duration=$$((iter_end - iter_start)); \
			total_time=$$((total_time + iter_duration)); \
			success_count=$$((success_count + 1)); \
			echo "✅ Iteration $$i completed successfully in $${iter_duration}s"; \
			echo "⏰ End time: $$(date)"; \
			rm -f /tmp/test_output_$$i.log; \
		else \
			iter_end=$$(date +%s); \
			iter_duration=$$((iter_end - iter_start)); \
			echo ""; \
			echo "❌ FAILED at iteration $$i after $${iter_duration}s"; \
			echo "⏰ End time: $$(date)"; \
			echo ""; \
			echo "🚨 FLAKINESS DETECTED - Test failed at iteration $$i"; \
			echo "Total successful iterations: $$((i-1))/5"; \
			echo ""; \
			echo "📋 FAILURE DETAILS:"; \
			echo "=================================================="; \
			if [ -f /tmp/test_output_$$i.log ]; then \
				echo "Last 50 lines of test output:"; \
				tail -50 /tmp/test_output_$$i.log; \
				echo ""; \
				echo "Full test output saved to: /tmp/test_output_$$i.log"; \
			else \
				echo "No test output captured"; \
			fi; \
			echo "=================================================="; \
			exit 1; \
		fi; \
	done; \
	end_time=$$(date +%s); \
	total_duration=$$((end_time - start_time)); \
	avg_time=$$((total_time / success_count)); \
	echo ""; \
	echo "=================================================="; \
	echo "✅ ALL 5 ITERATIONS PASSED - NO FLAKINESS DETECTED"; \
	echo "📊 PERFORMANCE SUMMARY:"; \
	echo "   Total time: $${total_duration}s"; \
	echo "   Average iteration time: $${avg_time}s"; \
	echo "   Success rate: 5/5 (100%)"; \
	echo "=================================================="

open_coverage:
	open coverage/tarpaulin-report.html
