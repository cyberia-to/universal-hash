//! UniversalHash Prover Library
//!
//! A democratic proof-of-work mining system for the Bostrom blockchain.
//!
//! # Overview
//!
//! UniversalHash (U-Hash) enables fair token distribution where smartphones
//! and consumer devices can meaningfully compete with servers and GPUs.
//!
//! # Features
//!
//! - **Democratic Mining**: Phone-to-desktop ratio of 1:3-5
//! - **ASIC Resistance**: Multi-primitive design (AES + SHA-256 + BLAKE3)
//! - **Memory-Hard**: 2MB scratchpad prevents GPU parallelism
//! - **Self-Authenticating**: Proofs include miner address, no signature needed
//!
//! # Example
//!
//! ```rust
//! use uhash::algorithm::{hash, meets_difficulty};
//!
//! // Create proof input
//! let input = b"epoch_seed|miner_address|timestamp|nonce";
//!
//! // Compute hash
//! let result = hash(input);
//!
//! // Check if it meets difficulty requirement
//! if meets_difficulty(&result, 16) {
//!     println!("Valid proof found!");
//! }
//! ```

// Re-export the core algorithm
pub use uhash_core as algorithm;

pub mod rpc;
pub mod wallet;

// Convenience re-exports
pub use algorithm::{hash, meets_difficulty, UniversalHash};
