# === Stage 1: UI Builder ===
FROM node:slim as ui-builder
WORKDIR /app/dashboard
COPY dashboard/package*.json ./
RUN npm install
COPY dashboard/ .
RUN npm run build

# === Stage 2: Rust Builder ===
FROM rustlang/rust:nightly as builder

# Install protobuf compiler
RUN apt-get update && apt-get install -y protobuf-compiler cmake

WORKDIR /app
COPY . .
# Copy built UI assets to correct location
COPY --from=ui-builder /app/dashboard/dist ./dashboard/dist

# Remove toolchain file
RUN rm -f rust-toolchain.toml

# Build Release
RUN cargo build --release --workspace

# === Stage 3: Runtime ===
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -m -u 1000 -U -s /bin/sh -d /app hyperspace

WORKDIR /app

# Copy binaries
COPY --from=builder /app/target/release/hyperspace-server /usr/local/bin/
COPY --from=builder /app/target/release/hyperspace-cli /usr/local/bin/

# Create data dir
RUN mkdir -p /app/data && chown -R hyperspace:hyperspace /app

# Switch to non-root user
USER hyperspace

ENV RUST_LOG=info

# Label the image
LABEL org.opencontainers.image.source=https://github.com/yarlabs/hyperspace-db

# Expose ports
EXPOSE 50051
EXPOSE 50050

# Default command
CMD ["hyperspace-server"]
