//! UniversalHash Prover CLI
//!
//! A command-line tool for mining U-Hash tokens on the Bostrom blockchain.
//!
//! # Commands
//!
//! - `mine` - Start mining
//! - `send` - Submit a proof to the chain
//! - `import-mnemonic` - Import a wallet from mnemonic phrase
//! - `export-mnemonic` - Export the wallet mnemonic
//! - `benchmark` - Run performance benchmark

use clap::{Parser, Subcommand};
use std::path::PathBuf;
use std::time::{Duration, Instant};

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

    /// Custom wallet file path
    #[arg(long, global = true)]
    wallet: Option<PathBuf>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start mining U-Hash tokens
    Mine {
        /// Number of threads to use (default: number of CPU cores)
        #[arg(short, long)]
        threads: Option<usize>,

        /// Target difficulty (number of leading zero bits)
        #[arg(short, long, default_value = "16")]
        difficulty: u32,
    },

    /// Submit a proof to the chain
    Send {
        /// The hash to submit
        #[arg(long)]
        hash: String,

        /// The nonce used
        #[arg(long)]
        nonce: u64,
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

    let result = match cli.command {
        Commands::Mine {
            threads,
            difficulty,
        } => cmd_mine(threads, difficulty),
        Commands::Send { hash, nonce } => cmd_send(&hash, nonce),
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

fn cmd_mine(threads: Option<usize>, difficulty: u32) -> anyhow::Result<()> {
    let wallet_path = default_wallet_path();

    if !wallet_path.exists() {
        anyhow::bail!(
            "No wallet found. Create one with 'uhash new-wallet' or 'uhash import-mnemonic'"
        );
    }

    let wallet = Wallet::load_from_file(&wallet_path)?;
    let address = wallet.address_str();

    println!("Starting mining...");
    println!("Address: {}", address);
    println!("Difficulty: {} bits", difficulty);

    let num_threads = threads.unwrap_or_else(num_cpus::get);
    println!("Threads: {}", num_threads);

    // TODO: Get epoch seed from contract
    let epoch_seed = [0u8; 32]; // Placeholder

    let mut hasher = UniversalHash::new();
    let mut nonce: u64 = 0;
    let mut hashes: u64 = 0;
    let start = Instant::now();
    let mut last_report = Instant::now();

    loop {
        // Build input: epoch_seed || miner_address || timestamp || nonce
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut input = Vec::with_capacity(128);
        input.extend_from_slice(&epoch_seed);
        input.extend_from_slice(address.as_bytes());
        input.extend_from_slice(&timestamp.to_le_bytes());
        input.extend_from_slice(&nonce.to_le_bytes());

        let result = hasher.hash(&input);
        hashes += 1;

        if meets_difficulty(&result, difficulty) {
            println!("\nFound valid hash!");
            println!("Hash: {}", hex::encode(result));
            println!("Nonce: {}", nonce);
            println!("Hashes computed: {}", hashes);

            // TODO: Submit to chain
            println!("\nNote: Auto-submission not yet implemented. Use 'uhash send' to submit.");
            break;
        }

        nonce = nonce.wrapping_add(1);

        // Report hashrate every 5 seconds
        if last_report.elapsed() >= Duration::from_secs(5) {
            let elapsed = start.elapsed().as_secs_f64();
            let hashrate = hashes as f64 / elapsed;
            print!(
                "\rHashrate: {:.2} H/s | Hashes: {} | Time: {:.0}s",
                hashrate, hashes, elapsed
            );
            use std::io::Write;
            std::io::stdout().flush().ok();
            last_report = Instant::now();
        }
    }

    Ok(())
}

fn cmd_send(hash_hex: &str, nonce: u64) -> anyhow::Result<()> {
    let wallet_path = default_wallet_path();

    if !wallet_path.exists() {
        anyhow::bail!("No wallet found. Create one with 'uhash new-wallet'");
    }

    let wallet = Wallet::load_from_file(&wallet_path)?;

    println!("Submitting proof...");
    println!("From: {}", wallet.address_str());
    println!("Hash: {}", hash_hex);
    println!("Nonce: {}", nonce);

    // TODO: Implement actual submission
    println!("\nNote: Proof submission not yet implemented - waiting for contract deployment.");

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
