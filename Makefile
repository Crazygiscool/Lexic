.PHONY: all build test check clippy fmt fmt-check run clean docs

CARGO ?= cargo
CARGO_FLAGS ?=

all: check test

build:
	$(CARGO) build --workspace $(CARGO_FLAGS)

test:
	$(CARGO) test --workspace $(CARGO_FLAGS)

check: fmt-check clippy

clippy:
	$(CARGO) clippy --workspace --all-targets -- $(CARGO_FLAGS) -- -D warnings

fmt:
	$(CARGO) fmt --all

fmt-check:
	$(CARGO) fmt --all -- --check

run:
	$(CARGO) run -p lexic-gui $(CARGO_FLAGS)

run-release:
	$(CARGO) run --release -p lexic-gui $(CARGO_FLAGS)

clean:
	$(CARGO) clean

docs:
	$(CARGO) doc --workspace --no-deps $(CARGO_FLAGS)
