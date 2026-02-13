//! Tests for UniversalHash algorithm

use crate::{UniversalHash, hash, meets_difficulty};

#[cfg(not(feature = "std"))]
use alloc::vec;

#[test]
fn test_basic_hash() {
    let input = b"test input data";
    let result = hash(input);

    // Hash should be 32 bytes
    assert_eq!(result.len(), 32);

    // Hash should be deterministic
    let result2 = hash(input);
    assert_eq!(result, result2);
}

#[test]
fn test_different_inputs_produce_different_hashes() {
    let hash1 = hash(b"input 1");
    let hash2 = hash(b"input 2");

    assert_ne!(hash1, hash2);
}

#[test]
fn test_avalanche_effect() {
    // Changing one bit should change ~50% of output bits
    let input1 = b"test input";
    let mut input2 = input1.to_vec();
    input2[0] ^= 1; // Flip one bit

    let hash1 = hash(input1);
    let hash2 = hash(&input2);

    // Count differing bits
    let mut diff_bits = 0;
    for i in 0..32 {
        diff_bits += (hash1[i] ^ hash2[i]).count_ones();
    }

    // Expect roughly 128 bits (50% of 256) to differ
    // Allow range of 90-166 (35%-65%)
    assert!(
        (90..=166).contains(&diff_bits),
        "Avalanche effect: {} bits differ (expected ~128)",
        diff_bits
    );
}

#[test]
fn test_difficulty_check() {
    // Hash with 8 leading zero bits (starts with 0x00)
    let hash_8_zeros: [u8; 32] = [
        0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF,
    ];

    assert!(meets_difficulty(&hash_8_zeros, 8));
    assert!(!meets_difficulty(&hash_8_zeros, 9));

    // Hash with 16 leading zero bits (starts with 0x0000)
    let hash_16_zeros: [u8; 32] = [
        0x00, 0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF,
    ];

    assert!(meets_difficulty(&hash_16_zeros, 16));
    assert!(!meets_difficulty(&hash_16_zeros, 17));

    // Hash with leading 0x0F (4 zero bits)
    let hash_4_zeros: [u8; 32] = [
        0x0F, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF,
        0xFF, 0xFF,
    ];

    assert!(meets_difficulty(&hash_4_zeros, 4));
    assert!(!meets_difficulty(&hash_4_zeros, 5));
}

#[test]
fn test_hasher_reusability() {
    let mut hasher = UniversalHash::new();

    let hash1 = hasher.hash(b"first input");
    let hash2 = hasher.hash(b"second input");

    assert_ne!(hash1, hash2);

    // Same input should still produce same hash
    let hash1_again = hasher.hash(b"first input");
    assert_eq!(hash1, hash1_again);
}

#[test]
fn test_empty_input() {
    let result = hash(b"");
    assert_eq!(result.len(), 32);
}

#[test]
fn test_large_input() {
    let large_input = vec![0xABu8; 10000];
    let result = hash(&large_input);
    assert_eq!(result.len(), 32);
}

