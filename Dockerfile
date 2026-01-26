# === Stage 1: Builder ===
FROM rustlang/rust:nightly as builder

# Install protobuf compiler
RUN apt-get update && apt-get install -y protobuf-compiler cmake

WORKDIR /app
COPY . .
# Remove toolchain file to use the container's default nightly and avoid os error 18
RUN rm -f rust-toolchain.toml

# Build Release
# Note: We build the workspace
RUN cargo build --release --workspace

# === Stage 2: Runtime ===
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binaries
COPY --from=builder /app/target/release/hyperspace-server /usr/local/bin/
COPY --from=builder /app/target/release/hyperspace-cli /usr/local/bin/

# Copy resources if needed (e.g. config files, though we use flags currently)
# Create data dir
RUN mkdir -p /app/data

ENV RUST_LOG=info

# Expose gRPC port
EXPOSE 50051

# Default command
CMD ["hyperspace-server"]
