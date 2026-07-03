.PHONY: all build release deb appimage run clean

all: release

build:
	cargo build --manifest-path src-tauri/Cargo.toml

release:
	cargo build --release --manifest-path src-tauri/Cargo.toml

deb:
	npx tauri build --bundles deb

appimage:
	npx tauri build --bundles appimage

run:
	cargo run --manifest-path src-tauri/Cargo.toml

clean:
	cargo clean --manifest-path src-tauri/Cargo.toml
	rm -rf src-tauri/target
