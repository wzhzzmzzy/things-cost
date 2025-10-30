#!/bin/bash

# Build release script for things-cost
# This script builds the project for multiple platforms

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print colored output
print_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check if we're in the project root
if [ ! -f "Cargo.toml" ]; then
    print_error "Please run this script from the project root directory"
    exit 1
fi

# Create output directory
OUTPUT_DIR="dist"
mkdir -p "$OUTPUT_DIR"

# Get version from Cargo.toml
VERSION=$(grep '^version =' Cargo.toml | head -1 | cut -d '"' -f2)
print_info "Building version: $VERSION"

# Build targets
TARGETS=(
    "x86_64-unknown-linux-gnu"
    "x86_64-apple-darwin"
    "x86_64-pc-windows-msvc"
    "aarch64-unknown-linux-gnu"
    "aarch64-apple-darwin"
)

# Install cross-compilation tools if needed
if command_exists "apt-get"; then
    print_info "Installing cross-compilation tools..."
    sudo apt-get update
    sudo apt-get install -y gcc-aarch64-linux-gnu
fi

# Build for each target
for target in "${TARGETS[@]}"; do
    print_info "Building for target: $target"

    # Install target if not installed
    if ! rustup target list | grep -q "$target (installed)"; then
        print_info "Installing target: $target"
        rustup target add "$target"
    fi

    # Build the project
    if cargo build --release --target "$target"; then
        print_info "Build successful for $target"

        # Determine binary name and extension
        if [[ "$target" == *"windows"* ]]; then
            BINARY_NAME="things-cost.exe"
            ARCHIVE_EXT="zip"
        else
            BINARY_NAME="things-cost"
            ARCHIVE_EXT="tar.gz"
        fi

        # Strip binary (if available)
        BINARY_PATH="target/$target/release/$BINARY_NAME"
        if command_exists "strip" && [[ "$target" != *"windows"* ]]; then
            print_info "Stripping binary..."
            strip "$BINARY_PATH"
        fi

        # Create archive
        ARCHIVE_NAME="things-cost-v${VERSION}-${target}.${ARCHIVE_EXT}"

        if [[ "$ARCHIVE_EXT" == "zip" ]]; then
            cd "target/$target/release" && zip "../../../$OUTPUT_DIR/$ARCHIVE_NAME" "$BINARY_NAME" && cd ../../..
        else
            tar czf "$OUTPUT_DIR/$ARCHIVE_NAME" -C "target/$target/release" "$BINARY_NAME"
        fi

        print_info "Created archive: $OUTPUT_DIR/$ARCHIVE_NAME"

        # Generate SHA256 checksum
        cd "$OUTPUT_DIR"
        shasum -a 256 "$ARCHIVE_NAME" > "$ARCHIVE_NAME.sha256"
        cd ..

        print_info "Created checksum: $OUTPUT_DIR/$ARCHIVE_NAME.sha256"

    else
        print_error "Build failed for $target"
        exit 1
    fi
done

print_info "All builds completed successfully!"
print_info "Output files are in: $OUTPUT_DIR/"

# List created files
print_info "Created files:"
ls -la "$OUTPUT_DIR/"