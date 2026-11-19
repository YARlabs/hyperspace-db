#!/usr/bin/env bash
# Profile-Guided Optimization (PGO) release build for hyperspace-server.
#
# 1. Builds an instrumented binary set (-Cprofile-generate).
# 2. Runs a training workload that exercises the HNSW insert / search / filter /
#    quantization hot paths (release test suites of hyperspace-core + hyperspace-index,
#    optionally your own workload via PGO_TRAIN_CMD).
# 3. Merges the raw profiles with llvm-profdata (rustup component `llvm-tools-preview`,
#    already listed in rust-toolchain.toml).
# 4. Rebuilds hyperspace-server with -Cprofile-use.
#
# Usage:
#   scripts/pgo_build.sh
#   PGO_TRAIN_CMD="./target/release/hyperspace-server & sleep 2; python benchmarks/run_l2_1m.py" scripts/pgo_build.sh
#
# Branch-heavy code (mode × metric distance dispatch, heap maintenance, filter
# evaluation) typically gains 10–20% throughput from PGO.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

PGO_DIR="${PGO_DIR:-$ROOT/target/pgo-profiles}"
MERGED="$PGO_DIR/merged.profdata"
BASE_RUSTFLAGS="${RUSTFLAGS:-}"

HOST="$(rustc -vV | sed -n 's/^host: //p')"
PROFDATA="$(rustc --print sysroot)/lib/rustlib/$HOST/bin/llvm-profdata"
if [[ ! -x "$PROFDATA" ]]; then
  echo "llvm-profdata not found at $PROFDATA" >&2
  echo "Install it with: rustup component add llvm-tools-preview" >&2
  exit 1
fi

rm -rf "$PGO_DIR"
mkdir -p "$PGO_DIR"

echo "==> [1/4] Instrumented build"
export RUSTFLAGS="$BASE_RUSTFLAGS -Cprofile-generate=$PGO_DIR"
cargo build --release -p hyperspace-server

echo "==> [2/4] Training workload"
cargo test --release -p hyperspace-core -p hyperspace-index -- --test-threads=4
if [[ -n "${PGO_TRAIN_CMD:-}" ]]; then
  bash -c "$PGO_TRAIN_CMD"
fi

echo "==> [3/4] Merging profiles"
"$PROFDATA" merge -o "$MERGED" "$PGO_DIR"

echo "==> [4/4] Optimized build (-Cprofile-use)"
export RUSTFLAGS="$BASE_RUSTFLAGS -Cprofile-use=$MERGED"
cargo build --release -p hyperspace-server

echo "PGO build ready: target/release/hyperspace-server (profile: $MERGED)"
