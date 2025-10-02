# GLIN Smart Contracts Deployment Guide

## Prerequisites

Before deploying the smart contracts to the GLIN testnet, ensure the following:

### 1. Node Requirements
✅ The GLIN blockchain node **must be rebuilt** with the contracts pallet RPC support we added:
- Modified `/glin-chain/node/Cargo.toml` to include `pallet-contracts-rpc`
- Modified `/glin-chain/node/src/rpc.rs` to expose Contracts RPC endpoints

**Action Required:**
```bash
cd /home/eralp/Projects/glin/glin-chain
cargo build --release
```

Then redeploy all Railway validator and RPC services with the new binary.

### 2. Contracts Compilation
✅ All contracts have been compiled successfully:
- GenericEscrow: 16K (optimized)
- ProfessionalRegistry: 16K (optimized)
- ArbitrationDAO: 16K (optimized)

Located in: `/home/eralp/Projects/glin/glin-contracts/target/ink/`

### 3. Deployment Account
You'll need an account with sufficient balance (tGLIN) for:
- Gas fees for deployment
- Initial storage deposit for each contract

## Deployment Options

### Option A: Local Development Node

For testing, deploy to a local node first:

```bash
# 1. Start local GLIN node
cd /home/eralp/Projects/glin/glin-chain
./target/release/glin-node --dev

# 2. Deploy contracts
cd /home/eralp/Projects/glin/glin-contracts/scripts
./deploy-testnet.sh
```

The script will prompt for:
- Network confirmation (defaults to ws://localhost:9944)
- Deployer account (defaults to //Alice for dev)
- Contract addresses after each deployment

### Option B: GLIN Testnet

Deploy to the production testnet:

```bash
cd /home/eralp/Projects/glin/glin-contracts/scripts

# The script automatically reads from validator-keys/faucet_account.json (gitignored)
export TESTNET_URL="wss://glin-rpc-production.up.railway.app"
./deploy-testnet.sh
```

**⚠️ Security Architecture:**
- ✅ The script reads seed phrases from `glin-chain/validator-keys/` (gitignored directory)
- ✅ **NEVER** hardcodes seed phrases in script files
- ✅ Seed phrases are loaded at runtime from JSON files
- ✅ The validator-keys directory is in `.gitignore` and never committed
- ⚠️ For custom accounts, use: `export SURI="your seed phrase"` (environment variable only)
- ⚠️ For production mainnet, use hardware wallets or HSM

## Deployment Process

The deployment script will deploy contracts in this order:

### 1. ProfessionalRegistry
Constructor parameters:
- `owner`: Account that can slash professionals and update parameters
- `slash_treasury`: Where slashed funds are sent
- `slash_percentage_bps`: Slashing percentage (1000 = 10%)

**Recommended for testnet:** `owner=deployer`, `treasury=deployer`, `slash_bps=1000`

### 2. ArbitrationDAO
Constructor parameters:
- `owner`: Account that can update DAO parameters
- `min_arbitrator_stake`: 100 GLIN (100000000000000000000)
- `voting_period`: 7 days in milliseconds (604800000)
- `quorum_bps`: 50% quorum (5000)

### 3. GenericEscrow
Constructor parameters:
- `platform_account`: Receives platform fees
- `platform_fee_bps`: 2% fee (200)

## Post-Deployment

After successful deployment:

### 1. Save Contract Addresses
The script creates `deployment-manifest.json` with:
```json
{
  "network": "wss://...",
  "timestamp": "2025-10-02T...",
  "contracts": {
    "ProfessionalRegistry": { "address": "5..." },
    "ArbitrationDAO": { "address": "5..." },
    "GenericEscrow": { "address": "5..." }
  }
}
```

### 2. Update Documentation
Update `/docs/developers/smart-contracts.md` with actual contract addresses.

### 3. Verify in Polkadot.js Apps
1. Go to https://polkadot.js.org/apps/?rpc=wss://glin-rpc-production.up.railway.app#/contracts
2. Add each contract using its address and metadata JSON
3. Test basic read functions (getters)
4. Test write functions with testnet tokens

### 4. Integration Testing

Test each contract's core functionality:

**ProfessionalRegistry:**
```bash
# Register as a professional
cargo contract call \
  --contract 5... \
  --message register \
  --args "Lawyer" "ipfs://metadata" \
  --value 100000000000000000000 \
  --suri "//Alice"
```

**ArbitrationDAO:**
```bash
# Register as arbitrator
cargo contract call \
  --contract 5... \
  --message register_arbitrator \
  --value 100000000000000000000 \
  --suri "//Bob"
```

**GenericEscrow:**
```bash
# Create agreement
cargo contract call \
  --contract 5... \
  --message create_agreement \
  --args "5Bob..." '["Milestone 1"]' '[1000000000000000000]' '[1704067200000]' 1704672000000 null \
  --value 1000000000000000000 \
  --suri "//Alice"
```

### 5. Update Frontend/Backend

Update contract addresses in:
- Backend environment variables
- Frontend configuration
- SDK examples
- Developer documentation

## Troubleshooting

### Error: Connection Refused
**Problem:** Can't connect to RPC endpoint
**Solution:**
- Verify node is running and RPC is enabled with `--rpc-external`
- Check firewall rules allow WebSocket connections
- Confirm URL is correct (ws:// for local, wss:// for remote)

### Error: Module Not Found: Contracts
**Problem:** Node doesn't have contracts pallet
**Solution:** Rebuild node with contracts pallet support (see Prerequisites)

### Error: Insufficient Balance
**Problem:** Deployment account doesn't have enough tokens
**Solution:**
- For local dev: Use //Alice which has initial balance
- For testnet: Get tGLIN from faucet or transfer from another account

### Error: Storage Deposit Limit Reached
**Problem:** Contract needs more storage deposit
**Solution:** Increase storage deposit in deployment command:
```bash
cargo contract instantiate ... --storage-deposit-limit 1000000000000
```

## Network Endpoints

### Mainnet (Future)
- **RPC:** wss://mainnet.glin.network
- **Explorer:** https://explorer.glin.network
- **Polkadot.js:** https://polkadot.js.org/apps/?rpc=wss://mainnet.glin.network

### Testnet (Current)
- **RPC:** wss://glin-rpc-production.up.railway.app
- **Explorer:** https://glin-explorer.vercel.app
- **Polkadot.js:** https://polkadot.js.org/apps/?rpc=wss://glin-rpc-production.up.railway.app

### Local Development
- **RPC:** ws://localhost:9944
- **Polkadot.js:** https://polkadot.js.org/apps/?rpc=ws://localhost:9944

## Next Steps

1. ✅ Compile contracts (COMPLETED)
2. ⏳ Rebuild blockchain node with contracts RPC
3. ⏳ Redeploy validators and RPC node to Railway
4. ⏳ Deploy contracts to testnet
5. ⏳ Update documentation with contract addresses
6. ⏳ Integrate contracts with backend API
7. ⏳ Add contract interaction examples to SDK

## Reference

- **ink! Documentation:** https://use.ink
- **cargo-contract:** https://github.com/paritytech/cargo-contract
- **Substrate Contracts:** https://docs.substrate.io/tutorials/smart-contracts/
