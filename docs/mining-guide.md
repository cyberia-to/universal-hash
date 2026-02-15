# Mining Guide

Start mining LI (Lithium) tokens on the Bostrom blockchain with the UniversalHash prover.

## Install

### Download Pre-built Binary

Download the latest release for your platform from the [releases page](https://github.com/cyberia-to/universal-hash/releases):

- **macOS (Apple Silicon)**: `uhash-aarch64-apple-darwin`
- **macOS (Intel)**: `uhash-x86_64-apple-darwin`
- **Linux (x86_64)**: `uhash-x86_64-unknown-linux-gnu`
- **Linux (ARM64)**: `uhash-aarch64-unknown-linux-gnu`
- **Windows**: `uhash-x86_64-pc-windows-msvc.exe`

Make the binary executable (macOS/Linux):

```bash
chmod +x uhash
sudo mv uhash /usr/local/bin/
```

### Build from Source

Requires Rust 1.78+:

```bash
git clone https://github.com/cyberia-to/universal-hash.git
cd universal-hash
cargo build -p uhash-prover --release
# Binary at target/release/uhash
```

## Create a Wallet

Generate a new wallet:

```bash
uhash new-wallet
```

This creates a wallet file at `~/.uhash/wallet.json` and displays your Bostrom address.

**Important:** Back up your mnemonic phrase immediately:

```bash
uhash export-mnemonic
```

Store the 24-word phrase securely. If you lose it, your wallet and any mined LI tokens are unrecoverable.

### Import an Existing Wallet

If you already have a Bostrom mnemonic:

```bash
uhash import-mnemonic --phrase "word1 word2 word3 ... word24"
```

Or interactively (more secure, phrase not saved in shell history):

```bash
uhash import-mnemonic
```

## Start Mining

```bash
uhash mine
```

That's it. The miner will:

1. Fetch the current seed and difficulty from the on-chain contract
2. Start mining with all available CPU cores
3. Automatically submit valid proofs to the contract
4. Mint LI tokens directly to your wallet address

**Almost no BOOT needed.** Bostrom supports zero-fee transactions, but new wallets need a one-time activation — send 1 boot to your address from any existing wallet to create the account on-chain. After that, all mining and proof submissions are completely free.

### Mining Output

```
=== UniversalHash Miner ===
Contract: bostrom1qwys5wj3r4lry7dl74ukn5unhdpa6t397h097q36dqvrp5qgvjxqverdlf
Address:  bostrom1s7fuy43h8v6hzjtulx9gxyp30rl9t5cz3z56mk
Difficulty: 16 bits
Threads: 8
Seed: a1b2c3...
Auto-submit: on
===========================

Hashrate: 1420 H/s | Hashes: 28400 | Time: 20s | Proofs sent: 0

Found valid proof!
  Hash:      0000ab12cd34...
  Nonce:     4521
  Timestamp: 1707912345

Submitting proof to contract...
Proof accepted! TX: A1B2C3D4E5...
```

## Check Your Rewards

Query your LI token balance:

```bash
curl -s "https://lcd.bostrom.cybernode.ai/cosmos/bank/v1beta1/balances/YOUR_ADDRESS" | \
  jq '.balances[] | select(.denom | contains("li"))'
```

Replace `YOUR_ADDRESS` with your wallet address (shown by `uhash address`).

You can also view your transactions on [cyb.ai](https://cyb.ai).

## Advanced Usage

### Custom Thread Count

Limit mining to specific number of threads:

```bash
uhash mine --threads 4
```

### Difficulty Override

Override the contract's difficulty (useful for testing):

```bash
uhash mine --difficulty 12
```

### Dry Run (No Submission)

Find a proof without submitting it:

```bash
uhash mine --no-submit
```

### Custom Contract

Use a different contract (e.g., test contract):

```bash
uhash --contract bostrom1520mkjwwda7mtvf5wkztyjup2hh4ws26zdhqg0sg86wnxjms5h5s88clqm mine
```

### Custom RPC Endpoint

Connect to a different RPC node:

```bash
uhash --rpc https://rpc.example.com mine
```

### Transaction Fee Override

If your validator requires fees (uncommon on Bostrom):

```bash
uhash --fee 250000 mine
```

### Submit a Previously Found Proof

If you mined with `--no-submit`, submit the proof manually:

```bash
uhash send --hash 0000ab12cd34... --nonce 4521 --timestamp 1707912345
```

### Run a Benchmark

Test your device's hashrate without mining:

```bash
uhash benchmark --count 1000
```

## Cyb App Mining

The [Cyb app](https://cyb.ai) includes a built-in mining dashboard available on all platforms:

### Desktop (macOS, Linux, Windows)

1. Download and install the Cyb desktop app
2. Connect your wallet (import or create mnemonic)
3. Navigate to `/mining`
4. Click **Start Mining**

Native hashrates:
- Mac M1/M2: ~1,420 H/s

### iOS

1. Build from source: `APPLE_DEVELOPMENT_TEAM=<TEAM_ID> npx @tauri-apps/cli ios build`
2. Install the `.ipa` via Xcode or `xcrun devicectl`
3. Open the app, tap **Connect** at the bottom, go to Keys, and import your mnemonic
4. Navigate to `/mining` and start mining

Native hashrates:
- iPhone 14 Pro: ~900 H/s

### Android

1. Build from source (aarch64 only — 32-bit ARM is not supported):
   ```bash
   export ANDROID_HOME="$HOME/Library/Android/sdk"
   export NDK_HOME="$ANDROID_HOME/ndk/<version>"
   export JAVA_HOME="<path-to-jdk17>"
   npx @tauri-apps/cli android build --apk --target aarch64
   ```
2. Sign the APK:
   ```bash
   zipalign -f 4 app-universal-release-unsigned.apk aligned.apk
   apksigner sign --ks release.keystore --out cyb-signed.apk aligned.apk
   ```
3. Install: `adb install cyb-signed.apk`
4. Open the app, connect wallet via mnemonic import, navigate to `/mining`

Native hashrates:
- Galaxy A56 5G: ~400 H/s

### Mining Dashboard Features

The mining page shows:
- Real-time hashrate with sparkline chart
- LI tokens mined this session
- Estimated LI per hour
- Proof log with clickable TX explorer links
- Thread selector (adjust CPU cores used)
- Wallet LI balance (auto-refreshes)
- Active miner count on the network

Mining runs continuously — proof submission happens asynchronously without interrupting hashing. Mining also continues in the background when navigating to other pages.

## Browser Mining (WASM)

UniversalHash also supports mining directly in your browser via WASM. Visit the [Cyb app](https://cyb.ai) or try the [demo benchmark](https://github.com/cyberia-to/universal-hash/tree/dev/crates/demo).

Expected browser hashrates:
- Desktop (Safari/Chrome): ~400 H/s
- iPhone (Safari): ~207 H/s
- Android (Chrome): ~100 H/s

## Troubleshooting

### "No wallet found"

Create a wallet first: `uhash new-wallet`

### "Could not fetch seed/difficulty"

The RPC endpoint may be down. Try a different one: `uhash --rpc https://rpc2.bostrom.cybernode.ai mine`

### Transaction fails

- Check your address has no pending transactions
- Ensure the contract is not paused
- Verify your proof timestamp is within the allowed window (default: 1 hour)

### Low hashrate

- Use native builds (not WASM) for best performance
- Ensure hardware acceleration is enabled: copy `.cargo/config.toml.example` to `.cargo/config.toml` and rebuild
- Close other CPU-intensive applications