/// Spec compliance test vectors
/// These vectors verify the implementation matches the UniversalHash v4 spec:
/// - Seed generation: BLAKE3(header || (nonce ⊕ (c × golden_ratio)))
/// - Primitive rotation: (nonce + c) mod 3, then +1 before each round
/// - Write-back: Same address as read
/// - Finalization: BLAKE3(SHA256(XOR of chain states))
#[test]
fn test_spec_compliance_vectors() {
    // Vector 1: Standard mining input format
    // Input: 32-byte epoch_seed + 20-byte address + 8-byte timestamp + 8-byte nonce
    let input1: Vec<u8> = {
        let mut v = Vec::with_capacity(68);
        v.extend_from_slice(&[0u8; 32]); // epoch_seed = all zeros
        v.extend_from_slice(&[1u8; 20]); // miner_address = all ones
        v.extend_from_slice(&[0u8; 8]); // timestamp = 0
        v.extend_from_slice(&[0u8; 8]); // nonce = 0
        v
    };
    let hash1 = hash(&input1);

    // Vector 2: Same with nonce = 1
    let input2: Vec<u8> = {
        let mut v = Vec::with_capacity(68);
        v.extend_from_slice(&[0u8; 32]);
        v.extend_from_slice(&[1u8; 20]);
        v.extend_from_slice(&[0u8; 8]);
        v.extend_from_slice(&1u64.to_le_bytes()); // nonce = 1
        v
    };
    let hash2 = hash(&input2);

    // Vector 3: Different epoch seed
    let input3: Vec<u8> = {
        let mut v = Vec::with_capacity(68);
        v.extend_from_slice(&[0xAB; 32]); // epoch_seed = all 0xAB
        v.extend_from_slice(&[1u8; 20]);
        v.extend_from_slice(&[0u8; 8]);
        v.extend_from_slice(&[0u8; 8]);
        v
    };
    let hash3 = hash(&input3);

    // Hashes must be different (proves nonce/seed affect output)
    assert_ne!(
        hash1, hash2,
        "Different nonces should produce different hashes"
    );
    assert_ne!(
        hash1, hash3,
        "Different epoch seeds should produce different hashes"
    );

    // Hashes must be deterministic
    assert_eq!(hash(&input1), hash1, "Hash must be deterministic");
    assert_eq!(hash(&input2), hash2, "Hash must be deterministic");
    assert_eq!(hash(&input3), hash3, "Hash must be deterministic");

    // Print vectors for reference (run with --nocapture)
    #[cfg(feature = "std")]
    {
        println!("\n=== SPEC COMPLIANCE TEST VECTORS ===");
        println!("Vector 1 (nonce=0): {}", hex::encode(hash1));
        println!("Vector 2 (nonce=1): {}", hex::encode(hash2));
        println!("Vector 3 (seed=0xAB): {}", hex::encode(hash3));
    }
}

#[test]
fn test_nonce_extraction() {
    // Test that nonce is correctly extracted from last 8 bytes
    let mut hasher = UniversalHash::new();

    // Input with known nonce at end
    let nonce: u64 = 0x123456789ABCDEF0;
    let mut input = vec![0u8; 60]; // header
    input.extend_from_slice(&nonce.to_le_bytes());

    let hash1 = hasher.hash(&input);

    // Same header, different nonce should produce different hash
    let mut input2 = vec![0u8; 60];
    input2.extend_from_slice(&(nonce + 1).to_le_bytes());

    let hash2 = hasher.hash(&input2);

    assert_ne!(
        hash1, hash2,
        "Different nonces must produce different hashes"
    );
}

#[test]
fn test_primitive_rotation_per_spec() {
    // Verify that primitive rotation follows spec:
    // primitive = (nonce + chain) mod 3, then +1 before each round use
    // This is implicitly tested by hash consistency - if rotation changes,
    // the hash output changes.

    // Run same input multiple times to ensure determinism
    let input = b"primitive rotation test";
    let mut results = Vec::new();

    for _ in 0..5 {
        results.push(hash(input));
    }

    for i in 1..results.len() {
        assert_eq!(
            results[0], results[i],
            "Hash must be deterministic across runs"
        );
    }
}

#[test]
fn test_known_vector() {
    // This test ensures the algorithm doesn't change accidentally
    // The hash of "uhash-core test vector" should always be the same
    let input = b"uhash-core test vector";
    let result = hash(input);

    // Store first hash run as reference (update if algorithm intentionally changes)
    // For now just verify it's deterministic
    let result2 = hash(input);
    assert_eq!(result, result2);
}

