#!/bin/bash
set -e

VERSION="1.6.0"

OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)
ARCHIVE_NAME="hyperspace-db-v$VERSION-$OS-$ARCH.tar.gz"

echo "🚀 Publishing HyperspaceDB v$VERSION..."
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

# 3. Create Archive
echo "📦 Creating Release Archive: $ARCHIVE_NAME"
tar -czf "$ARCHIVE_NAME" -C "$STAGING_DIR" .
echo "✅ Archive created: $ARCHIVE_NAME"

# 4. Docker Build & Push (Multi-arch)
echo "🐳 Building & Pushing Docker Image (amd64 & arm64)..."
# Ensure builder exists
if ! docker buildx inspect hyperspace-builder >/dev/null 2>&1; then
    docker buildx create --name hyperspace-builder --use
else
    docker buildx use hyperspace-builder
fi

if docker buildx version >/dev/null 2>&1; then
    docker buildx build --platform linux/amd64,linux/arm64 \
        -t glukhota/hyperspace-db:latest \
        -t glukhota/hyperspace-db:$VERSION \
        -t ghcr.io/yarlabs/hyperspace-db:latest \
        -t ghcr.io/yarlabs/hyperspace-db:$VERSION \
        --push .
else
    echo "❌ docker buildx not found. Cannot push multi-arch."
    exit 1
fi
echo "✅ Docker images pushed."

# 5. Git Release
echo "🐙 Deploying to GitHub..."
git add .
# Commit any pending changes (e.g. version bumps)
git commit -m "chore: release v$VERSION artifacts" || echo "Nothing to commit"
git push origin HEAD
# Create tag if not exists
if git rev-parse "v$VERSION" >/dev/null 2>&1; then
    echo "ℹ️  Tag v$VERSION already exists. Skipping tag creation."
else
    git tag "v$VERSION"
fi
git push origin "v$VERSION"
echo "✅ GitHub release deployed."

echo "🎉 Release v$VERSION Complete! All artifacts published."
