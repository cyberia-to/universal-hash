//! UniversalHash Prover CLI
//!
//! A command-line tool for mining LI (Lithium) tokens on the Bostrom blockchain.
//!
//! # Commands
//!
//! - `mine` - Start mining (multi-threaded, auto-submit)
//! - `send` - Submit a proof to the chain
//! - `import-mnemonic` - Import a wallet from mnemonic phrase
//! - `export-mnemonic` - Export the wallet mnemonic
//! - `benchmark` - Run performance benchmark

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use uhash::rpc::{ProofSubmission, RpcClient};
use uhash::wallet::{default_wallet_path, ensure_wallet_dir, Wallet};
use uhash::{meets_difficulty, UniversalHash};

#[derive(Parser)]
#[command(name = "uhash")]
#[command(author = "Cyberia")]
#[command(version = "0.1.0")]
#[command(about = "UniversalHash proof-of-work miner for Bostrom blockchain")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Custom RPC endpoint
    #[arg(long, global = true)]
    rpc: Option<String>,

    /// Custom contract address (default: production contract)
    #[arg(long, global = true)]
    contract: Option<String>,

    /// Transaction fee in uboot (default: 0 for zero-fee Bostrom transactions)
    #[arg(long, global = true, default_value = "0")]
    fee: u128,

    /// Custom wallet file path
    #[arg(long, global = true)]
    wallet: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start mining LI (Lithium) tokens
    Mine {
        /// Number of threads to use (default: number of CPU cores)
        #[arg(short, long)]
        threads: Option<usize>,

        /// Target difficulty override (default: fetched from contract)
        #[arg(short, long)]
        difficulty: Option<u32>,

        /// Disable auto-submit (just print found proofs)
        #[arg(long)]
        no_submit: bool,
    },

    /// Submit a proof to the chain
    Send {
        /// The hash to submit
        #[arg(long)]
        hash: String,

        /// The nonce used
        #[arg(long)]
        nonce: u64,

        /// The timestamp when mining started (unix seconds)
        #[arg(long)]
        timestamp: u64,
    },

    /// Import a wallet from mnemonic phrase
    ImportMnemonic {
        /// The mnemonic phrase (will prompt if not provided)
        #[arg(long)]
        phrase: Option<String>,
    },

    /// Export the wallet mnemonic phrase
    ExportMnemonic,

    /// Generate a new wallet
    NewWallet,

    /// Show wallet address
    Address,

    /// Run performance benchmark
    Benchmark {
        /// Number of hashes to compute
        #[arg(short, long, default_value = "100")]
        count: u32,
    },
}

