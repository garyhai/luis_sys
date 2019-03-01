.PHONY: default build clean clippy doc format release run skeptic test

CARGO_FLAGS := --features "$(NS_FEATURES)"

default: build

build:
	cargo build $(CARGO_FLAGS)

clean:
	cargo clean --release

clippy:
	if $$CLIPPY; then cargo clippy $(CARGO_FLAGS); fi

doc: build
	cargo doc --no-deps $(CARGO_FLAGS)

format:
	cargo fmt

release:
	cargo build --release $(CARGO_FLAGS)

run:
	cargo run --example asr_simple

skeptic:
	USE_SKEPTIC=1 cargo test $(CARGO_FLAGS)

test: build
	cargo test $(CARGO_FLAGS)
