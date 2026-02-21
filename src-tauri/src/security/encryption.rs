//! AES-256-GCM encryption for sensitive data

use crate::security::{Result, SecurityError};
use aes_gcm::{
    aead::{Aead, AeadCore, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use sha2::{Digest, Sha256};

const KEY_SIZE: usize = 32; // 256 bits
const NONCE_SIZE: usize = 12; // 96 bits for GCM
const TAG_SIZE: usize = 16; // 128-bit authentication tag

/// Encrypted data with nonce
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct EncryptedData {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

/// Derive a key from a password using SHA-256
pub fn derive_key(password: &str, salt: &[u8]) -> [u8; KEY_SIZE] {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(salt);
    let result = hasher.finalize();
    let mut key = [0u8; KEY_SIZE];
    key.copy_from_slice(&result[..KEY_SIZE]);
    key
}

/// Encrypt data using AES-256-GCM
pub fn encrypt_data(plaintext: &[u8], key: &[u8; KEY_SIZE]) -> Result<EncryptedData> {
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);

    let ciphertext = cipher
        .encrypt(&nonce, plaintext)
        .map_err(|e| SecurityError::Encryption(e.to_string()))?;

    Ok(EncryptedData {
        ciphertext,
        nonce: nonce.to_vec(),
    })
}

/// Decrypt data using AES-256-GCM
pub fn decrypt_data(encrypted: &EncryptedData, key: &[u8; KEY_SIZE]) -> Result<Vec<u8>> {
    let cipher = Aes256Gcm::new(key.into());
    let nonce = Nonce::from_slice(&encrypted.nonce);

    cipher
        .decrypt(nonce, encrypted.ciphertext.as_ref())
        .map_err(|e| SecurityError::Decryption(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derive_key() {
        let key1 = derive_key("test_password", b"test_salt");
        let key2 = derive_key("test_password", b"test_salt");
        assert_eq!(key1, key2);

        let key3 = derive_key("different", b"test_salt");
        assert_ne!(key1, key3);
    }

    #[test]
    fn test_encrypt_decrypt() {
        let key = derive_key("test_password", b"test_salt");
        let plaintext = b"Hello, World!";

        let encrypted = encrypt_data(plaintext, &key).unwrap();
        let decrypted = decrypt_data(&encrypted, &key).unwrap();

        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_wrong_key_fails() {
        let key1 = derive_key("password1", b"salt");
        let key2 = derive_key("password2", b"salt");
        let plaintext = b"Secret data";

        let encrypted = encrypt_data(plaintext, &key1).unwrap();
        assert!(decrypt_data(&encrypted, &key2).is_err());
    }
}
