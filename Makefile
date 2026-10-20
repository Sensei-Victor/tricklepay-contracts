# soroban-sdk requires wasm32v1-none on Rust 1.82+; wasm32-unknown-unknown
# enables wasm features the Soroban environment does not support.
WASM_TARGET := wasm32v1-none
WASM := target/$(WASM_TARGET)/release/tricklepay_stream.wasm

.PHONY: all build wasm size test fmt fmt-check lint docs audit clean deploy verify-deployment check-markdown-links check-release-overflow snapshot check help

# help is the default target; running plain `make` lists available targets.
.DEFAULT_GOAL := help

# Check the release overflow setting and the built contract interface.
# Use this before opening a pull request.
check: check-release-overflow ## Check release overflow settings and the contract interface.
	./scripts/diff-interface.sh

all: check ## Alias for `check`.

check-release-overflow: ## Verify release builds keep overflow checks enabled.
	bash ./scripts/check-release-overflow.sh

verify-deployment: ## Compare a deployed contract's WASM with a local build.
	@if [ -z "$(CONTRACT_ID)" ] || [ -z "$(NETWORK)" ]; then echo "Usage: make verify-deployment CONTRACT_ID=C... NETWORK=testnet"; exit 2; fi
	./scripts/verify-deployment.sh "$(CONTRACT_ID)" "$(NETWORK)"

check-markdown-links: ## Check relative Markdown files and heading anchors.
	python3 scripts/check-markdown-links.py

snapshot: wasm ## Regenerate the checked-in contract interface snapshot.
	stellar contract inspect --wasm $(WASM) > docs/interface.txt

# Native debug build.
build: ## Native debug build.
	cargo build

wasm: ## Optimised WASM artifact for deployment.
	cargo build --release --target $(WASM_TARGET)
	@echo "built $(WASM)"

size: wasm ## Report the compiled contract's WASM size (builds it first if needed).
	@echo "$(WASM): $$(wc -c < $(WASM) | tr -d ' ') bytes ($$(du -h $(WASM) | cut -f1))"

test: ## Run the full test suite.
	cargo test

fmt: ## Format the workspace in place.
	cargo fmt

fmt-check: ## Verify formatting without modifying files (used in CI).
	cargo fmt --check

lint: ## Lint every target and treat warnings as errors.
	cargo clippy --all-targets -- -D warnings

docs: ## Build crate documentation, treating warnings as errors.
	RUSTDOCFLAGS="-D warnings" cargo doc --no-deps

audit: ## Audit dependencies for known vulnerabilities.
	cargo audit --deny warnings --file .cargo/audit.toml

clean: ## Remove build artifacts.
	cargo clean

deploy: ## Build, install, and deploy to testnet. Pass an identity: make deploy IDENTITY=alice
ifndef IDENTITY
	$(error IDENTITY is undefined. Please provide an identity argument (e.g. make deploy IDENTITY=alice))
endif
	./scripts/deploy.sh $(IDENTITY)

help: ## Show this help message.
	@echo "Usage: make <target>"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*##' $(MAKEFILE_LIST) \
		| awk 'BEGIN {FS = ":.*##"}; {printf "  %-14s %s\n", $$1, $$2}'
