# uhash-prover

CLI miner for UniversalHash proof-of-work on Bostrom blockchain.

## Installation

### From Source

```bash
cargo install --path .
```

### Pre-built Binaries

Download from [Releases](https://github.com/cyberia-to/universal-hash/releases).

## Quick Start

```bash
# Create a new wallet
uhash new-wallet

# Show your address
uhash address

# Start mining
uhash mine --difficulty 16

# Run benchmark
uhash benchmark --count 100
```

## Commands

| Command | Description |
|---------|-------------|
| `uhash mine` | Start mining U-Hash tokens |
| `uhash send` | Submit a proof to the chain |
| `uhash new-wallet` | Generate a new wallet |
| `uhash import-mnemonic` | Import wallet from 12/24 word phrase |
| `uhash export-mnemonic` | Export wallet mnemonic (backup) |
| `uhash address` | Show wallet address |
| `uhash benchmark` | Run performance test |

## Options

```bash
# Use custom RPC endpoint
uhash --rpc https://rpc.bostrom.cybernode.ai mine

# Use custom wallet file
uhash --wallet /path/to/wallet.json mine

# Mining with specific thread count and difficulty
uhash mine --threads 4 --difficulty 20
```

## Default Configuration

| Setting | Default |
|---------|---------|
| RPC | `https://rpc.bostrom.cybernode.ai` |
| Wallet | `~/.uhash/wallet.json` |
| Difficulty | 16 bits |
| Threads | All CPU cores |

## Algorithm

Uses [uhash-core](https://github.com/cyberia-to/uhash-core) - a mobile-friendly proof-of-work algorithm with:
- 2MB memory requirement (ASIC-resistant)
- 4 parallel chains (optimized for phones)
- Hardware crypto acceleration (AES, SHA256)

See uhash-core for algorithm details and benchmarks.

## License

Unlicense - Public Domain
