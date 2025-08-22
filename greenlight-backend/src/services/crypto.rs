use aes_gcm::{Aes256Gcm, Nonce, KeyInit};
use aes_gcm::aead::Aead;
use pbkdf2::pbkdf2_hmac;
use sha2::Sha256;
use rand::{RngCore, rngs::OsRng};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use argon2::password_hash::{rand_core, SaltString};
use bip39::{Mnemonic, Language};
use base64::{Engine as _, engine::general_purpose};
use crate::error::{AppError, Result};

const PBKDF2_ITERATIONS: u32 = 100_000;
const AES_KEY_SIZE: usize = 32;
const NONCE_SIZE: usize = 12;
const SALT_SIZE: usize = 32;

pub struct CryptoService;

impl CryptoService {
    /// Hash a password using Argon2
    pub fn hash_password(password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut rand_core::OsRng);
        let argon2 = Argon2::default();
        
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| AppError::Cryptography(format!("Failed to hash password: {}", e)))?;

        Ok(password_hash.to_string())
    }

    /// Verify a password against its hash
    pub fn verify_password(password: &str, hash: &str) -> Result<bool> {
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|e| AppError::Cryptography(format!("Invalid hash format: {}", e)))?;

        let argon2 = Argon2::default();
        Ok(argon2.verify_password(password.as_bytes(), &parsed_hash).is_ok())
    }

    /// Generate a new BIP39 mnemonic
    pub fn generate_mnemonic() -> Result<String> {
        let mut entropy = [0u8; 32];
        OsRng.fill_bytes(&mut entropy);
        let mnemonic = Mnemonic::from_entropy(&entropy)
            .map_err(|e| AppError::Cryptography(format!("Failed to generate mnemonic: {}", e)))?;
        
        Ok(mnemonic.to_string())
    }

    /// Derive an encryption key from a password using PBKDF2
    fn derive_key(password: &str, salt: &[u8]) -> [u8; AES_KEY_SIZE] {
        let mut key = [0u8; AES_KEY_SIZE];
        pbkdf2_hmac::<Sha256>(password.as_bytes(), salt, PBKDF2_ITERATIONS, &mut key);
        key
    }

    /// Encrypt data using AES-256-GCM
    pub fn encrypt(data: &str, password: &str) -> Result<String> {
        // Generate random salt and nonce
        let mut salt = [0u8; SALT_SIZE];
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut salt);
        OsRng.fill_bytes(&mut nonce_bytes);

        // Derive key from password
        let key_bytes = Self::derive_key(password, &salt);
        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        // Encrypt the data
        let cipher = Aes256Gcm::new(key);
        let ciphertext = cipher
            .encrypt(nonce, data.as_bytes())
            .map_err(|e| AppError::Cryptography(format!("Encryption failed: {}", e)))?;

        // Combine salt + nonce + ciphertext and encode as base64
        let mut result = Vec::new();
        result.extend_from_slice(&salt);
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        Ok(general_purpose::STANDARD.encode(result))
    }

    /// Decrypt data using AES-256-GCM
    pub fn decrypt(encrypted_data: &str, password: &str) -> Result<String> {
        // Decode from base64
        let data = general_purpose::STANDARD
            .decode(encrypted_data)
            .map_err(|e| AppError::Cryptography(format!("Invalid base64: {}", e)))?;

        if data.len() < SALT_SIZE + NONCE_SIZE {
            return Err(AppError::Cryptography("Invalid encrypted data length".to_string()));
        }

        // Extract salt, nonce, and ciphertext
        let salt = &data[0..SALT_SIZE];
        let nonce_bytes = &data[SALT_SIZE..SALT_SIZE + NONCE_SIZE];
        let ciphertext = &data[SALT_SIZE + NONCE_SIZE..];

        // Derive key from password
        let key_bytes = Self::derive_key(password, salt);
        let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&key_bytes);
        let nonce = Nonce::from_slice(nonce_bytes);

        // Decrypt the data
        let cipher = Aes256Gcm::new(key);
        let plaintext = cipher
            .decrypt(nonce, ciphertext)
            .map_err(|e| AppError::Cryptography(format!("Decryption failed: {}", e)))?;

        String::from_utf8(plaintext)
            .map_err(|e| AppError::Cryptography(format!("Invalid UTF-8: {}", e)))
    }

    /// Validate a BIP39 mnemonic
    pub fn validate_mnemonic(mnemonic: &str) -> Result<bool> {
        match Mnemonic::parse_in_normalized(Language::English, mnemonic) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Convert mnemonic to seed bytes
    pub fn mnemonic_to_seed(mnemonic: &str) -> Result<Vec<u8>> {
        let mnemonic = Mnemonic::parse_in_normalized(Language::English, mnemonic)
            .map_err(|e| AppError::Cryptography(format!("Invalid mnemonic: {}", e)))?;
        
        Ok(mnemonic.to_seed("").to_vec())
    }
}
