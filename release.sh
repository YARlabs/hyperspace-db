#!/bin/bash
set -e

VERSION="4.2.0"

OS=$(uname -s | tr '[:upper:]' '[:lower:]')
ARCH=$(uname -m)
ARCHIVE_NAME="hyperspace-db-v$VERSION-$OS-$ARCH.tar.gz"

echo "🚀 Publishing HyperspaceDB v$VERSION..."
echo "ℹ️  Host: $OS-$ARCH"

# 0. Build Dashboard UI (embedded via rust-embed into hyperspace-server)
echo "🎨 Building Dashboard UI (including PDK Plugins Workbench)..."
(cd dashboard && npm run build)

# 1. Build External PDK Plugins first so workspace tests can dynamically load them
echo "🔌 Building External PDK Plugins (hs-git-ast, hs-eco)..."
cargo build --release -p hs-git-ast -p hs-eco
for ext in dylib so dll; do
    for lib in libhs_git_ast libhs_eco hs_git_ast hs_eco; do
        if [ -f "target/release/${lib}.${ext}" ]; then
            cp "target/release/${lib}.${ext}" plugins/
        fi
    done
done

# 2. Run Quality Checks (Sync with CI)
echo "🧪 Running Quality Checks..."
cargo fmt --all -- --check || { echo "❌ Formatting errors found! Run 'cargo fmt --all' to fix."; exit 1; }
cargo clippy --all-targets --all-features -- -D warnings || { echo "❌ Clippy warnings found!"; exit 1; }
cargo clippy --tests --workspace -- -W clippy::pedantic || { echo "❌ Clippy pedantic warnings found!"; exit 1; }
cargo test --workspace --release || { echo "❌ Tests failed!"; exit 1; }

# 3. Build Release Binaries & External Plugins
echo "🔨 Building Release Binaries & PDK Extensions..."
cargo build --release -p hyperspace-server -p hyperspace-cli -p hs-git-ast -p hs-eco

# Use target directory for staging (cleaner)
STAGING_DIR="target/release_pkg"
rm -rf "$STAGING_DIR"
mkdir -p "$STAGING_DIR/plugins"

cp target/release/hyperspace-server "$STAGING_DIR/"
cp target/release/hyperspace-cli "$STAGING_DIR/"
cp plugins/plugins.toml "$STAGING_DIR/plugins/"

for ext in dylib so dll; do
    for lib in libhs_git_ast libhs_eco hs_git_ast hs_eco; do
        if [ -f "target/release/${lib}.${ext}" ]; then
            cp "target/release/${lib}.${ext}" "$STAGING_DIR/plugins/"
            cp "target/release/${lib}.${ext}" plugins/
        fi
    done
done

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
        -f Dockerfile \
        -t glukhota/hyperspace-db:latest \
        -t glukhota/hyperspace-db:$VERSION \
        -t ghcr.io/yarlabs/hyperspace-db:latest \
        -t ghcr.io/yarlabs/hyperspace-db:$VERSION \
        --push .

    echo "⚡ Building & Pushing SaaS Image (AMD64 optimized, s3-tiering, PDK & external plugins)..."
    docker buildx build --platform linux/amd64 \
        -f Dockerfile.saas \
        -t glukhota/hyperspace-db:latest-saas \
        -t glukhota/hyperspace-db:${VERSION}-saas \
        -t ghcr.io/yarlabs/hyperspace-db:latest-saas \
        -t ghcr.io/yarlabs/hyperspace-db:${VERSION}-saas \
        --push .

    echo "🧠 Building & Pushing All-in-One Image with Unified hs-embed ONNX Pipeline (amd64 & arm64)..."
    docker buildx build --platform linux/amd64,linux/arm64 \
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
