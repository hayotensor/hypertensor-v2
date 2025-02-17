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
use frame_system::pallet_prelude::BlockNumberFor;
use libm::exp;

impl<T: Config> Pallet<T> {
  pub fn get_current_block_as_u64() -> u64 {
    TryInto::try_into(<frame_system::Pallet<T>>::block_number())
      .ok()
      .expect("blockchain will not exceed 2^64 blocks; QED.")
  }

  pub fn convert_block_as_u64(block: BlockNumberFor<T>) -> u64 {
    TryInto::try_into(block)
      .ok()
      .expect("blockchain will not exceed 2^64 blocks; QED.")
  }
  
  pub fn get_eligible_epoch_block(epoch_length: u64, initialized: u64, epochs: u64) -> u64 {
    let eligible_block: u64 = initialized - (initialized % epoch_length) + epoch_length * epochs;
    eligible_block
  }

  // Loosely validates Node ID
  pub fn validate_peer_id(peer_id: PeerId) -> bool {
    let peer_id_0 = peer_id.0;
    let len = peer_id_0.len();

    // PeerId must be equal to or greater than 32 chars
    // PeerId must be equal to or less than 128 chars
    if len < 32 || len > 128 {
      return false
    };

    let first_char = peer_id_0[0];
    let second_char = peer_id_0[1];
    if first_char == 49 {
      // Node ID (ed25519, using the "identity" multihash) encoded as a raw base58btc multihash
      return len <= 128
    } else if first_char == 81 && second_char == 109 {
      // Node ID (sha256) encoded as a raw base58btc multihash
      return len <= 128;
    } else if first_char == 102 || first_char == 98 || first_char == 122 || first_char == 109 {
      // Node ID (sha256) encoded as a CID
      return len <= 128;
    }
    
    false
  }
  
  pub fn is_subnet_node_eligible(subnet_node: u32, account_id: T::AccountId) -> bool {
    let max_subnet_node_penalties = MaxSubnetNodePenalties::<T>::get();
    let penalties = SubnetNodePenalties::<T>::get(subnet_node, account_id);
    penalties <= max_subnet_node_penalties
  }

  pub fn get_tx_rate_limit() -> u64 {
    TxRateLimit::<T>::get()
  }

  pub fn set_last_tx_block(key: &T::AccountId, block: u64) {
    LastTxBlock::<T>::insert(key, block)
  }

  pub fn get_last_tx_block(key: &T::AccountId) -> u64 {
    LastTxBlock::<T>::get(key)
  }

  pub fn exceeds_tx_rate_limit(prev_tx_block: u64, current_block: u64) -> bool {
    let rate_limit: u64 = Self::get_tx_rate_limit();
    if rate_limit == 0 || prev_tx_block == 0 {
      return false;
    }

    return current_block - prev_tx_block <= rate_limit;
  }

  // If a subnet or subnet peer is able to be included or submit consensus
  //
  // This checks if the block is equal to or greater than therefor shouldn't 
  // be used while checking if a subnet or subnet peer was able to accept or be 
  // included in consensus during the forming of consensus since it checks for
  // the previous epochs eligibility
  pub fn is_epoch_block_eligible(
    block: u64, 
    epoch_length: u64, 
    epochs: u64, 
    initialized: u64
  ) -> bool {
    block >= Self::get_eligible_epoch_block(
      epoch_length, 
      initialized, 
      epochs
    )
  }

