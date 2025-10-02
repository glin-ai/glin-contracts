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

# Path to validator keys (gitignored directory with secrets)
VALIDATOR_KEYS_DIR="${VALIDATOR_KEYS_DIR:-../../glin-chain/validator-keys}"
FAUCET_KEY_FILE="$VALIDATOR_KEYS_DIR/faucet_account.json"

# For local --dev mode (Alice has balance in dev mode)
ALICE_SURI="//Alice"
ALICE_ACCOUNT="5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY"

# Load faucet account from JSON file (never hardcode seeds!)
load_faucet_account() {
    if [[ ! -f "$FAUCET_KEY_FILE" ]]; then
        echo "❌ Faucet key file not found: $FAUCET_KEY_FILE"
        echo "This file should be in the gitignored validator-keys directory."
        exit 1
    fi

    # Extract secretPhrase and ss58Address from JSON
    FAUCET_SURI=$(grep -oP '"secretPhrase":\s*"\K[^"]+' "$FAUCET_KEY_FILE")
    FAUCET_ACCOUNT=$(grep -oP '"ss58Address":\s*"\K[^"]+' "$FAUCET_KEY_FILE")

    if [[ -z "$FAUCET_SURI" ]] || [[ -z "$FAUCET_ACCOUNT" ]]; then
        echo "❌ Failed to parse faucet account from $FAUCET_KEY_FILE"
        exit 1
    fi
}

# Determine which account to use
if [[ "$TESTNET_URL" == *"localhost"* ]] || [[ "$TESTNET_URL" == *"127.0.0.1"* ]]; then
    # Check if user wants to use testnet chain spec on localhost
    if [[ -z "$USE_FAUCET" ]]; then
        echo "Local node detected. Using Alice for dev mode."
        echo "If running testnet chain spec locally, set USE_FAUCET=1"
        SURI="$ALICE_SURI"
        DEPLOY_ACCOUNT="$ALICE_ACCOUNT"
    else
        load_faucet_account
        SURI="$FAUCET_SURI"
        DEPLOY_ACCOUNT="$FAUCET_ACCOUNT"
    fi
else
    # Remote testnet - use faucet account
    load_faucet_account
    SURI="${SURI:-$FAUCET_SURI}"
    DEPLOY_ACCOUNT="$FAUCET_ACCOUNT"
fi

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo ""
echo -e "${YELLOW}⚠️  Configuration:${NC}"
echo "  • Network: $TESTNET_URL"
echo "  • Deployer: $DEPLOY_ACCOUNT"
echo "  • Using SURI: ${SURI:0:20}..." # Show first 20 chars only
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

# Get absolute path to build directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BUILD_DIR="$(cd "$SCRIPT_DIR/.." && pwd)/target/ink"

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
    "$BUILD_DIR/professional_registry.contract" \
    "ProfessionalRegistry" \
    "--args $DEPLOY_ACCOUNT $DEPLOY_ACCOUNT 1000"  # owner, treasury, slash_bps

# Save deployed address (user needs to input)
read -p "Enter ProfessionalRegistry contract address: " REGISTRY_ADDRESS
echo "Saved: $REGISTRY_ADDRESS"

# 2. Deploy ArbitrationDAO
echo ""
echo -e "${BLUE}2️⃣  ArbitrationDAO${NC}"
deploy_contract \
    "$BUILD_DIR/arbitration_dao.contract" \
    "ArbitrationDAO" \
    "--args $DEPLOY_ACCOUNT 100000000000000000000 604800000 5000"  # owner, min_stake (100 GLIN), voting_period (7 days), quorum 50%

# Save deployed address
read -p "Enter ArbitrationDAO contract address: " ARBITRATION_ADDRESS
echo "Saved: $ARBITRATION_ADDRESS"

# 3. Deploy GenericEscrow
echo ""
echo -e "${BLUE}3️⃣  GenericEscrow${NC}"
deploy_contract \
    "$BUILD_DIR/generic_escrow.contract" \
    "GenericEscrow" \
    "--args $DEPLOY_ACCOUNT 200"  # platform_account, platform_fee_bps (2%)

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
