# uhash-prover

CLI miner for **LI (Lithium)** tokens on the [Bostrom](https://cyb.ai) blockchain using UniversalHash proof-of-work.

Mines democratic proof-of-work where phones compete with servers (1:3 ratio instead of 1:100+).

## Contract

| | |
|---|---|
| **Chain** | bostrom |
| **Contract** | `bostrom1qwys5wj3r4lry7dl74ukn5unhdpa6t397h097q36dqvrp5qgvjxqverdlf` |
| **LI Token** | `factory/bostrom1qwys5wj3r4lry7dl74ukn5unhdpa6t397h097q36dqvrp5qgvjxqverdlf/li` |
| **Code ID** | 45 |
| **Difficulty** | 16 bits |
| **Reward** | 1,000,000 LI per valid proof |

## Installation

### From Source

```bash
cargo install --path .
```

Requires Rust 1.70+. The binary will be installed as `uhash`.

### Pre-built Binaries

Download from [Releases](https://github.com/cyberia-to/universal-hash/releases).

## Quick Start

```bash
# 1. Create a new wallet
uhash new-wallet

# 2. Fund your wallet with BOOT tokens (for gas fees)
#    Send some BOOT to the address shown above

# 3. Start mining (auto-submits proofs)
uhash mine

# 4. Check your LI balance on https://cyb.ai
```

## Commands

| Command | Description |
|---------|-------------|
| `mine` | Start mining LI tokens (auto-submits proofs) |
| `send` | Submit a specific proof to the chain |
| `new-wallet` | Generate a new wallet |
| `import-mnemonic` | Import wallet from 12/24 word mnemonic |
| `export-mnemonic` | Export wallet mnemonic (backup) |
| `address` | Show wallet address |
| `benchmark` | Run hashrate benchmark |

### Mining

```bash
# Mine with default settings (all cores, auto-submit)
uhash mine

# Use specific thread count
uhash mine --threads 2

# Override difficulty (default: fetched from contract)
uhash mine --difficulty 20

# Mine without auto-submit (print proofs only)
uhash mine --no-submit
```

The miner will:
1. Fetch the current seed and difficulty from the contract
2. Hash in parallel across all CPU cores
3. When a valid proof is found, automatically sign and submit the transaction
4. Print the TX hash with a link to the explorer
5. Continue mining for the next proof

### Manual Proof Submission

```bash
uhash send --hash <hex> --nonce <n> --timestamp <t>
```

### Wallet Management

```bash
# Create new wallet
uhash new-wallet

# Import existing mnemonic
uhash import-mnemonic --phrase "word1 word2 ... word24"

# Export mnemonic (for backup)
uhash export-mnemonic

# Show address
uhash address
```

### Benchmarking

```bash
# Quick benchmark (100 hashes)
uhash benchmark

# Longer benchmark
uhash benchmark --count 1000
```

## Global Options

```bash
# Custom RPC endpoint
uhash --rpc https://rpc.bostrom.cybernode.ai:443 mine

# Custom wallet file
uhash --wallet /path/to/wallet.json mine
```

## Configuration

| Setting | Default |
|---------|---------|
| RPC | `https://rpc.bostrom.cybernode.ai` |
| LCD | `https://lcd.bostrom.cybernode.ai` |
| Wallet | `~/.uhash/wallet.json` |
| Threads | All CPU cores |
| Gas | 20,000,000 |
| Fee | 250,000 boot |

## Performance

| Device | Native H/s | WASM H/s |
|--------|-----------|----------|
| Mac M1/M2 | ~1,420 | ~400 |
| iPhone 14 Pro | ~900 | ~207 |
| Galaxy A56 5G | ~400 | ~100 |

Phone-to-desktop ratio: **1.6:1 to 3.5:1** (target 1:3-5 achieved).

At difficulty 16, a single Mac finds a valid proof roughly every 30-40 seconds.

## Algorithm

Uses [uhash-core](https://github.com/cyberia-to/uhash-core) (v0.2.3) - a memory-hard proof-of-work algorithm designed for mobile fairness:

- **2 MB memory** (4 x 512 KB scratchpads) - fits in L2 cache
- **4 parallel chains** - matches phone core count
- **Triple primitive rotation**: AES + SHA256 + BLAKE3 compression functions
- **Hardware crypto acceleration** on ARM (AES-NI, SHA extensions) and x86
- **ASIC-resistant**: Memory-bound with sequential dependencies

### Self-Authenticating Proofs

```
hash = UniversalHash(seed || miner_address || timestamp || nonce)
```

The miner's address is embedded in the hash input, so changing the address invalidates the proof. No separate signature is needed.

## Architecture

```
uhash-prover (this repo)
├── CLI binary (mine, send, wallet commands)
├── RPC client (LCD queries, tx broadcast)
└── Wallet (bip39 mnemonic, secp256k1 signing)

uhash-core (shared algorithm)
├── UniversalHash v4 implementation
├── Platform-specific intrinsics (ARM, x86, WASM)
└── no_std compatible (runs in CosmWasm)

cw-universal-hash (on-chain verifier)
├── Proof verification
├── LI token minting via TokenFactory
└── Seed rotation & difficulty config
```

## License

Unlicense - Public Domain
