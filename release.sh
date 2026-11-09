#!/bin/bash
set -e

VERSION="${1:-4.2.3}"

OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)
ARCHIVE_NAME="hyperspace-db-v$VERSION-$OS-$ARCH.tar.gz"

echo "🚀 Publishing HyperspaceDB v$VERSION..."
echo "ℹ️  Host: $OS-$ARCH"

# Sync VERSION in Dockerfiles
echo "🔄 Synchronizing version v$VERSION in Dockerfiles..."
for df in Dockerfile Dockerfile.amd64 Dockerfile.arm64 Dockerfile.embed Dockerfile.saas; do
    if [ -f "$df" ]; then
        sed -i.bak -E "s/ARG VERSION=[0-9.]+/ARG VERSION=$VERSION/g" "$df" 2>/dev/null || true
        rm -f "${df}.bak"
    fi
done

# 0. Build Dashboard UI (embedded via rust-embed into hyperspace-server)
echo "🎨 Building Dashboard UI (including PDK Plugins Workbench)..."
(cd dashboard && npm run build)

# 1. External PDK Plugins are decoupled and distributed via Hyperspace Hub
# (Installed dynamically via 'hyperspace plugin install <plugin-id>')

# 2. Run Quality Checks (Sync with CI)
echo "🧪 Running Quality Checks..."
cargo fmt --all -- --check || { echo "❌ Formatting errors found! Run 'cargo fmt --all' to fix."; exit 1; }
cargo clippy --all-targets --all-features -- -D warnings || { echo "❌ Clippy warnings found!"; exit 1; }
cargo clippy --tests --workspace -- -W clippy::pedantic || { echo "❌ Clippy pedantic warnings found!"; exit 1; }
cargo test --workspace --release || { echo "❌ Tests failed!"; exit 1; }

# 3. Build Release Binaries
echo "🔨 Building Release Binaries (hyperspace-server, hyperspace-cli)..."
cargo build --release -p hyperspace-server -p hyperspace-cli

# Use target directory for staging (cleaner)
STAGING_DIR="target/release_pkg"
rm -rf "$STAGING_DIR"
mkdir -p "$STAGING_DIR/plugins"

cp target/release/hyperspace-server "$STAGING_DIR/"
cp target/release/hyperspace-cli "$STAGING_DIR/"
if [ -f "plugins/plugins.toml" ]; then
    cp plugins/plugins.toml "$STAGING_DIR/plugins/"
fi
if [ -f "plugins/plugins.toml.example" ]; then
    cp plugins/plugins.toml.example "$STAGING_DIR/plugins/"
fi

# 4. Create Archive
echo "📦 Creating Release Archive: $ARCHIVE_NAME"
tar -czf "$ARCHIVE_NAME" -C "$STAGING_DIR" .
echo "✅ Archive created: $ARCHIVE_NAME (includes hyperspace-server, hyperspace-cli, and ./plugins/)"

# 5. Docker Build & Push (Multi-arch)
echo "🐳 Building & Pushing Docker Image (amd64 & arm64)..."
if ! docker buildx inspect hyperspace-builder >/dev/null 2>&1; then
    docker buildx create --name hyperspace-builder --use
else
    docker buildx use hyperspace-builder
fi

if docker buildx version >/dev/null 2>&1; then
    echo "🐳 Building & Pushing Standard Base Image (amd64 & arm64, with PDK & external plugins)..."
    docker buildx build --platform linux/amd64,linux/arm64 \
        --build-arg VERSION="$VERSION" \
        -f Dockerfile \
        -t glukhota/hyperspace-db:latest \
        -t glukhota/hyperspace-db:$VERSION \
        -t ghcr.io/yarlabs/hyperspace-db:latest \
        -t ghcr.io/yarlabs/hyperspace-db:$VERSION \
        --push .

    echo "⚡ Building & Pushing SaaS Image (AMD64 optimized, s3-tiering, PDK & external plugins)..."
    docker buildx build --platform linux/amd64 \
        --build-arg VERSION="$VERSION" \
        -f Dockerfile.saas \
        -t glukhota/hyperspace-db:latest-saas \
        -t glukhota/hyperspace-db:${VERSION}-saas \
        -t ghcr.io/yarlabs/hyperspace-db:latest-saas \
        -t ghcr.io/yarlabs/hyperspace-db:${VERSION}-saas \
        --push .

    echo "🧠 Building & Pushing All-in-One Image with Unified hs-embed ONNX Pipeline (amd64 & arm64)..."
    docker buildx build --platform linux/amd64,linux/arm64 \
        --build-arg VERSION="$VERSION" \
        -f Dockerfile.embed \
        -t glukhota/hyperspace-db:latest-embed \
        -t glukhota/hyperspace-db:${VERSION}-embed \
        -t ghcr.io/yarlabs/hyperspace-db:latest-embed \
        -t ghcr.io/yarlabs/hyperspace-db:${VERSION}-embed \
        --push .
else
    echo "❌ docker buildx not found. Cannot push images."
    exit 1
fi
echo "✅ Docker images pushed."

# 6. Git Release
echo "🐙 Deploying to GitHub..."
git add .
git commit -m "chore: release v$VERSION artifacts" || echo "Nothing to commit"
git push origin HEAD
if git rev-parse "v$VERSION" >/dev/null 2>&1; then
    echo "ℹ️  Tag v$VERSION already exists. Skipping tag creation."
else
    git tag "v$VERSION"
fi
git push origin "v$VERSION"
echo "✅ GitHub release deployed."

echo "🎉 Release v$VERSION Complete! All artifacts published."
