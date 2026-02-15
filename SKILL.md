---
name: uhash-mining
description: Mine LI (Lithium) tokens on Bostrom blockchain using UniversalHash proof-of-work
tools: [Bash]
---

# UniversalHash Mining

Mine LI tokens on the Bostrom blockchain. Zero fees, zero setup tokens needed. Works on any device.

## Prerequisites

The `uhash` binary must be installed. Check with:

```bash
uhash --version
```

If not installed, build from source:

```bash
cd /Users/michaelborisov/Develop/universal-hash && cargo build -p uhash-prover --release
# Binary: target/release/uhash
```

Or download from [releases](https://github.com/cyberia-to/universal-hash/releases).

## Wallet Setup

**Check if wallet exists:**
```bash
uhash address
```

**Create new wallet (if needed):**
```bash
uhash --json new-wallet
```
Returns: `{"address":"bostrom1...","path":"..."}`

**Import existing mnemonic:**
```bash
uhash --json import-mnemonic --phrase "word1 word2 ... word24"
```

**Back up mnemonic (SENSITIVE):**
```bash
uhash --json export-mnemonic
```

## Query Contract Status

Check current mining parameters before starting:

```bash
uhash --json status
```

Returns:
```json
{
  "contract": "bostrom1qwys5wj3r4...",
  "seed": "8aff8c60d4d8...",
  "difficulty": 8,
  "min_profitable_difficulty": 8,
  "base_reward": "1000000",
  "period_duration": 600,
  "paused": false
}
```

Key fields:
- `difficulty` — current mining difficulty in bits
- `min_profitable_difficulty` — minimum difficulty for profitable proofs
- `paused` — if true, mining is disabled
- `period_duration` — seed rotation interval in seconds

## Start Mining

```bash
uhash --json mine --threads 4
```

Emits newline-delimited JSON events to stdout:

1. **mine_started** — confirms mining parameters
   ```json
   {"event":"mine_started","contract":"bostrom1...","address":"bostrom1...","difficulty":8,"threads":4,"seed":"...","auto_submit":true}
   ```

2. **proof_found** — valid proof discovered
   ```json
   {"event":"proof_found","hash":"0000...","nonce":1234,"timestamp":1707912345,"hashes_computed":50000,"hashrate":1420.0}
   ```

3. **proof_submitted** — proof sent to chain
   ```json
   {"event":"proof_submitted","tx_hash":"A1B2C3...","success":true,"proofs_submitted":1}
   ```

The miner runs continuously, finding and auto-submitting proofs in a loop.

### Mine Without Submitting

Find one proof without submitting (dry run):

```bash
uhash --json mine --no-submit
```

### Submit a Proof Manually

```bash
uhash --json send --hash 0000ab12cd34... --nonce 4521 --timestamp 1707912345
```

Returns: `{"tx_hash":"...","success":true}`

## Check LI Balance

Query the miner's LI token balance:

```bash
curl -s "https://lcd.bostrom.cybernode.ai/cosmos/bank/v1beta1/balances/$(uhash address)" | \
  python3 -c "import sys,json; balances=json.load(sys.stdin)['balances']; li=[b for b in balances if 'li' in b['denom']]; print(json.dumps(li[0]) if li else '{\"amount\":\"0\"}')"
```

## Run Benchmark

Test device hashrate without mining:

```bash
uhash --json benchmark -c 100
```

Returns:
```json
{"total_hashes":100,"elapsed_s":0.07,"hashrate":1420.0,"params":{"chains":4,"scratchpad_kb":512,"total_mb":2,"rounds":12288}}
```

Expected hashrates:
- Desktop (M1/M2): ~1,420 H/s
- iPhone 14 Pro: ~900 H/s
- Android mid-range: ~400 H/s

## Advanced Options

All commands accept these global flags:

| Flag | Description | Default |
|------|-------------|---------|
| `--json` | Machine-readable JSON output | off |
| `--contract <ADDR>` | Custom contract address | production contract |
| `--rpc <URL>` | Custom RPC endpoint | `https://rpc.bostrom.cybernode.ai` |
| `--fee <UBOOT>` | Transaction fee | `0` (zero-fee) |
| `--wallet <PATH>` | Custom wallet file | `~/.uhash/wallet.txt` |

## Error Handling

With `--json`, errors are returned as:
```json
{"error":"No wallet found. Create one with 'uhash new-wallet' or 'uhash import-mnemonic'"}
```
Exit code is `1` on error, `0` on success.

## Typical Agent Workflow

1. Check wallet: `uhash --json address`
2. If no wallet: `uhash --json new-wallet`
3. Check status: `uhash --json status` (verify not paused, check difficulty)
4. Run benchmark: `uhash --json benchmark -c 50` (estimate hashrate)
5. Start mining: `uhash --json mine --threads 4`
6. Parse NDJSON events from stdout for proof_found / proof_submitted
7. Check balance periodically via LCD API
