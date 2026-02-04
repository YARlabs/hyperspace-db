#!/bin/bash
set -e

# Run from sdks/ts/
cd "$(dirname "$0")"

mkdir -p src/proto

# Generate
./node_modules/.bin/grpc_tools_node_protoc \
    --js_out=import_style=commonjs,binary:src/proto \
    --grpc_out=grpc_js:src/proto \
    --ts_out=grpc_js:src/proto \
    --plugin=protoc-gen-grpc=./node_modules/.bin/grpc_tools_node_protoc_plugin \
    --plugin=protoc-gen-ts=./node_modules/.bin/protoc-gen-ts \
    -I ../../crates/hyperspace-proto/proto \
    ../../crates/hyperspace-proto/proto/hyperspace.proto

# Build TS
npm run build

# Copy proto JS files to dist (TSC doesn't do it)
mkdir -p dist/proto
cp src/proto/*.js dist/proto/

echo "TS Protos generated and built."
