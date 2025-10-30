#!/bin/bash

# Test script to simulate release workflow locally

set -e

echo "🚀 Testing release workflow locally..."

# Create a test tag for simulation
echo "Creating test tag..."
git tag -f test-release-v0.1.0 2>/dev/null || true

# Test building for different targets
echo ""
echo "🔨 Testing builds for different targets..."

TARGETS=(
    "x86_64-unknown-linux-gnu"
    "x86_64-apple-darwin"
    "x86_64-pc-windows-msvc"
)

for target in "${TARGETS[@]}"; do
    echo "Building for $target..."

    # Install target if needed
    if ! rustup target list | grep -q "$target (installed)"; then
        echo "  Installing target: $target"
        rustup target add "$target"
    fi

    # Build
    if cargo build --release --target "$target"; then
        echo "  ✅ Build successful"

        # Check if binary was created
        if [[ "$target" == *"windows"* ]]; then
            BINARY="target/$target/release/things-cost.exe"
        else
            BINARY="target/$target/release/things-cost"
        fi

        if [ -f "$BINARY" ]; then
            echo "  ✅ Binary created: $BINARY"
            ls -la "$BINARY"
        else
            echo "  ❌ Binary not found: $BINARY"
        fi
    else
        echo "  ❌ Build failed"
    fi
    echo ""
done

# Test archive creation
echo "📦 Testing archive creation..."

# Linux
if [ -f "target/x86_64-unknown-linux-gnu/release/things-cost" ]; then
    tar czf test-linux.tar.gz -C target/x86_64-unknown-linux-gnu/release things-cost
    echo "✅ Created Linux archive: test-linux.tar.gz"
    ls -la test-linux.tar.gz
fi

# Windows (simulate)
if [ -f "target/x86_64-pc-windows-msvc/release/things-cost.exe" ]; then
    echo "✅ Windows binary exists (would create ZIP in real workflow)"
fi

# Clean up
echo ""
echo "🧹 Cleaning up..."
rm -f test-linux.tar.gz

# Remove test tag
git tag -d test-release-v0.1.0 2>/dev/null || true

echo ""
echo "🎉 Release workflow test completed!"
echo ""
echo "📋 Next steps for actual release:"
echo "   1. Update version in Cargo.toml"
echo "   2. Commit changes: git add . && git commit -m 'Release vX.Y.Z'"
echo "   3. Create tag: git tag -a vX.Y.Z -m 'Release vX.Y.Z'"
echo "   4. Push tag: git push origin vX.Y.Z"
echo "   5. GitHub Actions will automatically create the release"