  /// Remove subnet peer from subnet
  // to-do: Add slashing to subnet peers stake balance
  pub fn perform_remove_subnet_node(block: u64, subnet_id: u32, account_id: T::AccountId) {
    // Take and remove SubnetNodesData account_id as key
    // `take()` returns and removes data
    if let Ok(subnet_node) = SubnetNodesData::<T>::try_get(subnet_id, account_id.clone()) {
      let peer_id = subnet_node.peer_id;

      // Remove from attestations
      let epoch_length: u64 = T::EpochLength::get();
			let epoch: u64 = block / epoch_length;

      let submittable_nodes: BTreeSet<T::AccountId> = Self::get_classified_accounts(subnet_id, &SubnetNodeClass::Validator, epoch);

      SubnetRewardsSubmission::<T>::try_mutate_exists(
        subnet_id,
        epoch as u32,
        |params| -> DispatchResult {
          let params = if let Some(params) = params {
            // --- Remove from consensus
            let mut data = &mut params.data;
            data.retain(|x| x.peer_id != peer_id);
            params.data = data.clone();
            
            // --- Remove from attestations
            let mut attests = &mut params.attests;
            if attests.remove(&account_id.clone()).is_some() {
              params.attests = attests.clone();
            }
          };
          Ok(())
        }
      );
    
      let subnet_node = SubnetNodesData::<T>::take(subnet_id, account_id.clone());

      if subnet_node.a.is_some() {
        SubnetNodeUniqueParam::<T>::remove(subnet_id, subnet_node.a.unwrap())
      }

      // Remove SubnetNodeAccount peer_id as key
      SubnetNodeAccount::<T>::remove(subnet_id, peer_id.clone());
      // Update total subnet peers by substracting 1
      TotalSubnetNodes::<T>::mutate(subnet_id, |n: &mut u32| n.saturating_dec());
      TotalActiveSubnetNodes::<T>::mutate(subnet_id, |n: &mut u32| n.saturating_dec());

      // Reset sequential absent subnet node count
      SubnetNodePenalties::<T>::remove(subnet_id, account_id.clone());

			Self::deposit_event(Event::SubnetNodeRemoved { subnet_id: subnet_id, account_id: account_id });
    }
  }


  pub fn do_clear_subnet(
    subnet_id: u32,
  ) {

  }

  pub fn get_min_subnet_nodes(base_node_memory: u128, memory_mb: u128) -> u32 {
    // TODO: Needs to be updated for smoother curve

    // log::error!(" ");
    // log::error!("get_min_subnet_nodes base_node_memory {:?}", base_node_memory);
    // log::error!("get_min_subnet_nodes memory_mb {:?}", memory_mb);

    // --- DEFAULT
    // --- Get min nodes based on default memory settings
    let simple_min_subnet_nodes: u128 = Self::percent_div(
      memory_mb * Self::PERCENTAGE_FACTOR, 
      base_node_memory * Self::PERCENTAGE_FACTOR
    );

    // --- Parameters
    let params: CurveParametersSet = MinNodesCurveParameters::<T>::get();
    let one_hundred = Self::PERCENTAGE_FACTOR;
    let x_curve_start = params.x_curve_start;
    let y_end = params.y_end;
    let y_start = params.y_start;
    let x_rise = Self::PERCENTAGE_FACTOR / 100;
    let max_x = params.max_x;

    let max_subnet_memory = MaxSubnetMemoryMB::<T>::get();

    let mut subnet_mem_position = Self::PERCENTAGE_FACTOR;
    
    // --- Get the x axis of the memory based on min and max
    // Redundant since subnet memory cannot be surpassed beyond the max subnet memory
    // If max subnet memory in curve is surpassed
    if memory_mb < max_subnet_memory {
      subnet_mem_position = memory_mb * Self::PERCENTAGE_FACTOR / max_subnet_memory;
    }

    let mut min_subnet_nodes: u128 = MinSubnetNodes::<T>::get() as u128 * Self::PERCENTAGE_FACTOR;

    if subnet_mem_position <= x_curve_start {
      if simple_min_subnet_nodes  > min_subnet_nodes {
        min_subnet_nodes = simple_min_subnet_nodes;
      }
      return (min_subnet_nodes / Self::PERCENTAGE_FACTOR) as u32
    }

    let mut x = 0;

    if subnet_mem_position >= x_curve_start && subnet_mem_position <= Self::PERCENTAGE_FACTOR {
      // If subnet memory position is in between range
      x = (subnet_mem_position-x_curve_start) * Self::PERCENTAGE_FACTOR / (Self::PERCENTAGE_FACTOR-x_curve_start);
    } else if subnet_mem_position > Self::PERCENTAGE_FACTOR {
      // If subnet memory is greater than 100%
      x = Self::PERCENTAGE_FACTOR;
    }

    let y = (y_start - y_end) * (Self::PERCENTAGE_FACTOR - x) / Self::PERCENTAGE_FACTOR + y_end;

    let y_ratio = Self::percent_div(y, y_start);

    let mut min_subnet_nodes_on_curve = Self::percent_mul(simple_min_subnet_nodes, y_ratio);

    // --- If over the max x position, we increase the min nodes to ensure the amount is never less than the previous position
    if subnet_mem_position > max_x {
      let x_max_x_ratio = Self::percent_div(max_x, subnet_mem_position);
      let comp_fraction = Self::PERCENTAGE_FACTOR - x_max_x_ratio;
      min_subnet_nodes_on_curve = Self::percent_mul(
        simple_min_subnet_nodes, 
        comp_fraction
      ) + min_subnet_nodes_on_curve;
    }

    // Redundant
    if min_subnet_nodes_on_curve > min_subnet_nodes {
      min_subnet_nodes = min_subnet_nodes_on_curve;
    }

    (min_subnet_nodes / Self::PERCENTAGE_FACTOR) as u32
  }

