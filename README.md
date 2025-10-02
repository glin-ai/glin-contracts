# GLIN Smart Contracts

Reference smart contract implementations for the GLIN AI-powered dApp platform. These contracts enable developers to build decentralized applications across legal, healthcare, real estate, and other industries.

## 📦 Contracts

### 1. GenericEscrow
**Purpose:** Milestone-based payment escrow with AI oracle integration

**Features:**
- ✅ Multi-party agreements (client & provider)
- ✅ Milestone-based payment releases
- ✅ Dispute resolution mechanism
- ✅ AI oracle verification support
- ✅ Platform fee collection (configurable)
- ✅ Deadline tracking per milestone

**Use Cases:**
- Freelance contracts
- Service agreements
- Real estate transactions
- Legal retainers
- Construction projects

**Key Functions:**
```rust
create_agreement()      // Create escrow with milestones
complete_milestone()    // Provider marks work done
approve_and_release()   // Client/oracle approves payment
raise_dispute()         // Either party disputes milestone
resolve_dispute()       // Oracle resolves dispute
```

---

### 2. ProfessionalRegistry
**Purpose:** Staking-based professional registry with reputation system

**Features:**
- ✅ Role-based registration (Lawyer, Doctor, Arbitrator, etc.)
- ✅ Stake requirements per role
- ✅ Reputation scoring (0-100+)
- ✅ Review/rating system
- ✅ Slashing for misbehavior
- ✅ Profile metadata (IPFS URI)

**Use Cases:**
- Professional verification
- Service provider discovery
- Reputation-based matching
- Quality assurance
- Credential validation

**Key Functions:**
```rust
register()              // Register as professional with stake
increase_stake()        // Add more stake to profile
submit_review()         // Rate a professional (1-5 stars)
slash()                 // Penalize misbehavior (owner only)
withdraw_stake()        // Exit registry
```

---

### 3. ArbitrationDAO
**Purpose:** Decentralized arbitration with weighted voting

**Features:**
- ✅ Stake-weighted voting
- ✅ Configurable voting periods
- ✅ Quorum requirements
- ✅ Appeal mechanism (one appeal per dispute)
- ✅ Evidence submission (IPFS)
- ✅ Arbitrator reputation tracking

**Use Cases:**
- Contract dispute resolution
- Community governance
- Decentralized justice
- Peer-to-peer conflict resolution
- DAO decision making

**Key Functions:**
```rust
register_arbitrator()   // Stake to become arbitrator
create_dispute()        // Open new dispute
start_voting()          // Begin arbitration process
vote()                  // Cast weighted vote
finalize_dispute()      // Execute final decision
appeal_dispute()        // Request re-vote (one-time)
```

---

## 🚀 Quick Start

### Prerequisites

1. **Install Rust & Cargo**
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. **Install cargo-contract**
```bash
cargo install cargo-contract --force
```

3. **Add wasm32 target**
```bash
rustup target add wasm32-unknown-unknown
```

### Build All Contracts

```bash
cd glin-contracts/scripts
./build-all.sh
```

This will:
- Build all 3 contracts in release mode
- Generate `.contract` and `.wasm` files
- Copy artifacts to `target/ink/`

### Build Individual Contract

```bash
cd glin-contracts/escrow
cargo contract build --release
```

### Run Tests

```bash
# Test all contracts
cargo test --workspace

# Test specific contract
cd escrow && cargo test
```

---

## 📤 Deployment

### Local Development Node

1. **Start GLIN node**
```bash
cd ../glin-chain
./target/release/glin-node --dev
```

2. **Deploy contracts**
```bash
cd ../glin-contracts/scripts
./deploy-testnet.sh
```

### Testnet Deployment

```bash
export TESTNET_URL="wss://testnet.glin.network"
export SURI="//YourSeedPhrase"  # NEVER commit this!
./deploy-testnet.sh
```

---

## 💡 Usage Examples

### Example 1: Create Escrow Agreement

```rust
// Client creates escrow for 2 milestones
escrow.create_agreement(
    provider_address,
    vec!["Design mockups", "Final implementation"],
    vec![500 * GLIN, 1500 * GLIN],
    vec![timestamp_week_1, timestamp_week_4],
    dispute_timeout,
    Some(oracle_address)
)
```

### Example 2: Register as Professional

```rust
// Lawyer registers with 100 GLIN stake
registry.register(
    ProfessionalRole::Lawyer,
    "ipfs://QmYourMetadata" // JSON with credentials
)
```

### Example 3: Create Dispute

```rust
// Claimant creates dispute
arbitration.create_dispute(
    defendant_address,
    "Contract breach - undelivered services",
    "ipfs://QmEvidenceBundle"
)
```

---

## 🏗️ Architecture

### Contract Interactions

