#!/bin/bash
set -e

# Run from sdks/python/
cd "$(dirname "$0")"

# Activate venv if exists
if [ -d "venv" ]; then
    source venv/bin/activate
fi

python3 -m grpc_tools.protoc \
    -I ../../crates/hyperspace-proto/proto \
    --python_out=hyperspace/proto \
    --grpc_python_out=hyperspace/proto \
    ../../crates/hyperspace-proto/proto/hyperspace.proto

# Fix import in generated grpc file
if [[ "$OSTYPE" == "darwin"* ]]; then
  sed -i '' 's/import hyperspace_pb2 as hyperspace__pb2/from . import hyperspace_pb2 as hyperspace__pb2/' hyperspace/proto/hyperspace_pb2_grpc.py
else
  sed -i 's/import hyperspace_pb2 as hyperspace__pb2/from . import hyperspace_pb2 as hyperspace__pb2/' hyperspace/proto/hyperspace_pb2_grpc.py
fi

echo "Python protos generated and patched."
