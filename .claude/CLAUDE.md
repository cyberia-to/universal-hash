# UniversalHash Project - Claude Context

## Project Overview

**UniversalHash** (U-Hash) is a democratic proof-of-work mining system designed for the Bostrom/Cyber blockchain ecosystem. The goal is to enable fair token distribution where smartphones and consumer devices can meaningfully compete with servers and GPUs.

### Core Vision
- **Democratic Mining**: Phone-to-desktop ratio of 1:3-5 (vs 1:50-100+ in traditional PoW)
- **ASIC Resistance**: Multi-primitive design makes specialized hardware economically infeasible
- **Permissionless Entry**: New miners can participate with zero tokens (gas deducted from rewards)
- **Pool-less Design**: Proportional epoch rewards eliminate need for mining pools

---

## Architecture Components

### 1. Verifier Contract (CosmWasm/Rust)
**Location**: `cw-cyber` repository (https://github.com/cyberia-to/cw-cyber/tree/main/contracts/cw-universal-hash)
**PR**: https://github.com/cyberia-to/cw-cyber/pull/46

**Purpose**: On-chain verification of PoW proofs submitted by miners

**Key Functions**:
- `verify_pow_proof()` - Validate submitted hash meets difficulty
- `compute_min_profitable_difficulty()` - Anti-spam threshold based on gas market
- `settle_epoch()` - Distribute rewards proportionally to difficulty-weighted work

**Integration Points**:
- Uses `cyber-std` bindings for Bostrom blockchain interactions
- Token minting via TokenFactory module bindings
- Epoch-based reward distribution (10-minute epochs default)

### 2. Prover (Standalone Rust Binary)
**Location**: `/Users/michaelborisov/Develop/universal-hash`

**Purpose**: CLI tool for mining that compiles to multiple platforms including WASM

**Requirements**:
- Rust binary with minimal CLI interface
- Commands: `mine`, `send`, `import-mnemonic`, `export-mnemonic`
- Builds for: Linux, macOS, Windows, ARM, **WASM (critical for browser)**
- Embedded wallet for transaction signing
- Configurable RPC endpoints with good defaults

**Algorithm**: UniversalHash v4 (spec-compliant)
- 4 parallel chains (matches phone core count)
- 2MB total memory (4x512KB scratchpads)
- 12,288 rounds per chain (per spec)
- Triple primitive rotation: AES_Compress + SHA256_Compress + BLAKE3_Compress
- AES-based scratchpad initialization (per spec)
- Sequential dependencies prevent GPU parallelism

**Dual Delivery Strategy**:
- **Native (Tauri v2)**: Full spec performance (400-1,420 H/s depending on device)
- **WASM (Browser)**: Slower but zero-install (100-400 H/s on phones/desktop)

### 3. Web Interface (cyb-ts)
**Repository**: https://github.com/cyberia-to/cyb-ts (local: `/Users/michaelborisov/Develop/cyb-ts`)

**Purpose**: Mining dashboard in the Cyb application

**Key Features to Implement**:
- Mining page with "start mining" button
- Hashrate display
- Transaction history for submitted proofs
- Rewards earned display
- Network statistics (estimated active miners)
- Integration with WASM-compiled prover

**Tech Stack**:
- React 18 + TypeScript
- @cosmjs/cosmwasm-stargate for contract interactions
- Tauri v2 for desktop builds (dmg, deb, AppImage)
- WASM integration for in-browser mining

---

## Key Technical Decisions

### Simplified Economic Model (No Epoching in Consensus)
Instead of batch epoch settlements requiring consensus changes:
- **Sliding reward calculation**: Pay immediately when valid proof arrives
- **Dynamic difficulty**: Adjusted based on rolling hashrate estimates
- **Gas-deducted rewards**: `net_reward = gross_reward - gas_cost`

### Self-Authenticating Proofs
```
hash = UniversalHash(epoch_seed || miner_address || timestamp || nonce)
```
- No signature required - changing miner_address invalidates hash
- Enables zero-token mining for new participants

### Validator Configuration
Validators need to accept special PoW proof transactions without upfront gas:
- Gas deducted from minted rewards at settlement
- `min_profitable_difficulty` threshold prevents spam

---

## Repository Structure

This is a single Cargo workspace with all crates under `crates/`.

```
universal-hash/
├── Cargo.toml              # Workspace root (no package)
├── crates/
│   ├── cli/                # uhash-prover - mining CLI binary
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs      # Re-exports uhash-core
│   │   │   ├── main.rs     # CLI binary
│   │   │   ├── wallet/     # Mnemonic & wallet management
│   │   │   └── rpc/        # Bostrom RPC client
│   │   └── benches/
│   │       └── uhash_bench.rs
│   ├── core/               # uhash-core - algorithm library (v0.2.3)
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs      # Library entry point (no_std compatible)
│   │   │   ├── params.rs   # Constants (CHAINS=4, ROUNDS=12288, etc.)
│   │   │   ├── primitives.rs # Raw compression functions (AES, SHA256, BLAKE3)
│   │   │   ├── uhash.rs    # UniversalHash v4 implementation
│   │   │   ├── ffi.rs      # C FFI bindings for iOS/Android
│   │   │   └── tests.rs    # Algorithm tests
│   │   └── benches/
│   │       └── uhash_bench.rs
│   ├── web/                # uhash-web - WASM wrapper for browsers
│   │   ├── Cargo.toml
│   │   └── src/lib.rs
│   └── demo/               # uhash-demo - Tauri v2 benchmark app
│       ├── dist/           # Unified frontend (auto-detects Native vs WASM)
│       │   ├── index.html
│       │   └── wasm/       # WASM build output
│       ├── README.md
│       └── src-tauri/      # Tauri v2 native backend
│           ├── Cargo.toml
│           ├── src/lib.rs
│           └── tauri.conf.json
├── Makefile                # Cross-platform build system
├── CHANGELOG.md
├── .cargo/config.toml      # Native CPU features (AES-NI, ARM Crypto)
└── .github/workflows/
    ├── ci.yml              # Lint, test, WASM build
    └── release.yml         # Prover binaries + Tauri desktop + WASM releases
```

**Key Features**:
- `no_std` compatible core (works in WASM/CosmWasm)
- Single source of truth for algorithm
- Unified frontend auto-detects Native vs WASM
- Makefile for all build targets
- C FFI for iOS/Android native apps
- ARM hardware intrinsics (AES, SHA256)

### External Repositories

**cw-cyber (CosmWasm Contracts)** — https://github.com/cyberia-to/cw-cyber
Local: `/Users/michaelborisov/Develop/cw-cyber-merge`

```
contracts/
├── cw-universal-hash/   # Verifier contract (uses uhash-core)
├── cw-cyber-gift/       # Airdrop distribution
├── cw-cyber-passport/   # Identity NFT
└── ...
packages/
├── cyber-std/           # Bostrom bindings
└── cyber-std-test/      # Testing utilities
```

**cyb-ts (Frontend)** — https://github.com/cyberia-to/cyb-ts
Local: `/Users/michaelborisov/Develop/cyb-ts`

---

## Implementation Tasks

### Phase 1: Infrastructure Setup ✅
| # | Task | Status | Notes |
|---|------|--------|-------|
| 1.1 | Merge `cw-cybergift` into `cw-cyber` | ✅ Done | PR #45 |
| 1.2 | Update documentation references | ✅ Done | PR #1372 |

### Phase 2: Prover (Rust CLI) - ✅ Complete
| # | Task | Status | Notes |
|---|------|--------|-------|
| 2.1 | Create Prover repository | ✅ Done | `crates/cli/` |
| 2.2 | Implement UniversalHash v4 algorithm | ✅ Done | `crates/core/src/uhash.rs` |
| 2.3 | `mine` command | ✅ Done | Multi-threaded, auto-submit, fetches seed/difficulty from contract |
| 2.4 | `send` command | ✅ Done | Signs MsgExecuteContract, broadcasts TX |
| 2.5 | `import-mnemonic` command | ✅ Done | Works with 12/24 word phrases |
| 2.6 | `export-mnemonic` command | ✅ Done | |
| 2.7 | RPC endpoint configuration | ✅ Done | Defaults to Bostrom mainnet |
| 2.8 | Multi-arch builds | ✅ Done | GitHub Actions CI/CD for Linux, macOS, Windows |
| 2.9 | WASM build | ✅ Done | `crates/web/` - tested on iPhone & Android |
| 2.10 | Tauri Demo App | ✅ Done | `crates/demo/` - iOS, Android, macOS builds |
| 2.11 | Ubuntu/Linux native build | ✅ Done | Docker ARM64 build + tests pass, `make linux` target |

### Phase 3: Verifier Contract (CosmWasm) - ✅ Complete
| # | Task | Status | Notes |
|---|------|--------|-------|
| 3.1 | Create `cw-universal-hash` contract | ✅ Done | `cw-cyber-merge/contracts/cw-universal-hash` |
| 3.2 | `verify_pow_proof()` | ✅ Done | Uses uhash-core, validates difficulty |
| 3.3 | Self-authenticating proof format | ✅ Done | Address in hash input |
| 3.4 | TokenFactory integration | ✅ Done | CyberMsg::mint_contract_tokens |
| 3.5 | Sliding reward calculation | ✅ Done | Immediate payout on valid proof |
| 3.6 | Dynamic difficulty calculation | ✅ Done | Auto-adjusts on epoch boundaries, clamped 4x |
| 3.7 | `min_profitable_difficulty` threshold | ✅ Done | `ceil(log2(total_work / base_reward))` from rolling window, 30 unit tests |
| 3.8 | Gas cost estimation | ✅ Done | Admin-set `estimated_gas_cost_uboot` (default 250k boot), returned in CalculateReward |
| 3.9 | Deploy to Bostrom mainnet | ✅ Done | Production: Code ID 45, Test (with 3.7/3.8): Code ID 46 |

### Phase 4: Validator Configuration - Priority P1
| # | Task | Status | Notes |
|---|------|--------|-------|
| 4.1 | Document gas-free PoW tx mechanism | ❌ | Validators accept proofs without upfront gas |
| 4.2 | Validator config instructions | ❌ | How to enable PoW proof acceptance |

### Phase 5: Frontend (cyb-ts) - Priority P2
| # | Task | Status | Notes |
|---|------|--------|-------|
| 5.1 | Create mining page | 🔄 Partial | Benchmark page at `crates/demo/dist/` |
| 5.2 | Integrate WASM prover | ✅ Done | wasm-bindgen wrapper working |
| 5.3 | Display hashrate | ✅ Done | Real-time in benchmark page |
| 5.4 | Display transaction history | ❌ | Past proofs submitted |
| 5.5 | Display rewards earned | ❌ | Total LI mined |
| 5.6 | Display peer count estimate | ❌ | Similar devices mining (from difficulty) |
| 5.7 | Fix contract listing UI | ❌ | Currently broken |
| 5.8 | Tauri native builds | ✅ Done | macOS .dmg, iOS .ipa, Android .apk |

### Phase 6: Polish & Documentation - Priority P3
| # | Task | Status | Notes |
|---|------|--------|-------|
| 6.1 | Prover documentation | ❌ | README, usage examples |
| 6.2 | Contract documentation | ❌ | API reference |
| 6.3 | Integration guide | ❌ | How to add mining to apps |
| 6.4 | Economic model documentation | ❌ | Reward calculation explained |

---

## Task Dependencies

```
                    ┌──> Phase O (Optimization) ──┐
                    │                             │
Phase 2 (Prover) ──┼──> Phase 3 (Contract) ──────┼──> Phase 4 (Validator)
                    │                             │
                    └──> Phase 5 (Frontend) ──────┘
                         (can start 5.1, 5.7, 5.8 in parallel)
```

**Critical Path**:
1. ~~Optimization (O1-O3)~~ - ✅ DONE - achieved 1,420 H/s native, 100-400 H/s WASM
2. ~~Prover (Phase 2)~~ - ✅ DONE - all commands, all platforms verified
3. ~~Contract deployment (3.9)~~ - ✅ DONE → Gas oracle (3.7-3.8) → Validator config (4.1-4.2)
4. **Next**: Gas oracle integration (3.7-3.8) or Frontend (5.x) in parallel

---

## External Dependencies

### Bostrom Blockchain
- RPC: `https://rpc.bostrom.cybernode.ai`
- LCD: `https://lcd.bostrom.cybernode.ai`
- GraphQL: `https://index.bostrom.cybernode.ai/v1/graphql`

### Key Libraries
- `@cosmjs/cosmwasm-stargate` - Contract interactions
- `@cybercongress/cyber-js` - Bostrom-specific client
- `cyber-std` (Rust) - CosmWasm bindings for Cyber

### TokenFactory Module
Bostrom has native TokenFactory for creating/minting tokens from smart contracts.
The LI (Lithium) token will be minted through this module.

### LI Token (Lithium) - DEPLOYED
- **Symbol:** LI
- **Full denom:** `factory/bostrom1qwys5wj3r4lry7dl74ukn5unhdpa6t397h097q36dqvrp5qgvjxqverdlf/li`
- **Contract:** `bostrom1qwys5wj3r4lry7dl74ukn5unhdpa6t397h097q36dqvrp5qgvjxqverdlf`
- **Code ID:** 45
- **Creation:** Automatic on contract instantiation
- **Minting:** On valid PoW proof submission
- **Purpose:** Democratic proof-of-work rewards for miners

---

## Algorithm Parameters (UniversalHash v4 - Spec Compliant)

```rust
CHAINS           = 4          // Parallel computation chains
SCRATCHPAD_KB    = 512        // Per-chain scratchpad (512 KB)
TOTAL_MEMORY     = 2 MB       // 4 x 512KB
ROUNDS           = 12_288     // Iterations per chain (per spec)
BLOCK_SIZE       = 64         // Bytes per memory block
```

**Spec-Compliant Algorithms:**
- Seed generation: `BLAKE3(input || chain × 0x9E3779B97F4A7C15)` (golden ratio)
- Scratchpad init: AES-based expansion (4 AESENC rounds per block)
- Address calculation: `state[0:8] ⊕ state[8:16] ⊕ rotl64(i,13) ⊕ (i × 0x517cc1b727220a95)`
- **Primitive rotation**: `(effective_nonce + chain + round) % 3` where effective_nonce derives from input
- Primitives: AES_Compress (4 rounds), SHA256_Compress (raw), BLAKE3_Compress (7 rounds)
- Finalization: `BLAKE3(SHA256(combined_states))`

**Hardware Acceleration (Native Builds):**
- ARM: AES via `vaeseq_u8/vaesmcq_u8`, SHA256 via `vsha256hq_u32`
- x86: AES via `_mm_aesenc_si128`, SHA256 via `sha2::compress256`
- BLAKE3: Software implementation (7-round compression)
- All primitives <1ns with hardware intrinsics; bottleneck is memory access

### Measured Performance (Spec-Compliant v0.2.3)

**Native - Tauri v2 Benchmarks (Feb 2026):**
| Device | Sustained H/s | Ratio to Mac | Notes |
|--------|---------------|--------------|-------|
| Mac M1/M2 | **1,420** | 1:1 | Desktop baseline |
| iPhone 14 Pro | **900** | 1.6:1 | Near parity |
| Galaxy A56 5G | **400** | 3.5:1 | Mid-range Android |

**Phone-to-desktop ratio: 1.6:1 to 3.5:1** - Target (1:3-5) achieved!

**WASM (Browser) - Measured Performance:**
| Device | Platform | Hashrate |
|--------|----------|----------|
| Mac | Safari/WASM | ~400 H/s |
| iPhone | Safari/WASM | ~207 H/s |
| Android | Chrome/WASM | ~100 H/s |

---

## Build Commands

### Workspace (from project root)

```bash
# Build everything
cargo build --workspace --release

# Run all tests
cargo test --workspace

# Run all benchmarks
cargo bench --workspace

# Run specific crate tests
cargo test -p uhash-core
cargo test -p uhash-prover

# Run prover CLI
cargo run -p uhash-prover --release -- mine
cargo run -p uhash-prover --release -- benchmark -c 100
```

### Makefile (cross-platform builds)

```bash
make help          # Show all available commands
make setup         # Install all dependencies (Rust, Java, Android SDK)
make build         # Build all platforms (WASM, macOS, iOS, Android)
make wasm          # WASM for browsers
make macos         # macOS .dmg
make linux         # Linux .deb + .AppImage
make ios           # iOS .ipa
make android       # Android .apk (signed)
make test          # Run all workspace tests
make bench         # Run all benchmarks
make lint          # Check formatting and clippy
```

### Contract WASM Build (from cw-cyber-merge root)

```bash
docker run --rm -v "$(pwd)":/code \
  -v "/Users/michaelborisov/Develop/universal-hash/crates/core":/uhash-core \
  --mount type=volume,source="cw_cyber_merge_cache",target=/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/optimizer:0.16.0 ./contracts/cw-universal-hash
```

**IMPORTANT**: uhash-core uses `edition = "2024"` but optimizer has Rust 1.78. Must temporarily change to `edition = "2021"` for contract WASM builds, then change back.

---

## Remaining Optimizations (For Future)

| # | Task | Effort | Est. Gain |
|---|------|--------|-----------|
| O3.1 | Profile-Guided Optimization (PGO) | Medium | 1.2-1.5x |
| O3.2 | SIMD block operations (NEON/AVX2) | High | 1.3-1.5x |
| O3.3 | Memory prefetching hints | High | 1.1-1.2x |
| O3.4 | Algorithm parameter tuning | Medium | Variable |

### Performance Summary

| Stage | Hashrate | Time/Hash | Improvement |
|-------|----------|-----------|-------------|
| Baseline (v4) | 16 H/s | 62ms | - |
| After O1+O2 | 57 H/s | 17.5ms | 3.5x |
| v4.1 (tuned) | 880 H/s | 1.1ms | 55x |
| **v0.2.3 (current)** | **1,420 H/s** | **0.7ms** | **89x** ✅ |
| Target | 600-1000 H/s | 1-1.7ms | Exceeded! |

---

## Contact & Resources

- Main discussion: Dima "21xhipster" Starodubcev
- Cyberia GitHub: https://github.com/cyberia-to
- cw-cyber (contracts): https://github.com/cyberia-to/cw-cyber
- cyb-ts (frontend): https://github.com/cyberia-to/cyb-ts
- Cyb App: https://cyb.ai
