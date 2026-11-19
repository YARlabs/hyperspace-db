#!/bin/bash
set -e

# Run from sdks/python/
cd "$(dirname "$0")"

# Activate venv if exists
if [ -d ".venv" ]; then
    source .venv/bin/activate
elif [ -d "venv" ]; then
    source venv/bin/activate
fi

python3 -m grpc_tools.protoc \
    -I ../../crates/hyperspace-proto/proto \
    --python_out=hyperspace/proto \
    --grpc_python_out=hyperspace/proto \
    ../../crates/hyperspace-proto/proto/hyperspace.proto

# Fix import and minimum grpcio version in generated grpc file
if [[ "$OSTYPE" == "darwin"* ]]; then
  sed -i '' 's/import hyperspace_pb2 as hyperspace__pb2/from . import hyperspace_pb2 as hyperspace__pb2/' hyperspace/proto/hyperspace_pb2_grpc.py
  sed -i '' "s/GRPC_GENERATED_VERSION = '.*'/GRPC_GENERATED_VERSION = '1.63.0'/" hyperspace/proto/hyperspace_pb2_grpc.py
else
  sed -i 's/import hyperspace_pb2 as hyperspace__pb2/from . import hyperspace_pb2 as hyperspace__pb2/' hyperspace/proto/hyperspace_pb2_grpc.py
  sed -i "s/GRPC_GENERATED_VERSION = '.*'/GRPC_GENERATED_VERSION = '1.63.0'/" hyperspace/proto/hyperspace_pb2_grpc.py
fi

# Keep the relative-import fallback and a lenient grpc version probe (older grpc lacks
# grpc._utilities); these were hand-patched in earlier releases.
python3 - <<'PY'
p = "hyperspace/proto/hyperspace_pb2_grpc.py"
s = open(p).read()
s = s.replace(
    "from . import hyperspace_pb2 as hyperspace__pb2\n",
    "try:\n    from . import hyperspace_pb2 as hyperspace__pb2\nexcept ImportError:\n    import hyperspace_pb2 as hyperspace__pb2\n",
    1,
)
s = s.replace("except ImportError:\n    _version_not_supported = True", "except ImportError:\n    _version_not_supported = False", 1)
open(p, "w").write(s)
PY

echo "Python protos generated and patched."
