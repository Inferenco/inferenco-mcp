#!/bin/bash

# Inferenco-MCP Build Script

set -e

echo "Building Inferenco-MCP Server..."
echo "=========================="

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "Error: Rust/Cargo not found. Please install Rust first."
    echo "Visit: https://rustup.rs/"
    exit 1
fi

# Build the project
echo "Building release binary..."
cargo build --release --bin inferenco-mcp-stdio

# Check if build was successful
if [ -f "target/release/inferenco-mcp-stdio" ]; then
    echo "✅ Build successful!"
    echo "Binary location: target/release/inferenco-mcp-stdio"
    
    # Show binary info
    echo ""
    echo "Binary information:"
    ls -lh target/release/inferenco-mcp-stdio
    
    echo ""
    echo "To run the server:"
    echo "  ./target/release/inferenco-mcp-stdio"
    echo ""
    echo "Or use cargo:"
    echo "  cargo run --bin inferenco-mcp-stdio"
else
    echo "❌ Build failed!"
    exit 1
fi