```
┌─────────────────┐
│  GenericEscrow  │
└────────┬────────┘
         │ calls
         ▼
┌─────────────────┐      ┌──────────────────┐
│ ArbitrationDAO  │◄────►│ ProfessionalReg  │
└─────────────────┘      └──────────────────┘
         ▲                        ▲
         │                        │
         └────────────────────────┘
         Check arbitrator status
```

### Integration with GLIN Backend

```
AI Backend (Rust/Axum)
  │
  ├─► Listens to contract events
  ├─► Submits AI analysis as oracle
  ├─► Caches data for fast queries
  └─► Provides REST API for dApps
```

---

## 📝 Development Guide

### Adding a New Contract

1. **Create directory**
```bash
mkdir glin-contracts/my_contract
```

2. **Add to workspace** (`Cargo.toml`)
```toml
members = [
    "escrow",
    "registry",
    "arbitration",
    "my_contract",  # Add here
]
```

3. **Create contract** (`my_contract/lib.rs`)
```rust
#![cfg_attr(not(feature = "std"), no_std, no_main)]

#[ink::contract]
mod my_contract {
    #[ink(storage)]
    pub struct MyContract {
        // Your storage
    }

    impl MyContract {
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {}
        }

        #[ink(message)]
        pub fn my_function(&self) {}
    }
}
```

4. **Update build script**
Add contract to `scripts/build-all.sh`

---

## 🧪 Testing

### Unit Tests

Each contract includes built-in tests:

```bash
cd escrow
cargo test

# With verbose output
cargo test -- --nocapture
```

### Integration Testing

```bash
# Start local node
./glin-chain/target/release/glin-node --dev

# Run integration tests (coming soon)
cd glin-contracts/integration-tests
cargo test
```

---

## 🔒 Security Considerations

### Storage Deposits
All contracts require deposits for storage:
- **Per item**: 1 GLIN
- **Per byte**: 0.001 GLIN

### Gas Limits
- Max code size: 256 KB
- Max storage key: 128 bytes
- Transient storage: 1 MB

### Best Practices
1. ✅ Always validate inputs
2. ✅ Use Checks-Effects-Interactions pattern
3. ✅ Emit events for important state changes
4. ✅ Implement access control for sensitive functions
5. ✅ Handle overflows/underflows
6. ✅ Test edge cases thoroughly

---

## 📊 Gas Cost Estimates

| Operation | Estimated Cost (GLIN) |
|-----------|---------------------|
| Deploy Escrow | ~0.5 |
| Create Agreement | ~0.1 |
| Complete Milestone | ~0.05 |
| Register Professional | ~0.2 |
| Submit Review | ~0.05 |
| Create Dispute | ~0.1 |
| Cast Vote | ~0.05 |

*Actual costs may vary based on network congestion*

---

## 🌐 Frontend Integration

### Using Polkadot.js

```javascript
import { ContractPromise } from '@polkadot/api-contract';

// Connect to contract
const contract = new ContractPromise(
  api,
  escrowMetadata,
  'CONTRACT_ADDRESS'
);

// Call contract method
await contract.tx
  .completeM ilestone({ value: 0, gasLimit: -1 }, agreementId, 0)
  .signAndSend(account);

// Query contract state
const { output } = await contract.query
  .getAgreement(account.address, { gasLimit: -1 }, agreementId);
```

### Using GLIN SDK (Coming Soon)

```javascript
import { GlinContracts } from '@glin/sdk';

const contracts = new GlinContracts(api);
await contracts.escrow.createAgreement(...);
```

---

## 🔗 Resources

- **ink! Documentation**: https://use.ink
- **Substrate Docs**: https://docs.substrate.io
- **GLIN Blockchain**: ../glin-chain
- **GLIN Backend**: ../glin-backend
- **Example dApps**: ../examples

---

## 📄 License

Apache 2.0 - see LICENSE file

---

## 🤝 Contributing

1. Fork the repository
2. Create feature branch (`git checkout -b feature/my-contract`)
3. Write tests
4. Ensure `cargo test` passes
5. Submit pull request

---

## 📞 Support

- GitHub Issues: https://github.com/glin/glin-chain/issues
- Discord: [Coming Soon]
- Email: dev@glin.network

---

## 🗺️ Roadmap

**Phase 1 (Current):** ✅ Core contracts
- GenericEscrow
- ProfessionalRegistry
- ArbitrationDAO

**Phase 2 (Q2 2025):** DeFi & NFT Contracts
- Lending pools
- NFT marketplace
- Staking derivatives

**Phase 3 (Q3 2025):** Cross-Chain
- XCM integration
- Bridge contracts
- Multi-chain deployment

**Phase 4 (Q4 2025):** Advanced Features
- Privacy contracts (zk-SNARKs)
- Automated market makers
- Prediction markets

---

Built with ❤️ by the GLIN Team
