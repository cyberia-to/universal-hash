# Changelog

All notable changes to uhash-core will be documented in this file.

## [0.2.5] - 2026-02-15

### Added

- **iOS build** (cyb-ts): Full Tauri v2 iOS support
  - Platform-specific config (`tauri.ios.conf.json`) — single window, no splash screen
  - Desktop/mobile code split with `#[cfg(desktop)]` guards
  - Heavy backend disabled on mobile (CozoDb, IPFS, ML embeddings, sync loops, Rune engine)
  - Mining-only invoke handler on mobile (no IPFS commands)
- **Android build** (cyb-ts): Tauri v2 Android APK support (aarch64 only)
  - Platform-specific config (`tauri.android.conf.json`)
  - Desktop-only deps gated with `cfg(not(any(target_os = "android", target_os = "ios")))` — avoids OpenSSL/RocksDB cross-compilation
  - BuildTask.kt fix: `npx @tauri-apps/cli` instead of `npm run tauri`
- **Continuous mining**: Mining no longer pauses during proof submission
  - Native (Rust): `pending_proofs: Vec<FoundProof>` queue, new `take_proofs` command drains queue
  - WASM: Workers keep hashing after finding proof, proofs queued in `pendingProofs[]`
  - Mining.tsx: Polling loop drains proof queue, submits async (fire-and-forget)
- **Network stats**: `useMinerStats` hook queries on-chain `{ stats: {} }` for unique miner count
- **Mobile wallet connection**: "Connect wallet" button on Tauri mobile (replaces "choose address in keplr")
- **Safe area support**: `viewport-fit=cover` + `env(safe-area-inset-*)` on header, footer, action bars, modals
- **Responsive modals**: ConnectWalletModal responsive for phone screens (auto-fill mnemonic grid, max-width constraints)

### Changed

- Mining docs updated with iOS/Android build instructions and continuous mining note

## [0.2.4] - 2026-02-14

### Added

- **Mining dashboard UI** (cyb-ts): Full-featured mining page with:
  - Hero hashrate display with CSS pulse glow animation
  - SVG sparkline chart showing rolling hashrate history
  - 4-card stat grid: LI Mined, Proofs, Est. LI/hr, Elapsed
  - LI wallet balance (polled every 30s)
  - Reward estimate from on-chain `calculate_reward` query
  - Thread selector (range input for CPU core count)
  - Proof log with TX explorer links, OK/FAIL status pills, relative timestamps
  - Wallet address display with copy button + Mining/Idle status pill
- **Mining state persistence**: Proof log and session LI saved to localStorage; on-mount recovery detects active mining in Rust backend
- **New hooks**: `useLiBalance`, `useRewardEstimate`, `useHashrateSamples`
- **New components**: `HashrateHero`, `StatCard`, `ProofLogEntry`, `ThreadSelector`

### Fixed

- **Hash input format** (critical): Tauri miner now builds hash input as binary structured bytes (`seed_raw_32B + address_utf8 + timestamp_8B_LE + nonce_8B_LE`) matching the on-chain contract verification exactly. Previously used string concatenation which produced different hashes.
- **Gas limit**: Increased from 600k (`fee(3)`) to 1.6M (`fee(8)`) for `submit_proof` transactions. On-chain hash verification requires more gas than the previous limit allowed.

## [0.2.3] - 2026-02-12

### Added

- **Makefile build system**: Single command builds for all platforms
  - `make setup` - Install all dependencies (Rust, Java, Android SDK)
  - `make build` - Build all platforms (WASM, macOS, iOS, Android)
  - `make run-{platform}` - Build and run on device
  - `make install-{platform}` - Install to connected device
- **Unified frontend**: Single `index.html` auto-detects Native vs WASM mode
- **Run targets**: `make run-ios`, `make run-android` for quick device testing

### Changed

- WASM now builds to `demo/dist/wasm/` for unified frontend
- Updated documentation with benchmark results and build instructions

### Fixed

- Removed unused imports in `lib.rs` (eliminated WASM build warnings)
- Suppressed Kotlin deprecation warnings in Android build
- Suppressed Gradle deprecation warnings

### Benchmarks

| Platform | Device | Hashrate |
|----------|--------|----------|
| Native | Mac M1/M2 | 1,420 H/s |
| Native | iPhone 14 Pro | 900 H/s |
| Native | Galaxy A56 5G | 400 H/s |
| WASM | Mac Safari | ~400 H/s |
| WASM | iPhone Safari | ~207 H/s |
| WASM | Android Chrome | ~100 H/s |

## [0.2.2] - 2026-02-12

### Fixed

- Fixed WASM build compatibility (array conversion in no_std mode)

## [0.2.1] - 2026-02-12

### Fixed

- Updated crate description to be blockchain-agnostic (algorithm is generic, not tied to any specific chain)

## [0.2.0] - 2026-02-12

### Breaking Changes

This release includes breaking changes to ensure full compliance with the UniversalHash v4 specification. Hash outputs will differ from v0.1.0.

### Fixed

- **Write-back address**: Now writes to the same address that was read from (per spec section 5.3.3), instead of computing a new address from the updated state
- **Primitive rotation**: Fixed to match spec formula exactly:
  - Initial: `primitive = (nonce + chain) mod 3`
  - Each round: `primitive = (initial + round + 1) mod 3` (increment BEFORE use)
- **Seed generation**: Now uses XOR per spec: `BLAKE3(header || (nonce ⊕ (c × golden_ratio)))` instead of concatenation
- **Nonce extraction**: Nonce is now correctly extracted from the last 8 bytes of input

### Removed

- **Cross-chain mixing**: Removed as it was not specified in the algorithm spec

### Added

- Spec compliance test vectors in `src/tests.rs`:
  - `test_spec_compliance_vectors()` - verifies standard mining input format
  - `test_nonce_extraction()` - verifies nonce parsing
  - `test_primitive_rotation_per_spec()` - verifies determinism
- `extract_nonce()` helper function for parsing nonce from input
- `effective_nonce` field in `UniversalHash` struct

### Performance

- ~1,100-1,400 H/s on Apple Silicon (release build)
- Sub-nanosecond primitive operations with hardware crypto acceleration

## [0.1.0] - 2026-02-12

### Added

- Initial implementation of UniversalHash v4 algorithm
- 4 parallel chains with 512KB scratchpads each
- 12,288 rounds per chain
- Triple primitive rotation: AES_Compress, SHA256_Compress, BLAKE3_Compress
- Hardware acceleration for ARM (AES, SHA2) and x86 (AES-NI)
- Software fallback for WASM and older CPUs
- `no_std` support for CosmWasm integration
- Parallel processing with rayon (optional)
- C FFI bindings for iOS/Android
- Criterion benchmarks
