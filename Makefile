-include Makefile.options
log?=INFO
###############################################################################
run/authware:
	RUST_LOG=$(log) cargo run --bin authware -- --redis-url=redis://localhost:6380 --encryption-key=1234567890123456asdasds
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
## build docker for provided service
docker/%/build: 
	cd build/$* && $(MAKE) dbuild
.PHONY: docker/*/build	
###############################################################################
.EXPORT_ALL_VARIABLES:

