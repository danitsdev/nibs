.PHONY: fmt check lint test verify run

CARGO ?= cargo
CCACHE_DIR ?= $(CURDIR)/target/ccache

fmt:
	$(CARGO) fmt

check:
	$(CARGO) fmt --check
	CCACHE_DIR=$(CCACHE_DIR) $(CARGO) check --all-targets --all-features

lint:
	CCACHE_DIR=$(CCACHE_DIR) $(CARGO) clippy --all-targets --all-features -- -D warnings

test:
	CCACHE_DIR=$(CCACHE_DIR) $(CARGO) test

verify: check lint test

run:
	CCACHE_DIR=$(CCACHE_DIR) $(CARGO) run
