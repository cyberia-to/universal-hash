# UniversalHash

CLI miner for **LI (Lithium)** tokens on the [Bostrom](https://cyb.ai) blockchain using UniversalHash proof-of-work.

Democratic proof-of-work where phones compete with servers (1:3 ratio instead of 1:100+). Zero fees required.

## Contract

| | |
|---|---|
| **Chain** | bostrom |
| **Contract** | `bostrom1qwys5wj3r4lry7dl74ukn5unhdpa6t397h097q36dqvrp5qgvjxqverdlf` |
| **LI Token** | `factory/bostrom1qwys5wj3r4lry7dl74ukn5unhdpa6t397h097q36dqvrp5qgvjxqverdlf/li` |
| **Code ID** | 45 |
| **Reward** | 1,000,000 LI per valid proof |

## Installation

### From Source

```bash
cargo build -p uhash-prover --release
# Binary: target/release/uhash
```

Requires Rust 1.78+.

### Pre-built Binaries

Download from [Releases](https://github.com/cyberia-to/universal-hash/releases).

### WASM (npm)

```bash
npm install uhash-web
```

## Quick Start

```bash
# 1. Create a new wallet
uhash new-wallet

# 2. Start mining (auto-submits proofs, zero fees)
uhash mine

# 3. Check your LI balance on https://cyb.ai
```

No BOOT tokens needed — Bostrom supports zero-fee transactions.

## Commands

| Command | Description |
|---------|-------------|
| `mine` | Start mining LI tokens (auto-submits proofs) |
| `send` | Submit a specific proof to the chain |
| `status` | Query contract state (seed, difficulty, config) |
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

### Contract Status

```bash
uhash status
# or with JSON output:
uhash --json status
```

Returns seed, difficulty, min profitable difficulty, base reward, and period duration.

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

## JSON Output (Agent Integration)

All commands support the `--json` flag for machine-readable output, enabling integration with AI agents (Claude Code, OpenClaw, LangChain, etc.):

```bash
# Structured benchmark output
uhash --json benchmark -c 100
# {"total_hashes":100,"elapsed_s":0.07,"hashrate":1420.0,"params":{"chains":4,"scratchpad_kb":512,"total_mb":2,"rounds":12288}}

# Contract status
uhash --json status
# {"contract":"bostrom1...","seed":"8aff...","difficulty":8,"min_profitable_difficulty":8,"base_reward":"1000000","period_duration":600}

# Mining emits NDJSON events
uhash --json mine
# {"event":"mine_started","contract":"bostrom1...","address":"bostrom1...","difficulty":8,"threads":8,"seed":"...","auto_submit":true}
# {"event":"proof_found","hash":"0000...","nonce":1234,"timestamp":1707912345,"hashes_computed":50000,"hashrate":1420.0}
# {"event":"proof_submitted","tx_hash":"A1B2C3...","success":true,"proofs_submitted":1}

# Errors return structured JSON with exit code 1
uhash --json mine
# {"error":"No wallet found. Create one with 'uhash new-wallet' or 'uhash import-mnemonic'"}
```

See [`SKILL.md`](SKILL.md) for a complete agent skill definition compatible with Claude Code and OpenClaw.

## Global Options

| Flag | Description | Default |
|------|-------------|---------|
| `--json` | Machine-readable JSON output | off |
| `--rpc <URL>` | Custom RPC endpoint | `https://rpc.bostrom.cybernode.ai` |
| `--contract <ADDR>` | Custom contract address | production contract |
| `--fee <UBOOT>` | Transaction fee in uboot | `0` (zero-fee) |
| `--wallet <PATH>` | Custom wallet file | `~/.uhash/wallet.txt` |

## Configuration

| Setting | Default |
|---------|---------|
| RPC | `https://rpc.bostrom.cybernode.ai` |
| LCD | `https://lcd.bostrom.cybernode.ai` |
| Wallet | `~/.uhash/wallet.txt` |
| Threads | All CPU cores |
| Gas | 600,000 |
| Fee | 0 boot (zero-fee) |

## Performance

| Device | Native H/s | WASM H/s |
|--------|-----------|----------|
| Mac M1/M2 | ~1,420 | ~400 |
| iPhone 14 Pro | ~900 | ~207 |
| Galaxy A56 5G | ~400 | ~100 |

Phone-to-desktop ratio: **1.6:1 to 3.5:1** (target 1:3-5 achieved).

## Algorithm

UniversalHash v4 — a memory-hard proof-of-work algorithm designed for mobile fairness:

- **2 MB memory** (4 x 512 KB scratchpads) — fits in L2 cache
- **4 parallel chains** — matches phone core count
- **Triple primitive rotation**: AES + SHA256 + BLAKE3 compression functions
- **Hardware crypto acceleration** on ARM (AES, SHA2 extensions) and x86 (AES-NI)
- **ASIC-resistant**: Memory-bound with sequential dependencies

### Self-Authenticating Proofs

```
hash = UniversalHash(seed || miner_address || timestamp || nonce)
```

The miner's address is embedded in the hash input, so changing the address invalidates the proof. No separate signature is needed.

## Architecture

```
universal-hash/
├── crates/
│   ├── cli/          uhash-prover — mining CLI binary
│   ├── core/         uhash-core — algorithm library (no_std)
│   ├── web/          uhash-web — WASM bindings (npm: uhash-web)
│   └── demo/         uhash-demo — Tauri v2 benchmark app
├── SKILL.md          Agent skill for AI integration
├── Makefile          Cross-platform build system
└── .github/          CI/CD workflows

cw-universal-hash (separate repo: cw-cyber)
├── Proof verification
├── LI token minting via TokenFactory
└── Seed rotation & difficulty adjustment
```

## License

Unlicense — Public Domain