  pub fn get_subnet_rewards_v2(
    base_node_memory: u128, 
    memory_mb: u128, 
    min_subnet_nodes: u32, 
    subnet_delegate_stake: u128
  ) {
    let min_subnet_delegate_stake_balance = Self::get_min_subnet_delegate_stake_balance(min_subnet_nodes);

    let dif = match subnet_delegate_stake > min_subnet_delegate_stake_balance {
      true => subnet_delegate_stake - min_subnet_delegate_stake_balance,
      false => 0,
    };

  }

  pub fn get_rewards_pool_inflation(subnets_count: u32) {

  }

  pub fn get_overall_subnet_reward() {

  }

  pub fn get_target_subnet_nodes(min_subnet_nodes: u32) -> u32 {
    Self::percent_mul(
      min_subnet_nodes.into(), 
      TargetSubnetNodesMultiplier::<T>::get()
    ) as u32 + min_subnet_nodes
  }

  pub fn get_subnet_rewards(memory_mb: u128, base_reward_per_mb: u128) -> u128 {
    Self::percent_mul(
      Self::percent_mul(base_reward_per_mb, memory_mb), 
      TargetSubnetNodesMultiplier::<T>::get()
    )
  }

  pub fn get_subnet_initialization_cost(block: u64) -> u128 {
    T::SubnetInitializationCost::get()
  }

  pub fn do_epoch_preliminaries(block: u64, epoch: u32, epoch_length: u64) {
    let max_subnet_penalty_count = MaxSubnetPenaltyCount::<T>::get();
    let subnet_activation_enactment_period = SubnetActivationEnactmentPeriod::<T>::get();

    for (subnet_id, data) in SubnetsData::<T>::iter() {
      // --- Ensure subnet is active is able to submit consensus
      let max_registration_block = data.initialized + data.registration_blocks + subnet_activation_enactment_period;
      if data.activated == 0 && block <= max_registration_block {
        // We check if the subnet is still in registration phase and not yet out of the enactment phase
        continue
      } else if data.activated == 0 && block > max_registration_block {
        // --- Ensure subnet is in registration period and hasn't passed enactment period
        // If subnet hasn't been activated after the enacement period, then remove subnet
				Self::deactivate_subnet(
					data.path,
					SubnetRemovalReason::EnactmentPeriod,
				);
        continue
			}

      // --- All subnets are now activated and passed the registration period
      // Must have:
      //  - Minimum nodes (increases penalties if less than)
      //  - Minimum delegate stake balance (remove subnet if less than)

      let min_subnet_nodes = data.min_nodes;
			let subnet_delegate_stake_balance = TotalSubnetDelegateStakeBalance::<T>::get(subnet_id);
			let min_subnet_delegate_stake_balance = Self::get_min_subnet_delegate_stake_balance(min_subnet_nodes);

      // --- Ensure min delegate stake balance is met
      if subnet_delegate_stake_balance < min_subnet_delegate_stake_balance {
        Self::deactivate_subnet(
          data.path,
          SubnetRemovalReason::MinSubnetDelegateStake,
        );
        continue
      }

      // --- Get all possible validators
      let subnet_node_accounts: Vec<T::AccountId> = Self::get_classified_accounts(subnet_id, &SubnetNodeClass::Validator, epoch as u64);
      let subnet_nodes_count = subnet_node_accounts.len();
      
      // --- Ensure min nodes are active
      // Only choose validator if min nodes are present
      // The ``SubnetPenaltyCount`` when surpassed doesn't penalize anyone, only removes the subnet from the chain
      if (subnet_nodes_count as u32) < min_subnet_nodes {
        SubnetPenaltyCount::<T>::mutate(subnet_id, |n: &mut u32| *n += 1);
      }

      // --- Check penalties and remove subnet is threshold is breached
      let penalties = SubnetPenaltyCount::<T>::get(subnet_id);
      if penalties >  max_subnet_penalty_count {
        Self::deactivate_subnet(
          data.path,
          SubnetRemovalReason::MaxPenalties,
        );
        continue
      }

      Self::choose_validator(
        block,
        subnet_id,
        subnet_node_accounts.clone(),
        min_subnet_nodes,
        epoch,
      );
    }
  }

