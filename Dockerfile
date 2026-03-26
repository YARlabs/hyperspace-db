# === Stage 1: UI Builder ===
FROM node:slim AS ui-builder
WORKDIR /app/dashboard
COPY dashboard/package*.json ./
RUN npm install
COPY dashboard/ .
RUN npm run build

# === Stage 2: Rust Builder ===
# We use Ubuntu 24.04 to get GLIBC 2.39+ (required by newest ORT binaries for __isoc23 symbols)
FROM ubuntu:24.04 AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    protobuf-compiler \
    cmake \
    clang \
    lld \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Install Rust nightly
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain nightly
ENV PATH="/root/.cargo/bin:${PATH}"

WORKDIR /app

# Remove toolchain and cargo config to avoid conflicts with build environment
COPY . .
RUN rm -f rust-toolchain.toml .cargo/config.toml

# Copy built UI assets to correct location
COPY --from=ui-builder /app/dashboard/dist ./dashboard/dist

# Build Release
# We use clang++ as linker for better C++ compatibility with 'ort'
ENV CC=clang
ENV CXX=clang++
ENV RUSTFLAGS="-C linker=clang++ -C link-arg=-fuse-ld=lld -C link-arg=-lstdc++"
RUN cargo build --release --workspace --features nightly-simd

# Strip binaries to reduce size
RUN strip target/release/hyperspace-server
RUN strip target/release/hyperspace-cli

# === Stage 3: Runtime ===
FROM ubuntu:24.04

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    openssl \
    libstdc++6 \
    libgomp1 \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -m -u 1001 -U -s /bin/sh -d /app hyperspace


WORKDIR /app

# Copy binaries
COPY --from=builder /app/target/release/hyperspace-server /usr/local/bin/
COPY --from=builder /app/target/release/hyperspace-cli /usr/local/bin/

# Create data dir
RUN mkdir -p /app/data && chown -R hyperspace:hyperspace /app

# Switch to non-root user
USER hyperspace

ENV RUST_LOG=info
ENV HS_DATA_DIR=/app/data

# Label the image
LABEL org.opencontainers.image.source=https://github.com/yarlabs/hyperspace-db

# Expose ports
EXPOSE 50051
EXPOSE 50050

# Default command
CMD ["hyperspace-server"]

