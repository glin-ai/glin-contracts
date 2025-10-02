#!/bin/bash

# GLIN Contracts Deployment Script
# Deploys contracts to GLIN testnet

set -e

echo "🚀 Deploying GLIN Smart Contracts to Testnet"
echo "============================================="

# Check if cargo-contract is installed
if ! command -v cargo-contract &> /dev/null; then
    echo "❌ cargo-contract not found!"
    echo "📦 Install it with: cargo install cargo-contract --force"
    exit 1
fi

# Configuration
TESTNET_URL="${TESTNET_URL:-ws://localhost:9944}"
SURI="${SURI:-//Alice}"  # Default to Alice for local testing

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo ""
echo -e "${YELLOW}⚠️  Configuration:${NC}"
echo "  • Network: $TESTNET_URL"
echo "  • Deployer: Using provided seed/key"
echo ""
echo -e "${YELLOW}📝 Note: Make sure contracts are built first (run build-all.sh)${NC}"
echo ""

# Wait for user confirmation
read -p "Continue with deployment? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "Deployment cancelled."
    exit 1
fi

BUILD_DIR="../target/ink"

# Deploy function
deploy_contract() {
    local contract_path=$1
    local contract_name=$2
    local constructor_args=$3

    echo ""
    echo -e "${BLUE}📤 Deploying $contract_name...${NC}"
    echo "------------------------"

    cargo contract instantiate \
        --url "$TESTNET_URL" \
        --suri "$SURI" \
        --constructor "new" \
        $constructor_args \
        --execute \
        "$contract_path"

    if [ $? -eq 0 ]; then
        echo -e "${GREEN}✅ $contract_name deployed successfully${NC}"
    else
        echo "❌ Failed to deploy $contract_name"
        exit 1
    fi
}

# Deploy contracts

echo ""
echo "========================"
echo "Deploying Contracts..."
echo "========================"

# 1. Deploy ProfessionalRegistry
echo ""
echo -e "${BLUE}1️⃣  ProfessionalRegistry${NC}"
deploy_contract \
    "$BUILD_DIR/registry.contract" \
    "ProfessionalRegistry" \
    "--args $SURI $SURI 1000"  # owner, treasury, slash_bps

# Save deployed address (user needs to input)
read -p "Enter ProfessionalRegistry contract address: " REGISTRY_ADDRESS
echo "Saved: $REGISTRY_ADDRESS"

# 2. Deploy ArbitrationDAO
echo ""
echo -e "${BLUE}2️⃣  ArbitrationDAO${NC}"
deploy_contract \
    "$BUILD_DIR/arbitration.contract" \
    "ArbitrationDAO" \
    "--args $SURI 100000000000000000000 604800000 5000"  # owner, min_stake (100 GLIN), voting_period (7 days), quorum 50%

# Save deployed address
read -p "Enter ArbitrationDAO contract address: " ARBITRATION_ADDRESS
echo "Saved: $ARBITRATION_ADDRESS"

# 3. Deploy GenericEscrow
echo ""
echo -e "${BLUE}3️⃣  GenericEscrow${NC}"
deploy_contract \
    "$BUILD_DIR/escrow.contract" \
    "GenericEscrow" \
    "--args $SURI 200"  # platform_account, platform_fee_bps (2%)

# Save deployed address
read -p "Enter GenericEscrow contract address: " ESCROW_ADDRESS
echo "Saved: $ESCROW_ADDRESS"

# Create deployment manifest
MANIFEST_FILE="deployment-manifest.json"

cat > "$MANIFEST_FILE" << EOF
{
  "network": "$TESTNET_URL",
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "deployer": "SURI hidden for security",
  "contracts": {
    "ProfessionalRegistry": {
      "address": "$REGISTRY_ADDRESS",
      "description": "Professional staking and reputation system"
    },
    "ArbitrationDAO": {
      "address": "$ARBITRATION_ADDRESS",
      "description": "Decentralized arbitration with weighted voting"
    },
    "GenericEscrow": {
      "address": "$ESCROW_ADDRESS",
      "description": "Milestone-based escrow with AI oracle integration"
    }
  }
}
EOF

echo ""
echo "============================================="
echo -e "${GREEN}🎉 All contracts deployed successfully!${NC}"
echo ""
echo "📋 Deployment manifest saved to: $MANIFEST_FILE"
echo ""
echo "📍 Deployed Addresses:"
echo "  • ProfessionalRegistry: $REGISTRY_ADDRESS"
echo "  • ArbitrationDAO:       $ARBITRATION_ADDRESS"
echo "  • GenericEscrow:        $ESCROW_ADDRESS"
echo ""
echo "🔗 Connect via Polkadot.js Apps:"
echo "   https://polkadot.js.org/apps/?rpc=$TESTNET_URL#/contracts"
echo ""
echo "📚 Next Steps:"
echo "  1. Verify contracts in Polkadot.js Apps"
echo "  2. Test basic functions"
echo "  3. Update frontend with contract addresses"
echo "  4. Document API endpoints"
echo ""
