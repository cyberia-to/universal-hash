//! Wallet management for UniversalHash miner
//!
//! Handles mnemonic generation, import/export, and transaction signing.

use bip32::secp256k1::ecdsa::SigningKey;
use bip32::{DerivationPath, XPrv};
use bip39::{Language, Mnemonic};
use cosmrs::crypto::secp256k1;
use cosmrs::AccountId;
use std::fs;
use std::path::PathBuf;
use thiserror::Error;

/// Default derivation path for Cosmos SDK chains
const DERIVATION_PATH: &str = "m/44'/118'/0'/0/0";

/// Bostrom address prefix
const BOSTROM_PREFIX: &str = "bostrom";

#[derive(Error, Debug)]
pub enum WalletError {
    #[error("Failed to generate mnemonic: {0}")]
    MnemonicGeneration(String),

    #[error("Invalid mnemonic phrase: {0}")]
    InvalidMnemonic(String),

    #[error("Derivation error: {0}")]
    Derivation(String),

    #[error("File I/O error: {0}")]
    FileError(#[from] std::io::Error),

    #[error("Invalid wallet file format")]
    InvalidFormat,
}

/// A wallet containing a mnemonic and derived keys
pub struct Wallet {
    mnemonic: Mnemonic,
    signing_key: SigningKey,
    address: AccountId,
}

impl Wallet {
    /// Create a new wallet with a random mnemonic
    pub fn new() -> Result<Self, WalletError> {
        // Generate 32 bytes of entropy for 24-word mnemonic
        let mut entropy = [0u8; 32];
        getrandom::getrandom(&mut entropy)
            .map_err(|e| WalletError::MnemonicGeneration(e.to_string()))?;

        let mnemonic = Mnemonic::from_entropy_in(Language::English, &entropy)
            .map_err(|e| WalletError::MnemonicGeneration(e.to_string()))?;

        Self::from_mnemonic(mnemonic)
    }

    /// Create a wallet from an existing mnemonic phrase
    pub fn from_phrase(phrase: &str) -> Result<Self, WalletError> {
        let mnemonic = Mnemonic::parse_in(Language::English, phrase)
            .map_err(|e| WalletError::InvalidMnemonic(e.to_string()))?;

        Self::from_mnemonic(mnemonic)
    }

    /// Create a wallet from a Mnemonic
    fn from_mnemonic(mnemonic: Mnemonic) -> Result<Self, WalletError> {
        let seed = mnemonic.to_seed("");

        let path: DerivationPath = DERIVATION_PATH
            .parse()
            .map_err(|e: bip32::Error| WalletError::Derivation(e.to_string()))?;

        let xprv = XPrv::derive_from_path(seed, &path)
            .map_err(|e| WalletError::Derivation(e.to_string()))?;

        let signing_key = xprv.private_key();

        // Derive address from public key
        let public_key = secp256k1::SigningKey::from_slice(&signing_key.to_bytes())
            .map_err(|e| WalletError::Derivation(e.to_string()))?
            .public_key();

        let address = public_key
            .account_id(BOSTROM_PREFIX)
            .map_err(|e| WalletError::Derivation(e.to_string()))?;

        Ok(Self {
            mnemonic,
            signing_key: signing_key.clone(),
            address,
        })
    }

    /// Get the mnemonic phrase
    pub fn mnemonic(&self) -> String {
        self.mnemonic.to_string()
    }

    /// Get the Bostrom address
    pub fn address(&self) -> &AccountId {
        &self.address
    }

    /// Get the address as a string
    pub fn address_str(&self) -> String {
        self.address.to_string()
    }

    /// Get the signing key for transaction signing
    pub fn signing_key(&self) -> &SigningKey {
        &self.signing_key
    }

    /// Save wallet mnemonic to a file (encrypted with password in future)
    pub fn save_to_file(&self, path: &PathBuf) -> Result<(), WalletError> {
        // For now, save as plaintext (TODO: add encryption)
        fs::write(path, self.mnemonic())?;
        Ok(())
    }

    /// Load wallet from a file
    pub fn load_from_file(path: &PathBuf) -> Result<Self, WalletError> {
        let content = fs::read_to_string(path)?;
        let phrase = content.trim();
        Self::from_phrase(phrase)
    }
}

impl Default for Wallet {
    fn default() -> Self {
        Self::new().expect("Failed to create wallet")
    }
}

/// Get the default wallet file path
#[cfg(feature = "cli")]
pub fn default_wallet_path() -> PathBuf {
    let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home.join(".uhash").join("wallet.txt")
}

/// Ensure the wallet directory exists
#[cfg(feature = "cli")]
pub fn ensure_wallet_dir() -> Result<PathBuf, WalletError> {
    let wallet_path = default_wallet_path();
    if let Some(parent) = wallet_path.parent() {
        fs::create_dir_all(parent)?;
    }
    Ok(wallet_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_wallet() {
        let wallet = Wallet::new().unwrap();
        let phrase = wallet.mnemonic();

        // Should have 24 words
        assert_eq!(phrase.split_whitespace().count(), 24);

        // Address should start with bostrom
        assert!(wallet.address_str().starts_with("bostrom"));
    }

    #[test]
    fn test_wallet_from_phrase() {
        let wallet1 = Wallet::new().unwrap();
        let phrase = wallet1.mnemonic();

        let wallet2 = Wallet::from_phrase(&phrase).unwrap();

        // Same mnemonic should produce same address
        assert_eq!(wallet1.address_str(), wallet2.address_str());
    }

    #[test]
    fn test_deterministic_derivation() {
        // Known test mnemonic (12 words for simplicity)
        let phrase = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";

        let wallet1 = Wallet::from_phrase(phrase).unwrap();
        let wallet2 = Wallet::from_phrase(phrase).unwrap();

        assert_eq!(wallet1.address_str(), wallet2.address_str());
    }
}