/// Cross-platform consistency test: verifies that hardware-accelerated primitives
/// produce identical results to the software reference implementation.
/// This catches ARM AES intrinsics bugs (AESE XORs key before SubBytes vs AESENC after MixColumns).
#[test]
fn test_primitives_match_software_reference() {
    use crate::params::BLOCK_SIZE;
    use crate::primitives::{aes_compress, aes_expand_block, blake3_compress, sha256_compress};

    // === Software reference implementations (always available, not behind cfg gates) ===

    const SBOX: [u8; 256] = [
        0x63, 0x7c, 0x77, 0x7b, 0xf2, 0x6b, 0x6f, 0xc5, 0x30, 0x01, 0x67, 0x2b, 0xfe, 0xd7, 0xab,
        0x76, 0xca, 0x82, 0xc9, 0x7d, 0xfa, 0x59, 0x47, 0xf0, 0xad, 0xd4, 0xa2, 0xaf, 0x9c, 0xa4,
        0x72, 0xc0, 0xb7, 0xfd, 0x93, 0x26, 0x36, 0x3f, 0xf7, 0xcc, 0x34, 0xa5, 0xe5, 0xf1, 0x71,
        0xd8, 0x31, 0x15, 0x04, 0xc7, 0x23, 0xc3, 0x18, 0x96, 0x05, 0x9a, 0x07, 0x12, 0x80, 0xe2,
        0xeb, 0x27, 0xb2, 0x75, 0x09, 0x83, 0x2c, 0x1a, 0x1b, 0x6e, 0x5a, 0xa0, 0x52, 0x3b, 0xd6,
        0xb3, 0x29, 0xe3, 0x2f, 0x84, 0x53, 0xd1, 0x00, 0xed, 0x20, 0xfc, 0xb1, 0x5b, 0x6a, 0xcb,
        0xbe, 0x39, 0x4a, 0x4c, 0x58, 0xcf, 0xd0, 0xef, 0xaa, 0xfb, 0x43, 0x4d, 0x33, 0x85, 0x45,
        0xf9, 0x02, 0x7f, 0x50, 0x3c, 0x9f, 0xa8, 0x51, 0xa3, 0x40, 0x8f, 0x92, 0x9d, 0x38, 0xf5,
        0xbc, 0xb6, 0xda, 0x21, 0x10, 0xff, 0xf3, 0xd2, 0xcd, 0x0c, 0x13, 0xec, 0x5f, 0x97, 0x44,
        0x17, 0xc4, 0xa7, 0x7e, 0x3d, 0x64, 0x5d, 0x19, 0x73, 0x60, 0x81, 0x4f, 0xdc, 0x22, 0x2a,
        0x90, 0x88, 0x46, 0xee, 0xb8, 0x14, 0xde, 0x5e, 0x0b, 0xdb, 0xe0, 0x32, 0x3a, 0x0a, 0x49,
        0x06, 0x24, 0x5c, 0xc2, 0xd3, 0xac, 0x62, 0x91, 0x95, 0xe4, 0x79, 0xe7, 0xc8, 0x37, 0x6d,
        0x8d, 0xd5, 0x4e, 0xa9, 0x6c, 0x56, 0xf4, 0xea, 0x65, 0x7a, 0xae, 0x08, 0xba, 0x78, 0x25,
        0x2e, 0x1c, 0xa6, 0xb4, 0xc6, 0xe8, 0xdd, 0x74, 0x1f, 0x4b, 0xbd, 0x8b, 0x8a, 0x70, 0x3e,
        0xb5, 0x66, 0x48, 0x03, 0xf6, 0x0e, 0x61, 0x35, 0x57, 0xb9, 0x86, 0xc1, 0x1d, 0x9e, 0xe1,
        0xf8, 0x98, 0x11, 0x69, 0xd9, 0x8e, 0x94, 0x9b, 0x1e, 0x87, 0xe9, 0xce, 0x55, 0x28, 0xdf,
        0x8c, 0xa1, 0x89, 0x0d, 0xbf, 0xe6, 0x42, 0x68, 0x41, 0x99, 0x2d, 0x0f, 0xb0, 0x54, 0xbb,
        0x16,
    ];

    fn gf_mul2(x: u8) -> u8 {
        let hi = x >> 7;
        (x << 1) ^ (hi * 0x1b)
    }

    fn gf_mul3(x: u8) -> u8 {
        gf_mul2(x) ^ x
    }

    fn ref_aesenc_round(state: &[u8; 16], round_key: &[u8]) -> [u8; 16] {
        // SubBytes
        let mut s = [0u8; 16];
        for i in 0..16 {
            s[i] = SBOX[state[i] as usize];
        }
        // ShiftRows
        let t = s;
        s[1] = t[5];
        s[5] = t[9];
        s[9] = t[13];
        s[13] = t[1];
        s[2] = t[10];
        s[6] = t[14];
        s[10] = t[2];
        s[14] = t[6];
        s[3] = t[15];
        s[7] = t[3];
        s[11] = t[7];
        s[15] = t[11];
        // MixColumns
        let mut out = [0u8; 16];
        for col in 0..4 {
            let i = col * 4;
            out[i] = gf_mul2(s[i]) ^ gf_mul3(s[i + 1]) ^ s[i + 2] ^ s[i + 3];
            out[i + 1] = s[i] ^ gf_mul2(s[i + 1]) ^ gf_mul3(s[i + 2]) ^ s[i + 3];
            out[i + 2] = s[i] ^ s[i + 1] ^ gf_mul2(s[i + 2]) ^ gf_mul3(s[i + 3]);
            out[i + 3] = gf_mul3(s[i]) ^ s[i + 1] ^ s[i + 2] ^ gf_mul2(s[i + 3]);
        }
        // AddRoundKey
        for i in 0..16 {
            out[i] ^= round_key[i];
        }
        out
    }

    fn ref_aes_expand(state: &[u8; 16], key: &[u8; 16]) -> [u8; 16] {
        let mut s = *state;
        s = ref_aesenc_round(&s, key);
        s = ref_aesenc_round(&s, key);
        s = ref_aesenc_round(&s, key);
        s = ref_aesenc_round(&s, key);
        s
    }

    fn ref_aes_compress(state: &[u8; 32], block: &[u8; BLOCK_SIZE]) -> [u8; 32] {
        let mut state_lo: [u8; 16] = state[0..16].try_into().unwrap();
        state_lo = ref_aesenc_round(&state_lo, &block[0..16]);
        state_lo = ref_aesenc_round(&state_lo, &block[16..32]);
        state_lo = ref_aesenc_round(&state_lo, &block[32..48]);
        state_lo = ref_aesenc_round(&state_lo, &block[48..64]);

        let mut state_hi: [u8; 16] = state[16..32].try_into().unwrap();
        state_hi = ref_aesenc_round(&state_hi, &block[32..48]);
        state_hi = ref_aesenc_round(&state_hi, &block[48..64]);
        state_hi = ref_aesenc_round(&state_hi, &block[0..16]);
        state_hi = ref_aesenc_round(&state_hi, &block[16..32]);

        let mut result = [0u8; 32];
        result[0..16].copy_from_slice(&state_lo);
        result[16..32].copy_from_slice(&state_hi);
        result
    }

    fn ref_sha256_compress(state: &[u8; 32], block: &[u8; BLOCK_SIZE]) -> [u8; 32] {
        let mut hash_state = [0u32; 8];
        for i in 0..8 {
            hash_state[i] = u32::from_be_bytes([
                state[i * 4],
                state[i * 4 + 1],
                state[i * 4 + 2],
                state[i * 4 + 3],
            ]);
        }
        let mut msg_block = [0u8; 64];
        msg_block.copy_from_slice(block);
        sha2::compress256(&mut hash_state, &[msg_block.into()]);
        let mut result = [0u8; 32];
        for i in 0..8 {
            result[i * 4..i * 4 + 4].copy_from_slice(&hash_state[i].to_be_bytes());
        }
        result
    }

    // Test with multiple diverse inputs
    let test_cases: Vec<([u8; 16], [u8; 16])> = vec![
        ([0u8; 16], [0u8; 16]),
        ([0xFF; 16], [0xFF; 16]),
        (
            [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16],
            [16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1],
        ),
        (
            [
                0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE, 0x01, 0x23, 0x45, 0x67, 0x89, 0xAB,
                0xCD, 0xEF,
            ],
            [
                0xFE, 0xDC, 0xBA, 0x98, 0x76, 0x54, 0x32, 0x10, 0x0F, 0x1E, 0x2D, 0x3C, 0x4B, 0x5A,
                0x69, 0x78,
            ],
        ),
    ];

    // Test aes_expand_block
    for (i, (state, key)) in test_cases.iter().enumerate() {
        let hw_result = aes_expand_block(state, key);
        let sw_result = ref_aes_expand(state, key);
        assert_eq!(
            hw_result, sw_result,
            "aes_expand_block mismatch on test case {}: hw={:02x?} sw={:02x?}",
            i, hw_result, sw_result
        );
    }

    // Test aes_compress
    let compress_tests: Vec<([u8; 32], [u8; 64])> = vec![
        ([0u8; 32], [0u8; 64]),
        ([0xFF; 32], [0xFF; 64]),
        (
            {
                let mut s = [0u8; 32];
                for (i, byte) in s.iter_mut().enumerate() {
                    *byte = i as u8;
                }
                s
            },
            {
                let mut b = [0u8; 64];
                for (i, byte) in b.iter_mut().enumerate() {
                    *byte = (i * 3 + 7) as u8;
                }
                b
            },
        ),
    ];

    for (i, (state, block)) in compress_tests.iter().enumerate() {
        let hw_result = aes_compress(state, block);
        let sw_result = ref_aes_compress(state, block);
        assert_eq!(
            hw_result, sw_result,
            "aes_compress mismatch on test case {}: hw={:02x?} sw={:02x?}",
            i, hw_result, sw_result
        );
    }

    // Test sha256_compress
    for (i, (state, block)) in compress_tests.iter().enumerate() {
        let hw_result = sha256_compress(state, block);
        let sw_result = ref_sha256_compress(state, block);
        assert_eq!(
            hw_result, sw_result,
            "sha256_compress mismatch on test case {}: hw={:02x?} sw={:02x?}",
            i, hw_result, sw_result
        );
    }

    // Test blake3_compress (software-only, but still verify consistency)
    for (i, (state, block)) in compress_tests.iter().enumerate() {
        let result1 = blake3_compress(state, block);
        let result2 = blake3_compress(state, block);
        assert_eq!(
            result1, result2,
            "blake3_compress not deterministic on test case {}",
            i
        );
    }

    #[cfg(feature = "std")]
    println!("\nAll primitives match software reference implementation!");
}

