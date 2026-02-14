# Validator Configuration for UniversalHash Mining

## Overview

UniversalHash is a democratic proof-of-work mining system on the Bostrom blockchain. Miners submit PoW proofs to a CosmWasm smart contract, which verifies the proof and mints LI (Lithium) tokens as rewards.

A key design goal is **permissionless entry**: new miners with zero tokens can start mining immediately. This is possible because Bostrom supports zero-fee transactions natively.

## Validator Requirements

### Zero-Fee Transaction Support

Most Bostrom validators already accept zero-fee transactions. Verify your validator's configuration:

**File:** `~/.cyber/config/app.toml`

```toml
# This should be "0boot" or empty to accept zero-fee transactions
minimum-gas-prices = "0boot"
```

If your validator requires non-zero fees, PoW miners without existing BOOT tokens will be unable to submit proofs.

### Gas Limits

PoW proof submissions consume approximately 400,000-500,000 gas per transaction. The default block gas limit is sufficient to handle these transactions alongside normal chain activity.

No changes to gas limits are needed.

## Spam Protection

Even with zero-fee transactions, the system has multiple layers of spam protection:

### 1. Proof-of-Work Difficulty

The contract enforces a minimum difficulty requirement. Each proof must demonstrate that the miner performed significant computational work (UniversalHash v4 algorithm with 2MB memory, 12,288 rounds per chain). Invalid or below-difficulty proofs are rejected by the contract.

### 2. Dynamic Difficulty Adjustment

The contract automatically adjusts difficulty based on the rolling hashrate. As more miners join, difficulty increases, making spam more computationally expensive.

### 3. Minimum Profitable Difficulty

The contract calculates a `min_profitable_difficulty` threshold. Proofs below this threshold are rejected even if they meet the base difficulty, preventing economic spam where proofs cost more to verify than the rewards they generate.

### 4. Gas Metering

Even with zero fees, Bostrom's gas metering still applies. Each transaction consumes gas and is limited by the block gas limit. This naturally caps the throughput of proof submissions.

### 5. Nonce Uniqueness

Each proof includes a unique nonce and timestamp. The contract rejects duplicate proofs and proofs with timestamps outside the allowed window (`max_proof_age`).

### 6. Self-Authenticating Proofs

The miner's address is included in the hash input: `hash = UniversalHash(seed || address || timestamp || nonce)`. Changing the miner address invalidates the proof, preventing proof theft.

## Economics

Validators benefit from PoW mining activity:

- **Increased transactions**: More transactions means more block rewards for validators
- **Network growth**: Zero-barrier mining attracts new users to Bostrom
- **No cost to validators**: Zero-fee PoW transactions still consume gas and are bounded by block limits

## Verification

### Check your minimum gas prices

```bash
grep minimum-gas-prices ~/.cyber/config/app.toml
```

Expected output:
```
minimum-gas-prices = "0boot"
```

### Test zero-fee acceptance

Submit a test transaction with zero fee:

```bash
cyber tx wasm execute \
  bostrom1qwys5wj3r4lry7dl74ukn5unhdpa6t397h097q36dqvrp5qgvjxqverdlf \
  '{"submit_proof":{"hash":"0000...","nonce":0,"timestamp":0}}' \
  --from <your-key> \
  --fees 0boot \
  --gas 600000 \
  --chain-id bostrom \
  --node https://rpc.bostrom.cybernode.ai:443
```

The transaction should be accepted into the mempool (it will fail contract validation since the proof is invalid, but the zero-fee acceptance can be verified).

### Query contract configuration

```bash
cyber query wasm contract-state smart \
  bostrom1qwys5wj3r4lry7dl74ukn5unhdpa6t397h097q36dqvrp5qgvjxqverdlf \
  '{"config":{}}' \
  --node https://rpc.bostrom.cybernode.ai:443
```
