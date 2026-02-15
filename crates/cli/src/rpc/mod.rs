//! RPC client for Bostrom blockchain interaction
//!
//! Handles submitting proofs and querying chain state.

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Default RPC endpoint for Bostrom
pub const DEFAULT_RPC: &str = "https://rpc.bostrom.cybernode.ai";

/// Default LCD endpoint for Bostrom
pub const DEFAULT_LCD: &str = "https://lcd.bostrom.cybernode.ai";

/// UniversalHash verifier contract address on Bostrom mainnet
pub const CONTRACT_ADDRESS: &str =
    "bostrom1qwys5wj3r4lry7dl74ukn5unhdpa6t397h097q36dqvrp5qgvjxqverdlf";

/// LI token denom
pub const LI_DENOM: &str =
    "factory/bostrom1qwys5wj3r4lry7dl74ukn5unhdpa6t397h097q36dqvrp5qgvjxqverdlf/li";

/// RPC client configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcConfig {
    /// RPC endpoint URL
    pub rpc_url: String,
    /// LCD/REST endpoint URL
    pub lcd_url: String,
    /// Chain ID
    pub chain_id: String,
    /// Contract address for UniversalHash verifier
    pub contract_address: String,
    /// Fee amount in uboot (default: 0 for zero-fee Bostrom transactions)
    pub fee_amount: u128,
}

impl Default for RpcConfig {
    fn default() -> Self {
        Self {
            rpc_url: DEFAULT_RPC.to_string(),
            lcd_url: DEFAULT_LCD.to_string(),
            chain_id: "bostrom".to_string(),
            contract_address: CONTRACT_ADDRESS.to_string(),
            fee_amount: 0,
        }
    }
}

/// Proof submission message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofSubmission {
    /// The computed hash
    pub hash: String,
    /// Nonce used to find the hash
    pub nonce: u64,
    /// Timestamp when mining started
    pub timestamp: u64,
    /// Miner's address
    pub miner_address: String,
}

/// Result of submitting a proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubmitResult {
    /// Transaction hash
    pub tx_hash: String,
    /// Whether the proof was accepted
    pub accepted: bool,
    /// Reward amount if accepted
    pub reward: Option<String>,
    /// Error message if rejected
    pub error: Option<String>,
}

/// Contract execute message for submitting proofs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    SubmitProof {
        hash: String,
        nonce: u64,
        timestamp: u64,
    },
}

/// Contract query message
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    Seed {},
    Difficulty {},
}

/// Config response from contract
/// Supports both production (epoch_duration) and test (period_duration) field names.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigResponse {
    pub token_denom: String,
    pub difficulty: u32,
    pub base_reward: String,
    pub max_proof_age: u64,
    #[serde(alias = "epoch_duration")]
    pub period_duration: u64,
    #[serde(default)]
    pub target_proofs_per_window: Option<u64>,
    pub admin: String,
    pub paused: bool,
}

/// Seed response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedResponse {
    pub seed: String,
    pub seed_interval: u64,
}

/// Difficulty response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifficultyResponse {
    pub current: u32,
    pub min_profitable: u32,
}

/// RPC client for interacting with Bostrom
pub struct RpcClient {
    config: RpcConfig,
    http_client: reqwest::Client,
}

impl RpcClient {
    /// Create a new RPC client with default configuration
    pub fn new() -> Self {
        Self {
            config: RpcConfig::default(),
            http_client: reqwest::Client::new(),
        }
    }

    /// Create a new RPC client with custom configuration
    pub fn with_config(config: RpcConfig) -> Self {
        Self {
            config,
            http_client: reqwest::Client::new(),
        }
    }

    /// Get the current configuration
    pub fn config(&self) -> &RpcConfig {
        &self.config
    }

    /// Query account info (sequence and account number)
    pub async fn get_account_info(&self, address: &str) -> Result<(u64, u64)> {
        let url = format!(
            "{}/cosmos/auth/v1beta1/accounts/{}",
            self.config.lcd_url, address
        );

        let resp: serde_json::Value = self.http_client.get(&url).send().await?.json().await?;

        let account = &resp["account"];
        let sequence: u64 = account["sequence"]
            .as_str()
            .unwrap_or("0")
            .parse()
            .unwrap_or(0);
        let account_number: u64 = account["account_number"]
            .as_str()
            .unwrap_or("0")
            .parse()
            .unwrap_or(0);

        Ok((account_number, sequence))
    }

    /// Broadcast a signed transaction
    pub async fn broadcast_tx(&self, tx_bytes: Vec<u8>) -> Result<String> {
        let url = format!("{}/cosmos/tx/v1beta1/txs", self.config.lcd_url);

        let body = serde_json::json!({
            "tx_bytes": base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &tx_bytes),
            "mode": "BROADCAST_MODE_SYNC"
        });

        let resp: serde_json::Value = self
            .http_client
            .post(&url)
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        if let Some(code) = resp["tx_response"]["code"].as_u64() {
            if code != 0 {
                let raw_log = resp["tx_response"]["raw_log"]
                    .as_str()
                    .unwrap_or("Unknown error");
                anyhow::bail!("Transaction failed with code {}: {}", code, raw_log);
            }
        }

