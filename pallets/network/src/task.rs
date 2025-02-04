// Copyright (C) Hypertensor.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::*;
use x25519_dalek::PublicKey;
use x25519_dalek::StaticSecret;

impl<T: Config> Pallet<T> {
  pub fn submit_task(origin: OriginFor<T>, task_id: u32, data: Vec<u8>, encrypted_responses: BTreeSet<Vec<u8>>) -> DispatchResult {
    let sender = ensure_signed(origin)?;

    ensure!(!Tasks::<T>::contains_key(task_id), Error::<T>::TaskNotFound);

    let task = Task {
        owner: sender.clone(),
        data,
        encrypted_responses,
    };

    Tasks::<T>::insert(task_id, task);

    Self::deposit_event(Event::TaskSubmitted(sender, task_id));
    Ok(())
  }

  /// Submit private key to decrypt results
  pub fn reveal_results(origin: OriginFor<T>, task_id: u32, private_key_bytes: [u8; 32]) -> DispatchResult {
    let sender = ensure_signed(origin)?;

    // Retrieve task
    let task = Tasks::<T>::get(task_id).ok_or(Error::<T>::TaskNotFound)?;

    // Ensure sender is the owner
    ensure!(sender == task.owner, Error::<T>::TaskNotFound);

    // Convert private key bytes into StaticSecret
    let private_key = StaticSecret::from(private_key_bytes);
    let mut decrypted_results: BTreeSet<Vec<u8>> = BTreeSet::new();

    for encrypted_data in task.encrypted_responses.iter() {
      let decrypted = Self::decrypt_data(&private_key, encrypted_data)?;
      decrypted_results.insert(decrypted);
    }

    // Store decrypted results on-chain
    DecryptedResults::<T>::insert(task_id, decrypted_results);

    Self::deposit_event(Event::ResultsDecrypted(task_id));
    Ok(())
  }

  fn decrypt_data(recipient_private_key: &StaticSecret, encrypted_data: &Vec<u8>) -> Result<Vec<u8>, Error<T>> {
    // Extract the recipient's public key from the encrypted data (the first 32 bytes)
    if encrypted_data.len() < 32 {
        return Err(Error::<T>::DecryptionFailed);
    }

    let sender_public_key_bytes: [u8; 32] = encrypted_data[..32].try_into().map_err(|_| Error::<T>::DecryptionFailed)?;
    let sender_public_key = PublicKey::from(sender_public_key_bytes);

    // Derive the shared secret using the recipient's private key and the sender's public key
    let shared_secret = recipient_private_key.diffie_hellman(&sender_public_key);

    // Decrypt the data by XORing it with the shared secret
    let decrypted_data: Vec<u8> = encrypted_data[32..]
        .iter()
        .zip(shared_secret.as_bytes().iter().cycle())
        .map(|(enc_byte, key_byte)| enc_byte ^ key_byte)
        .collect();

    Ok(decrypted_data)
  }
}
