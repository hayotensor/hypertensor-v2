use super::*;
use sp_runtime::traits::BlakeTwo256;
use sp_core::blake2_256;
use sp_std::vec;

impl<T: Config> Pallet<T> {
  pub fn submit_benchmark_weights(
    origin: T::RuntimeOrigin,
    encrypted_weights: BTreeSet<SubnetBenchmarkWeightCommitment>,
    nonce: u32,
  ) -> DispatchResult {
    let subnets: Vec<_> = SubnetsData::<T>::iter().collect();
    let total_subnets: u32 = subnets.len() as u32;
    let total_commitments: u32 = encrypted_weights.len() as u32;

    for (subnet_id, _) in subnets {

    }
    Ok(())
  }

  pub fn compute_benchmark_weights() {

  }

  pub fn reveal(value: u32, seed: &[u8]) -> [u8; 32] {
    let mut data = vec![];
    data.extend_from_slice(&value.to_le_bytes());
    data.extend_from_slice(&seed);
    blake2_256(&data)
  }

  pub fn generate_commitment(value: u32, seed: &[u8]) -> [u8; 32] {
    let mut data = vec![];
    data.extend_from_slice(&value.to_le_bytes());
    data.extend_from_slice(&seed);
    blake2_256(&data)
  }

  // pub fn reveal(value: u32, nonce: u32) -> [u8; 32] {
  //   let recomputed_hash = blake2_256(&[value.to_le_bytes(), nonce.to_le_bytes()].concat());
  //   recomputed_hash
  // }

  // pub fn generate_commitment(value: u32, nonce: u32) -> [u8; 32] {
  //   let mut data = vec![];
  //   data.extend_from_slice(&value.to_le_bytes());
  //   data.extend_from_slice(&nonce.to_le_bytes());
  //   blake2_256(&data)
  // }

  // pub fn encrypt(secret_key: &[u8; 32], nonce: &[u8; 24], plaintext: &[u8]) -> Vec<u8> {
  //   let key = Key::from_slice(secret_key);
  //   let cipher = XChaCha20Poly1305::new(key);

  //   let nonce_obj = XNonce::from_slice(nonce);

  //   cipher.encrypt(nonce_obj, plaintext).expect("Encryption failed")
  // }

  // pub fn decrypt(secret_key: &[u8; 32], nonce: &[u8; 24], ciphertext: &[u8]) -> Vec<u8> {
  //   let key = Key::from_slice(secret_key);
  //   let cipher = XChaCha20Poly1305::new(key);

  //   let nonce_obj = XNonce::from_slice(nonce);

  //   cipher.decrypt(nonce_obj, ciphertext).expect("Decryption failed!")
  // }
}