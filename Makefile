.PHONY: default build clean clippy doc format release run skeptic test

CARGO_FLAGS := --features "$(NS_FEATURES)"
SPECIAL_FILES := examples/asr_simple.rs

default: build

build:
	cargo build $(CARGO_FLAGS)

clean:
	cargo clean
	cargo clean --release

clippy:
	if $$CLIPPY; then cargo clippy $(CARGO_FLAGS); fi

doc: build
	cargo doc --no-deps $(CARGO_FLAGS)

format:
	cargo fmt

release:
	cargo build --release $(CARGO_FLAGS)

run: build
	DYLD_FRAMEWORK_PATH="SpeechSDK/macos_sdk" cargo run --example asr_simple

skeptic:
	USE_SKEPTIC=1 cargo test $(CARGO_FLAGS)

test: build
	cargo test $(CARGO_FLAGS)

special:
	git update-index --no-assume-unchanged $(SPECIAL_FILES)
	git add $(SPECIAL_FILES)
	git update-index --assume-unchanged $(SPECIAL_FILES)
