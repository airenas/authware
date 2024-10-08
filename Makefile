-include Makefile.options
log?=INFO
###############################################################################
run/authware:
	RUST_LOG=$(log) cargo run --bin authware -- --redis-url=redis://localhost:6380 --encryption-key=1234567890123456asdasds
.PHONY: run/authware
###############################################################################
run/authware/inmemory:
	RUST_LOG=$(log) cargo run --bin authware -- --encryption-key=1234567890123456asdasds
.PHONY: run/authware
###############################################################################
build/local: 
	cargo build --release
.PHONY: build/local
###############################################################################
test/unit:
	RUST_LOG=DEBUG cargo test --bins --lib --no-fail-fast
.PHONY: test/unit
test/coverage:
	cargo tarpaulin --lib --bins --ignore-tests
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
## run integration tests
test/integration: 
	cd tests && ( $(MAKE) -j1 test/integration clean || ( $(MAKE) clean; exit 1; ))
.PHONY: test/integration
###############################################################################
.EXPORT_ALL_VARIABLES:

