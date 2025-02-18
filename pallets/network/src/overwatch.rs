use super::*;
use chacha20poly1305::{
  aead::{Aead, KeyInit},
  XChaCha20Poly1305, Key, XNonce,
};
use sp_runtime::traits::BlakeTwo256;

impl<T: Config> Pallet<T> {
  pub fn submit_benchmark_weights(
    origin: T::RuntimeOrigin,
    encrypted_benchmark_weights: Vec<u8>,

  ) -> DispatchResult {
    Ok(())
  }

  /// Encrypt data using XChaCha20Poly1305
  pub fn encrypt(secret_key: &[u8; 32], nonce: &[u8; 24], plaintext: &[u8]) -> Vec<u8> {
    let key = Key::from_slice(secret_key);
    let cipher = XChaCha20Poly1305::new(key);

    let nonce_obj = XNonce::from_slice(nonce);

    cipher.encrypt(nonce_obj, plaintext).expect("Encryption failed")
  }

  pub fn decrypt(secret_key: &[u8; 32], nonce: &[u8; 24], ciphertext: &[u8]) -> Vec<u8> {
    let key = Key::from_slice(secret_key);
    let cipher = XChaCha20Poly1305::new(key);

    let nonce_obj = XNonce::from_slice(nonce);

    cipher.decrypt(nonce_obj, ciphertext).expect("Decryption failed!")
  }
}