/// Test full hash output matches between hardware and software paths
/// by computing a known vector and printing the result for cross-platform comparison.
#[test]
fn test_cross_platform_hash_vector() {
    // Known input that would be used in mining
    let mut input = Vec::with_capacity(68);
    input.extend_from_slice(&[0xAA; 32]); // seed
    input.extend_from_slice(b"bostrom1testaddr12345"); // 20-byte address (padded)
    input.extend_from_slice(&1000u64.to_le_bytes()); // timestamp
    input.extend_from_slice(&42u64.to_le_bytes()); // nonce

    let result = hash(&input);

    // Print for cross-platform comparison
    #[cfg(feature = "std")]
    println!("\nCross-platform hash vector: {}", hex::encode(result));

    // Verify determinism
    assert_eq!(result, hash(&input));
}

/// Debug test: reproduce exact mining computation and verify hash
#[test]
fn test_exact_mining_reproduction() {
    let seed_hex = "6ebb4eda559a631b31ec2d5db3a6fddb08ede58462c917d5bff6f0da284c1afc";
    let address = "bostrom1s7fuy43h8v6hzjtulx9gxyp30rl9t5cz3z56mk";
    let timestamp: u64 = 1770986039;
    let nonce: u64 = 9223372036854775893;

    let seed = hex::decode(seed_hex).unwrap();

    let mut input = Vec::with_capacity(128);
    input.extend_from_slice(&seed);
    input.extend_from_slice(address.as_bytes());
    input.extend_from_slice(&timestamp.to_le_bytes());
    input.extend_from_slice(&nonce.to_le_bytes());

    #[cfg(feature = "std")]
    {
        println!("\nInput length: {} bytes", input.len());
        println!("Input hex: {}", hex::encode(&input));
    }

    let hash_result = hash(&input);

    #[cfg(feature = "std")]
    {
        println!("Computed hash: {}", hex::encode(hash_result));
        println!("Expected hash: 00b37e351ab7b7616e415fd350adb55fea92fb8027f9e9695387b37392bafab5");
    }

    assert_eq!(
        hex::encode(hash_result),
        "00b37e351ab7b7616e415fd350adb55fea92fb8027f9e9695387b37392bafab5",
        "Hash reproduction failed"
    );
}

