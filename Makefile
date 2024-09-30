-include Makefile.options
log?=INFO
###############################################################################
run/authware:
	RUST_LOG=$(log) cargo run --bin authware -- --sample-user-pass=olia1 --redis-url=redis://localhost:6380
.PHONY: run/authware
###############################################################################
build/local: 
	cargo build --release
.PHONY: build/local
###############################################################################
test/unit:
	RUST_LOG=DEBUG cargo test --no-fail-fast
.PHONY: test/unit
test/coverage:
	cargo tarpaulin --ignore-tests
.PHONY: test/coverage
.PHONY: test/unit	
test/lint:
	@cargo clippy -V
	cargo clippy --all-targets --all-features -- -D warnings
.PHONY: test/lint	
test/format:
	cargo fmt -- --check
.PHONY: test/format
audit:
	cargo audit
.PHONY: audit
install/checks:
	cargo install cargo-audit
	cargo install cargo-tarpaulin
.PHONY: install/checks
###############################################################################

.EXPORT_ALL_VARIABLES:

