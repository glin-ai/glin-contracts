#!/bin/bash

# GLIN Contracts Build Script
# Builds all smart contracts in the workspace

set -e

echo "🔨 Building GLIN Smart Contracts..."
echo "=================================="

# Check if cargo-contract is installed
if ! command -v cargo-contract &> /dev/null; then
    echo "❌ cargo-contract not found!"
    echo "📦 Install it with: cargo install cargo-contract --force"
    exit 1
fi

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Build directory
BUILD_DIR="../target/ink"
mkdir -p "$BUILD_DIR"

# Contracts to build
contracts=("escrow" "registry" "arbitration")
contract_names=("GenericEscrow" "ProfessionalRegistry" "ArbitrationDAO")

# Build each contract
for i in "${!contracts[@]}"; do
    contract="${contracts[$i]}"
    name="${contract_names[$i]}"

    echo ""
    echo -e "${BLUE}📋 Building $name...${NC}"
    echo "------------------------"

    cd "../$contract"
    cargo contract build --release

    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ $name built successfully${NC}"

        # Copy artifacts to build directory
        cp "target/ink/${contract}.contract" "$BUILD_DIR/${contract}.contract"
        cp "target/ink/${contract}.wasm" "$BUILD_DIR/${contract}.wasm"

        # Display file info
        echo "📦 Contract size: $(du -h target/ink/${contract}.wasm | cut -f1)"
    else
        echo "❌ Failed to build $name"
        exit 1
    fi

    cd ../scripts
done

echo ""
echo "=================================="
echo -e "${GREEN}🎉 All contracts built successfully!${NC}"
echo ""
echo "📂 Artifacts location: $BUILD_DIR"
echo ""
echo "Built contracts:"
for contract in "${contracts[@]}"; do
    echo "  • ${contract}.contract"
    echo "  • ${contract}.wasm"
done
echo ""
echo "🚀 Ready for deployment!"
