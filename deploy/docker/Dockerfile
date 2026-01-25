# Stage 1: Builder
FROM rustlang/rust:nightly as builder
WORKDIR /app
COPY . .
# Build release mode
RUN cargo build --release --bin hyperspace-server

# Stage 2: Runtime (Distroless for security)
FROM gcr.io/distroless/cc-debian12
COPY --from=builder /app/target/release/hyperspace-server /
CMD ["./hyperspace-server"]