  // pub fn validate_signature(
  //   data: &Vec<u8>,
  //   signature: &T::OffchainSignature,
  //   signer: &T::AccountId,
  // ) -> DispatchResult {
  //   if signature.verify(&**data, &signer) {
  //     return Ok(())
  //   }

  //   // NOTE: for security reasons modern UIs implicitly wrap the data requested to sign into
  //   // <Bytes></Bytes>, that's why we support both wrapped and raw versions.
  //   let prefix = b"<Bytes>";
  //   let suffix = b"</Bytes>";
  //   let mut wrapped: Vec<u8> = Vec::with_capacity(data.len() + prefix.len() + suffix.len());
  //   wrapped.extend(prefix);
  //   wrapped.extend(data);
  //   wrapped.extend(suffix);

  //   ensure!(signature.verify(&*wrapped, &signer), Error::<T>::WrongSignature);

  //   Ok(())
  // }
  
  /// The minimum delegate stake balance for a subnet to stay live
  pub fn get_min_subnet_delegate_stake_balance(min_subnet_nodes: u32) -> u128 {
    // --- Get minimum stake balance per subnet node
    let min_stake_balance = MinStakeBalance::<T>::get();
    // --- Get minimum subnet stake balance
    let min_subnet_stake_balance = min_stake_balance * min_subnet_nodes as u128;
    // --- Get required delegate stake balance for a subnet to have to stay live
    let min_subnet_delegate_stake_balance = Self::percent_mul(
      min_subnet_stake_balance, 
      MinSubnetDelegateStakePercentage::<T>::get()
    );
    // --- Get absolute minimum required subnet delegate stake balance
    let min_subnet_delegate_stake = MinSubnetDelegateStake::<T>::get();
    // --- Return here if the absolute minimum required subnet delegate stake balance is greater
    //     than the calculated minimum requirement
    if min_subnet_delegate_stake > min_subnet_delegate_stake_balance {
      return min_subnet_delegate_stake
    }
    min_subnet_delegate_stake_balance
  }

  /// Get subnet nodes by classification
  pub fn get_classified_subnet_nodes(subnet_id: u32, classification: &SubnetNodeClass, epoch: u64) -> Vec<SubnetNode<T::AccountId>> {
    SubnetNodesData::<T>::iter_prefix_values(subnet_id)
      .filter(|subnet_node| subnet_node.has_classification(classification, epoch))
      .collect()
  }

  pub fn get_classified_subnet_node_info(subnet_id: u32, classification: &SubnetNodeClass, epoch: u64) -> Vec<SubnetNodeInfo<T::AccountId>> {
    SubnetNodesData::<T>::iter_prefix_values(subnet_id)
      .filter(|subnet_node| subnet_node.has_classification(classification, epoch))
      .map(|subnet_node| {
        SubnetNodeInfo {
          coldkey: KeyOwner::<T>::get(subnet_node.hotkey.clone()),
          hotkey: subnet_node.hotkey,
          peer_id: subnet_node.peer_id,
        }
      })
      .collect()
  }

  // Get subnet node ``account_ids`` by classification
  pub fn get_classified_accounts<C>(
    subnet_id: u32,
    classification: &SubnetNodeClass,
    epoch: u64,
  ) -> C
  where
    C: FromIterator<T::AccountId>,
  {
    SubnetNodesData::<T>::iter_prefix_values(subnet_id)
      .filter(|subnet_node| subnet_node.has_classification(classification, epoch))
      .map(|subnet_node| subnet_node.hotkey)
      .collect()
  }
}
