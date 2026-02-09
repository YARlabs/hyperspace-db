#!/bin/bash
set -e

VERSION="1.5.0"
OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)
ARCHIVE_NAME="hyperspace-db-v$VERSION-$OS-$ARCH.tar.gz"

echo "🚀 Preparing HyperspaceDB v$VERSION Release..."
echo "ℹ️  Host: $OS-$ARCH"

# 1. Run Tests (Fast check)
echo "🧪 Running Tests (Hyperspace Core)..."
cargo test -p hyperspace-core --release

# 2. Build Release Binaries
echo "🔨 Building Release Binaries..."
cargo build --release -p hyperspace-server -p hyperspace-cli

# Use target directory for staging (cleaner)
STAGING_DIR="target/release_pkg"
rm -rf "$STAGING_DIR"
mkdir -p "$STAGING_DIR"

cp target/release/hyperspace-server "$STAGING_DIR/"
cp target/release/hyperspace-cli "$STAGING_DIR/"

# 3. Create Archive (in root, or target? Keeping archive in root is fine for output, but build artifacts in target)
echo "📦 Creating Release Archive: $ARCHIVE_NAME"
tar -czf "$ARCHIVE_NAME" -C "$STAGING_DIR" .
echo "✅ Archive created: $ARCHIVE_NAME"

# 4. Docker Build (Multi-arch verification)
echo "🐳 Building Docker Image (amd64 & arm64)..."
if docker buildx version >/dev/null 2>&1; then
    echo "   Using docker buildx..."
    # We use --platform but cannot --load multi-arch. We build to cache.
    docker buildx build --platform linux/amd64,linux/arm64 -t glukhota/hyperspace-db:latest -t glukhota/hyperspace-db:$VERSION .
else
    echo "⚠️  docker buildx not found. Falling back to standard build."
    docker build -t glukhota/hyperspace-db:latest -t glukhota/hyperspace-db:$VERSION .
fi

echo "✅ Docker build complete (cached)."
echo "ℹ️  To push multi-arch: docker buildx build --platform linux/amd64,linux/arm64 -t glukhota/hyperspace-db:$VERSION --push ."

echo "🎉 Release v$VERSION Preparation Complete!"
