#!/bin/bash
# Generate Python protobuf files from hyperspace.proto

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROTO_DIR="$SCRIPT_DIR/src/langchain_hyperspace/proto"
OUT_DIR="$SCRIPT_DIR/src/langchain_hyperspace/generated"

# Create output directory
mkdir -p "$OUT_DIR"

# Generate Python protobuf and gRPC code
python -m grpc_tools.protoc \
    -I"$PROTO_DIR" \
    --python_out="$OUT_DIR" \
    --grpc_python_out="$OUT_DIR" \
    --pyi_out="$OUT_DIR" \
    "$PROTO_DIR/hyperspace.proto"

# Fix imports in generated files (grpc_tools generates incorrect relative imports)
if [[ "$OSTYPE" == "darwin"* ]]; then
    # macOS
    sed -i '' 's/^import hyperspace_pb2/from . import hyperspace_pb2/' "$OUT_DIR/hyperspace_pb2_grpc.py"
else
    # Linux
    sed -i 's/^import hyperspace_pb2/from . import hyperspace_pb2/' "$OUT_DIR/hyperspace_pb2_grpc.py"
fi

# Create __init__.py
cat > "$OUT_DIR/__init__.py" << 'EOF'
"""Generated protobuf files for HyperspaceDB."""

from langchain_hyperspace.generated import hyperspace_pb2, hyperspace_pb2_grpc

__all__ = ["hyperspace_pb2", "hyperspace_pb2_grpc"]
EOF

echo "✅ Protobuf files generated successfully in $OUT_DIR"
