use uhash_core::{UniversalHash, meets_difficulty};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct Benchmark {
    hasher: UniversalHash,
}

impl Default for Benchmark {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl Benchmark {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            hasher: UniversalHash::new(),
        }
    }

    /// Run benchmark with specified number of hashes
    /// Returns hashrate in H/s
    #[wasm_bindgen]
    pub fn run(&mut self, num_hashes: u32) -> f64 {
        let window = web_sys::window().unwrap();
        let performance = window.performance().unwrap();

        let start = performance.now();

        for i in 0..num_hashes {
            let input = format!("benchmark_input_{}", i);
            let _ = self.hasher.hash(input.as_bytes());
        }

        let end = performance.now();
        let elapsed_ms = end - start;
        let elapsed_s = elapsed_ms / 1000.0;

        (num_hashes as f64) / elapsed_s
    }

    /// Get algorithm parameters as JSON string
    #[wasm_bindgen]
    pub fn get_params(&self) -> String {
        format!(
            r#"{{"chains": {}, "scratchpad_kb": {}, "total_mb": {}, "rounds": {}}}"#,
            uhash_core::CHAINS,
            uhash_core::SCRATCHPAD_SIZE / 1024,
            uhash_core::TOTAL_MEMORY / (1024 * 1024),
            uhash_core::ROUNDS
        )
    }
}

/// Single hash function for testing
#[wasm_bindgen]
pub fn hash_once(input: &[u8]) -> Vec<u8> {
    uhash_core::hash(input).to_vec()
}

/// Mining struct for Web Worker usage.
/// Reuses UniversalHash across batches to avoid 2MB re-allocation per hash.
#[wasm_bindgen]
pub struct Miner {
    hasher: UniversalHash,
    seed_bytes: Vec<u8>,
    address_bytes: Vec<u8>,
    timestamp_bytes: [u8; 8],
    difficulty: u32,
}

#[wasm_bindgen]
impl Miner {
    #[wasm_bindgen(constructor)]
    pub fn new(seed_hex: &str, address: &str, timestamp: f64, difficulty: u32) -> Miner {
        let seed_bytes = hex::decode(seed_hex).unwrap_or_else(|_| seed_hex.as_bytes().to_vec());
        let address_bytes = address.as_bytes().to_vec();
        let timestamp_bytes = (timestamp as u64).to_le_bytes();
        Miner {
            hasher: UniversalHash::new(),
            seed_bytes,
            address_bytes,
            timestamp_bytes,
            difficulty,
        }
    }

    /// Mine a batch of nonces. Returns JSON string:
    /// `{"found":true,"hash":"...","nonce":N,"count":M}` or `{"found":false,"count":M}`
    ///
    /// - `start_nonce`: first nonce to try (as f64, safe up to 2^53)
    /// - `nonce_step`: increment between nonces (for interleaved multi-worker mining)
    /// - `batch_size`: number of nonces to try in this batch
    pub fn mine_batch(&mut self, start_nonce: f64, nonce_step: u32, batch_size: u32) -> String {
        let mut nonce = start_nonce as u64;
        let step = nonce_step as u64;
        let capacity = self.seed_bytes.len() + self.address_bytes.len() + 16;

        for i in 0..batch_size {
            let mut input = Vec::with_capacity(capacity);
            input.extend_from_slice(&self.seed_bytes);
            input.extend_from_slice(&self.address_bytes);
            input.extend_from_slice(&self.timestamp_bytes);
            input.extend_from_slice(&nonce.to_le_bytes());

            let hash = self.hasher.hash(&input);

            if meets_difficulty(&hash, self.difficulty) {
                let hash_hex = hex::encode(hash);
                return format!(
                    r#"{{"found":true,"hash":"{}","nonce":{},"count":{}}}"#,
                    hash_hex,
                    nonce,
                    i + 1
                );
            }

            nonce += step;
        }

        format!(r#"{{"found":false,"count":{}}}"#, batch_size)
    }
}