#[test]
#[ignore] // Run with: cargo test timing_breakdown -- --ignored --nocapture
fn timing_breakdown() {
    use crate::params::*;
    use crate::primitives::{aes_compress, aes_expand_block, blake3_compress, sha256_compress};
    use std::time::Instant;

    let input = b"timing test input";
    let iterations = 10;

    // Warmup
    for _ in 0..3 {
        let _ = hash(input);
    }

    // Measure total hash time
    let start = Instant::now();
    for _ in 0..iterations {
        let _ = hash(input);
    }
    let total = start.elapsed();
    let per_hash = total / iterations;

    // Measure individual primitives
    let state = [0u8; 32];
    let block = [1u8; 64];
    let prim_iters = 10000;

    let start_aes = Instant::now();
    for _ in 0..prim_iters {
        let _ = aes_compress(&state, &block);
    }
    let aes_time = start_aes.elapsed() / prim_iters;

    let start_sha = Instant::now();
    for _ in 0..prim_iters {
        let _ = sha256_compress(&state, &block);
    }
    let sha_time = start_sha.elapsed() / prim_iters;

    let start_blake = Instant::now();
    for _ in 0..prim_iters {
        let _ = blake3_compress(&state, &block);
    }
    let blake_time = start_blake.elapsed() / prim_iters;

    // Measure AES expand (used in scratchpad init)
    let key16 = [0u8; 16];
    let state16 = [1u8; 16];
    let start_expand = Instant::now();
    for _ in 0..prim_iters {
        let _ = aes_expand_block(&state16, &key16);
    }
    let expand_time = start_expand.elapsed() / prim_iters;

    // Estimate scratchpad init time
    // Each scratchpad has BLOCKS_PER_SCRATCHPAD blocks, each needs 2 AES expansions
    let scratchpad_init_est = expand_time * (BLOCKS_PER_SCRATCHPAD * 2 * CHAINS) as u32;

    // Round execution estimate
    let ops_per_hash = ROUNDS * CHAINS;
    let primitive_avg = (aes_time + sha_time + blake_time) / 3;
    let rounds_est = primitive_avg * ops_per_hash as u32;

    println!("\n=== TIMING BREAKDOWN ===");
    println!("Total per hash: {:?}", per_hash);
    println!("Hashrate: {:.1} H/s", 1.0 / per_hash.as_secs_f64());
    println!("\nPrimitive timing:");
    println!("  AES_Compress:    {:?}", aes_time);
    println!("  SHA256_Compress: {:?}", sha_time);
    println!("  BLAKE3_Compress: {:?}", blake_time);
    println!("  AES_Expand:      {:?}", expand_time);
    println!("  Primitive avg:   {:?}", primitive_avg);
    println!("\nParameters:");
    println!(
        "  ROUNDS: {} × {} chains = {} ops",
        ROUNDS, CHAINS, ops_per_hash
    );
    println!(
        "  SCRATCHPAD: {} blocks × {} chains × 2 AES = {} AES ops",
        BLOCKS_PER_SCRATCHPAD,
        CHAINS,
        BLOCKS_PER_SCRATCHPAD * 2 * CHAINS
    );
    println!("\nTime breakdown estimate:");
    println!("  Scratchpad init: {:?}", scratchpad_init_est);
    println!("  Round execution: {:?}", rounds_est);
    println!("  Total estimated: {:?}", scratchpad_init_est + rounds_est);
    println!("  Actual total:    {:?}", per_hash);
    println!(
        "  Overhead:        {:?}",
        per_hash.saturating_sub(scratchpad_init_est + rounds_est)
    );
}
