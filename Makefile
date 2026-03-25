.PHONY: build test lint clean install fmt check test-all tag

# Build the project
build:
	cargo build --release

# Build debug version
build-dev:
	cargo build

# Run tests
test:
	cargo test -q -- --test-threads=1

# Run tests with coverage
test-cov:
	cargo test -- --nocapture

# Run all tests (including doc tests)
test-all:
	cargo test --all

# Run linter
lint:
	cargo clippy -q -- -D warnings

# Format code
fmt:
	cargo fmt -- --check

# Auto-fix formatting
fmt-fix:
	cargo fmt

# Check everything (fmt + lint + test)
check: fmt lint test

# Clean build artifacts
clean:
	cargo clean

# Install locally
install:
	cargo install --path .

# Run the binary
run:
	cargo run --release -- $(filter-out $@,$(MAKECMDGOALS))

# Full pipeline: format, lint, test
ci: fmt lint test

# Create a version tag (usage: make tag VERSION=0.2.0)
tag:
ifndef VERSION
	$(error VERSION is required. Usage: make tag VERSION=0.2.0)
endif
	@sed -i '' 's/^version = ".*"/version = "$(VERSION)"/' Cargo.toml
	@cargo check -q 2>/dev/null
	@git add Cargo.toml Cargo.lock
	@git commit -m "bump version to $(VERSION)"
	@git tag v$(VERSION)
	@echo "Tagged v$(VERSION). Push with: git push origin main --tags"

# Help
help:
	@echo "Available targets:"
	@echo "  make build       - Build release binary"
	@echo "  make build-dev  - Build debug binary"
	@echo "  make test       - Run tests"
	@echo "  make test-all   - Run all tests including doc tests"
	@echo "  make lint       - Run clippy linter"
	@echo "  make fmt        - Check code formatting"
	@echo "  make fmt-fix    - Auto-fix formatting"
	@echo "  make check      - Full check (fmt + lint + test)"
	@echo "  make clean      - Clean build artifacts"
	@echo "  make install    - Install binary locally"
	@echo "  make ci         - Run CI pipeline (fmt + lint + test)"
	@echo "  make tag        - Create version tag (VERSION=x.y.z)"
	@echo "  make help       - Show this help"