        let tx_hash = resp["tx_response"]["txhash"]
            .as_str()
            .unwrap_or("")
            .to_string();

        Ok(tx_hash)
    }

    /// Submit a proof to the chain
    pub async fn submit_proof(
        &self,
        proof: ProofSubmission,
        signing_key: &cosmrs::crypto::secp256k1::SigningKey,
    ) -> Result<SubmitResult> {
        use cosmrs::cosmwasm::MsgExecuteContract;
        use cosmrs::tx::{Body, Fee, Msg, SignDoc, SignerInfo};
        use cosmrs::{AccountId, Coin};

        // Get account info
        let (account_number, sequence) = self.get_account_info(&proof.miner_address).await?;

        // Build execute message
        let execute_msg = ExecuteMsg::SubmitProof {
            hash: proof.hash.clone(),
            nonce: proof.nonce,
            timestamp: proof.timestamp,
        };
        let msg_bytes = serde_json::to_vec(&execute_msg)?;

        // Parse addresses
        let sender: AccountId = proof
            .miner_address
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid sender address: {}", e))?;
        let contract: AccountId = self
            .config
            .contract_address
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid contract address: {}", e))?;

        // Build MsgExecuteContract
        let msg = MsgExecuteContract {
            sender,
            contract,
            msg: msg_bytes,
            funds: vec![],
        };

        // Convert to Any
        let msg_any = msg
            .to_any()
            .map_err(|e| anyhow::anyhow!("Failed to convert message: {}", e))?;

        // Build transaction body
        let body = Body::new(vec![msg_any], "", 0u32);

        // Build auth info with fee (default 0 for Bostrom zero-fee transactions)
        let denom: cosmrs::Denom = "boot"
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid denom: {}", e))?;
        let fee = Fee::from_amount_and_gas(
            Coin {
                denom,
                amount: self.config.fee_amount,
            },
            600000u64,
        );

        let signer_info = SignerInfo::single_direct(Some(signing_key.public_key()), sequence);
        let auth_info = signer_info.auth_info(fee);

        // Build sign doc
        let chain_id: cosmrs::tendermint::chain::Id = self
            .config
            .chain_id
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid chain ID: {}", e))?;
        let sign_doc = SignDoc::new(&body, &auth_info, &chain_id, account_number)
            .map_err(|e| anyhow::anyhow!("Failed to create sign doc: {}", e))?;

        // Sign
        let tx_signed = sign_doc
            .sign(signing_key)
            .map_err(|e| anyhow::anyhow!("Failed to sign transaction: {}", e))?;
        let tx_bytes = tx_signed
            .to_bytes()
            .map_err(|e| anyhow::anyhow!("Failed to serialize transaction: {}", e))?;

        // Broadcast
        let tx_hash = self.broadcast_tx(tx_bytes).await?;

        Ok(SubmitResult {
            tx_hash,
            accepted: true,
            reward: None, // Will be in events
            error: None,
        })
    }

    /// Query the current mining seed from the contract
    pub async fn get_seed(&self) -> Result<[u8; 32]> {
        let query = QueryMsg::Seed {};
        let query_b64 = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            serde_json::to_vec(&query)?,
        );

        let url = format!(
            "{}/cosmwasm/wasm/v1/contract/{}/smart/{}",
            self.config.lcd_url, self.config.contract_address, query_b64
        );

        let resp: serde_json::Value = self.http_client.get(&url).send().await?.json().await?;

        let seed_hex = resp["data"]["seed"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid seed response"))?;

        let seed_bytes = hex::decode(seed_hex)?;
        let mut seed = [0u8; 32];
        seed.copy_from_slice(&seed_bytes);

        Ok(seed)
    }

    /// Query the current difficulty target
    pub async fn get_difficulty(&self) -> Result<u32> {
        let query = QueryMsg::Difficulty {};
        let query_b64 = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            serde_json::to_vec(&query)?,
        );

        let url = format!(
            "{}/cosmwasm/wasm/v1/contract/{}/smart/{}",
            self.config.lcd_url, self.config.contract_address, query_b64
        );

        let resp: serde_json::Value = self.http_client.get(&url).send().await?.json().await?;

        let difficulty = resp["data"]["current"]
            .as_u64()
            .ok_or_else(|| anyhow::anyhow!("Invalid difficulty response"))?
            as u32;

        Ok(difficulty)
    }

    /// Query the minimum profitable difficulty
    pub async fn get_min_profitable_difficulty(&self) -> Result<u32> {
        let query = QueryMsg::Difficulty {};
        let query_b64 = base64::Engine::encode(
            &base64::engine::general_purpose::STANDARD,
            serde_json::to_vec(&query)?,
        );

        let url = format!(
            "{}/cosmwasm/wasm/v1/contract/{}/smart/{}",
            self.config.lcd_url, self.config.contract_address, query_b64
        );

        let resp: serde_json::Value = self.http_client.get(&url).send().await?.json().await?;

        let min_profitable = resp["data"]["min_profitable"]
            .as_u64()
            .ok_or_else(|| anyhow::anyhow!("Invalid min_profitable response"))?
            as u32;

        Ok(min_profitable)
    }
}

impl Default for RpcClient {
    fn default() -> Self {
        Self::new()
    }
}