fn main() {
    let cli = Cli::parse();

    let rpc_config = build_rpc_config(cli.rpc.as_deref(), cli.contract.as_deref(), cli.fee);

    let result = match cli.command {
        Commands::Mine {
            threads,
            difficulty,
            no_submit,
        } => cmd_mine(threads, difficulty, no_submit, &rpc_config),
        Commands::Send {
            hash,
            nonce,
            timestamp,
        } => cmd_send(&hash, nonce, timestamp, &rpc_config),
        Commands::ImportMnemonic { phrase } => cmd_import_mnemonic(phrase, cli.wallet),
        Commands::ExportMnemonic => cmd_export_mnemonic(cli.wallet),
        Commands::NewWallet => cmd_new_wallet(cli.wallet),
        Commands::Address => cmd_address(cli.wallet),
        Commands::Benchmark { count } => cmd_benchmark(count),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

/// Build RPC config from CLI args
fn build_rpc_config(
    rpc_url: Option<&str>,
    contract: Option<&str>,
    fee: u128,
) -> uhash::rpc::RpcConfig {
    let mut config = uhash::rpc::RpcConfig::default();
    if let Some(url) = rpc_url {
        config.rpc_url = url.to_string();
        config.lcd_url = url.replace("rpc", "lcd");
    }
    if let Some(addr) = contract {
        config.contract_address = addr.to_string();
    }
    config.fee_amount = fee;
    config
}

/// A valid proof found by a mining thread
struct FoundProof {
    hash: Vec<u8>,
    nonce: u64,
    timestamp: u64,
}

fn cmd_mine(
    threads: Option<usize>,
    difficulty_override: Option<u32>,
    no_submit: bool,
    rpc_config: &uhash::rpc::RpcConfig,
) -> anyhow::Result<()> {
    let wallet_path = default_wallet_path();

    if !wallet_path.exists() {
        anyhow::bail!(
            "No wallet found. Create one with 'uhash new-wallet' or 'uhash import-mnemonic'"
        );
    }

    let wallet = Wallet::load_from_file(&wallet_path)?;
    let address = wallet.address_str();

    // Create RPC client
    let client = RpcClient::with_config(rpc_config.clone());

    let rt = tokio::runtime::Runtime::new()?;

    // Fetch difficulty from contract (unless overridden)
    let difficulty = if let Some(d) = difficulty_override {
        println!("Using difficulty override: {} bits", d);
        d
    } else {
        println!("Fetching difficulty from contract...");
        match rt.block_on(client.get_difficulty()) {
            Ok(d) => {
                println!("Contract difficulty: {} bits", d);
                d
            }
            Err(e) => {
                eprintln!(
                    "Warning: Could not fetch difficulty ({}), using default 16",
                    e
                );
                16
            }
        }
    };

    // Query mining seed from contract
    println!("Fetching seed from contract...");
    let epoch_seed = rt.block_on(client.get_seed()).unwrap_or_else(|e| {
        eprintln!("Warning: Could not fetch seed ({}), using zeros", e);
        [0u8; 32]
    });

    let num_threads = threads.unwrap_or_else(num_cpus::get);

    println!("\n=== UniversalHash Miner ===");
    println!("Contract: {}", rpc_config.contract_address);
    println!("Address:  {}", address);
    println!("Difficulty: {} bits", difficulty);
    println!("Threads: {}", num_threads);
    println!("Seed: {}", hex::encode(epoch_seed));
    println!("Auto-submit: {}", if no_submit { "off" } else { "on" });
    println!("===========================\n");

    // Shared state for threads
    let total_hashes = Arc::new(AtomicU64::new(0));
    let found = Arc::new(std::sync::Mutex::new(None::<FoundProof>));
    let stop = Arc::new(AtomicBool::new(false));

    // Get signing key for auto-submit
    let signing_key =
        cosmrs::crypto::secp256k1::SigningKey::from_slice(&wallet.signing_key().to_bytes())
            .map_err(|e| anyhow::anyhow!("Invalid signing key: {}", e))?;

    let mut proofs_submitted: u64 = 0;

    loop {
        // Reset for new round
        stop.store(false, Ordering::SeqCst);
        *found.lock().unwrap() = None;
        total_hashes.store(0, Ordering::Relaxed);

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let start = Instant::now();

        // Spawn mining threads
        let mut handles = Vec::with_capacity(num_threads);
        for thread_id in 0..num_threads {
            let address = address.clone();
            let total_hashes = Arc::clone(&total_hashes);
            let found = Arc::clone(&found);
            let stop = Arc::clone(&stop);

            // Each thread uses interleaved nonces: thread_id, thread_id + N, thread_id + 2N, ...
            // This keeps all nonces small and avoids JSON precision issues with u64 > 2^53
            let handle = std::thread::spawn(move || {
                let mut hasher = UniversalHash::new();
                let mut nonce = thread_id as u64;

                while !stop.load(Ordering::Relaxed) {
                    let mut input = Vec::with_capacity(128);
                    input.extend_from_slice(&epoch_seed);
                    input.extend_from_slice(address.as_bytes());
                    input.extend_from_slice(&timestamp.to_le_bytes());
                    input.extend_from_slice(&nonce.to_le_bytes());

                    let result = hasher.hash(&input);
                    total_hashes.fetch_add(1, Ordering::Relaxed);

                    if meets_difficulty(&result, difficulty) {
                        let mut guard = found.lock().unwrap();
                        if guard.is_none() {
                            *guard = Some(FoundProof {
                                hash: result.to_vec(),
                                nonce,
                                timestamp,
                            });
                            stop.store(true, Ordering::SeqCst);
                        }
                        return;
                    }

                    nonce += num_threads as u64;
                }
            });
            handles.push(handle);
        }

        // Monitor progress while threads work
        loop {
            std::thread::sleep(Duration::from_secs(2));

            let hashes = total_hashes.load(Ordering::Relaxed);
            let elapsed = start.elapsed().as_secs_f64();
            let hashrate = if elapsed > 0.0 {
                hashes as f64 / elapsed
            } else {
                0.0
            };

            if stop.load(Ordering::Relaxed) {
                break;
            }

            print!(
                "\rHashrate: {:.0} H/s | Hashes: {} | Time: {:.0}s | Proofs sent: {}",
                hashrate, hashes, elapsed, proofs_submitted
            );
            use std::io::Write;
            std::io::stdout().flush().ok();
        }

        // Wait for all threads to finish
        for handle in handles {
            let _ = handle.join();
        }

        // Process found proof
        let proof_data = found.lock().unwrap().take();
        if let Some(proof) = proof_data {
            let hashes = total_hashes.load(Ordering::Relaxed);
            let elapsed = start.elapsed().as_secs_f64();

            println!("\n\nFound valid proof!");
            println!("  Hash:      {}", hex::encode(&proof.hash));
            println!("  Nonce:     {}", proof.nonce);
            println!("  Timestamp: {}", proof.timestamp);
            println!(
                "  Hashes:    {} ({:.0} H/s)",
                hashes,
                hashes as f64 / elapsed
            );

            if no_submit {
                println!("\nTo submit this proof, run:");
                println!(
                    "  uhash send --hash {} --nonce {} --timestamp {}",
                    hex::encode(&proof.hash),
                    proof.nonce,
                    proof.timestamp
                );
                // In no-submit mode, exit after first proof
                break;
            }

            // Auto-submit
            println!("\nSubmitting proof to contract...");
            let submission = ProofSubmission {
                hash: hex::encode(&proof.hash),
                nonce: proof.nonce,
                timestamp: proof.timestamp,
                miner_address: address.clone(),
            };

            match rt.block_on(client.submit_proof(submission, &signing_key)) {
                Ok(result) => {
                    proofs_submitted += 1;
                    println!("Proof accepted! TX: {}", result.tx_hash);
                    println!("View: https://cyb.ai/network/bostrom/tx/{}", result.tx_hash);
                }
                Err(e) => {
                    eprintln!("Submit failed: {}. Continuing to mine...", e);
                }
            }

            println!("\nContinuing to mine...\n");
            // Loop continues — mine next proof
        } else {
            // Interrupted without finding proof
            break;
        }
    }

    Ok(())
}

fn cmd_send(
    hash_hex: &str,
    nonce: u64,
    timestamp: u64,
    rpc_config: &uhash::rpc::RpcConfig,
) -> anyhow::Result<()> {
    let wallet_path = default_wallet_path();

    if !wallet_path.exists() {
        anyhow::bail!("No wallet found. Create one with 'uhash new-wallet'");
    }

    let wallet = Wallet::load_from_file(&wallet_path)?;

    println!("Submitting proof to contract...");
    println!("Contract: {}", rpc_config.contract_address);
    println!("From: {}", wallet.address_str());
    println!("Hash: {}", hash_hex);
    println!("Nonce: {}", nonce);
    println!("Timestamp: {}", timestamp);

    // Create RPC client
    let client = RpcClient::with_config(rpc_config.clone());

    // Build proof submission
    let proof = ProofSubmission {
        hash: hash_hex.to_string(),
        nonce,
        timestamp,
        miner_address: wallet.address_str(),
    };

    // Get signing key from wallet
    let signing_key =
        cosmrs::crypto::secp256k1::SigningKey::from_slice(&wallet.signing_key().to_bytes())
            .map_err(|e| anyhow::anyhow!("Invalid signing key: {}", e))?;

    // Submit using tokio runtime
    let rt = tokio::runtime::Runtime::new()?;
    let result = rt.block_on(client.submit_proof(proof, &signing_key))?;

    println!("\nProof submitted successfully!");
    println!("Transaction hash: {}", result.tx_hash);
    println!(
        "\nView on explorer: https://cyb.ai/network/bostrom/tx/{}",
        result.tx_hash
    );

    Ok(())
}

fn cmd_import_mnemonic(phrase: Option<String>, wallet_path: Option<PathBuf>) -> anyhow::Result<()> {
    let phrase = match phrase {
        Some(p) => p,
        None => {
            println!("Enter your 24-word mnemonic phrase:");
            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            input.trim().to_string()
        }
    };

    let wallet = Wallet::from_phrase(&phrase)?;
    let path = wallet_path
        .unwrap_or_else(|| ensure_wallet_dir().expect("Failed to create wallet directory"));

    wallet.save_to_file(&path)?;

    println!("Wallet imported successfully!");
    println!("Address: {}", wallet.address_str());
    println!("Saved to: {}", path.display());

    Ok(())
}

fn cmd_export_mnemonic(wallet_path: Option<PathBuf>) -> anyhow::Result<()> {
    let path = wallet_path.unwrap_or_else(default_wallet_path);

    if !path.exists() {
        anyhow::bail!("No wallet found at {}", path.display());
    }

    let wallet = Wallet::load_from_file(&path)?;

    println!("WARNING: Keep this mnemonic phrase secret and secure!");
    println!("\n{}\n", wallet.mnemonic());

    Ok(())
}

fn cmd_new_wallet(wallet_path: Option<PathBuf>) -> anyhow::Result<()> {
    let path = wallet_path
        .unwrap_or_else(|| ensure_wallet_dir().expect("Failed to create wallet directory"));

    if path.exists() {
        anyhow::bail!(
            "Wallet already exists at {}. Use 'uhash export-mnemonic' to backup, then delete the file to create a new one.",
            path.display()
        );
    }

    let wallet = Wallet::new()?;
    wallet.save_to_file(&path)?;

    println!("New wallet created!");
    println!("Address: {}", wallet.address_str());
    println!("Saved to: {}", path.display());
    println!("\nIMPORTANT: Backup your mnemonic phrase with 'uhash export-mnemonic'");

    Ok(())
}

fn cmd_address(wallet_path: Option<PathBuf>) -> anyhow::Result<()> {
    let path = wallet_path.unwrap_or_else(default_wallet_path);

    if !path.exists() {
        anyhow::bail!("No wallet found. Create one with 'uhash new-wallet'");
    }

    let wallet = Wallet::load_from_file(&path)?;
    println!("{}", wallet.address_str());

    Ok(())
}

fn cmd_benchmark(count: u32) -> anyhow::Result<()> {
    println!("Running benchmark with {} hashes...", count);

    let mut hasher = UniversalHash::new();
    let input = b"benchmark input data for UniversalHash v4";

    let start = Instant::now();

    for i in 0..count {
        let mut data = input.to_vec();
        data.extend_from_slice(&i.to_le_bytes());
        let _ = hasher.hash(&data);
    }

    let elapsed = start.elapsed();
    let hashrate = count as f64 / elapsed.as_secs_f64();

    println!("\nResults:");
    println!("  Total hashes: {}", count);
    println!("  Time elapsed: {:.2}s", elapsed.as_secs_f64());
    println!("  Hashrate: {:.2} H/s", hashrate);

    // Memory info
    println!("\nAlgorithm parameters:");
    println!("  Chains: {}", uhash_core::CHAINS);
    println!(
        "  Memory per chain: {} KB",
        uhash_core::SCRATCHPAD_SIZE / 1024
    );
    println!(
        "  Total memory: {} MB",
        uhash_core::TOTAL_MEMORY / (1024 * 1024)
    );
    println!("  Rounds: {}", uhash_core::ROUNDS);

    Ok(())
}
