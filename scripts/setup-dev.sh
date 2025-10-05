#!/bin/bash
# Development setup script

set -e

echo "🥒 Setting up Raugupatis Log development environment..."

# Create data directory
mkdir -p data

# Install Rust if not already installed
if ! command -v cargo &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source ~/.cargo/env
fi

# Install development tools
echo "Installing development tools..."
cargo install cargo-watch
cargo install cargo-nextest

# Build the project
echo "Building the project..."
cargo build

echo "✅ Development environment setup complete!"
echo ""
echo "To start the development server:"
echo "  cargo run"
echo ""
echo "To start with hot-reload:"
echo "  cargo watch -x run"
echo ""
echo "To run tests:"
echo "  cargo test"
echo ""
echo "To build and run with Docker:"
echo "  docker-compose up --build"