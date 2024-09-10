use serde_derive::Serialize;
use serde_derive::Deserialize;
use generic_array::GenericArray;
use aes_gcm::{Aes256Gcm, KeyInit};
use aes_gcm::aead::Aead;


// TODO: Using a fixed hard coded KEY for testing purposes only!
const KEY: [u8; 32] = [0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
    0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
    0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef,
    0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef];
const NONCE: [u8; 12] = [0; 12];  // Using a constant nonce for simplicity; in a real application, each nonce should be unique

pub fn encrypt(data: &[u8]) -> Result<Vec<u8>, String> {
    let key = GenericArray::from_slice(&KEY);
    let nonce = GenericArray::from_slice(&NONCE);
    let cipher = Aes256Gcm::new(key);

    match cipher.encrypt(nonce, data.as_ref()) {
        Ok(ciphertext) => Ok(ciphertext),
        Err(_) => Err("Encryption failure!".to_string())
    }
}

pub fn decrypt(encrypted_data: &[u8]) -> Vec<u8> {
    let key = GenericArray::from_slice(&KEY);
    let nonce = GenericArray::from_slice(&NONCE);
    let cipher = Aes256Gcm::new(key);

    let decrypted_data = cipher.decrypt(nonce, encrypted_data.as_ref()).expect("decryption failure!");
    decrypted_data
}

#[derive(Serialize, Deserialize)]
pub struct VpnPacket {
    pub data: Vec<u8>,
}