# Justfile

default: build

# Build release
build:
    cargo build --release

# Run server locally
run:
    cargo run -p hyperspace-server

# Run TUI dashboard
tui:
    cargo run -p hyperspace-cli

# Generate Protobuf code
proto:
    cargo run -p hyperspace-proto --bin gen_proto

# Run SIMD tests
test-simd:
    RUSTFLAGS="-C target-cpu=native" cargo test -p hyperspace-core
