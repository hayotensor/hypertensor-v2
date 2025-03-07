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
  
  pub fn get_current_epoch_as_u32() -> u32 {
    let current_block = Self::get_current_block_as_u64();
    let epoch_length: u64 = T::EpochLength::get();
    (current_block / epoch_length) as u32
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

  /// Remove subnet peer from subnet
  // to-do: Add slashing to subnet peers stake balance
  pub fn perform_remove_subnet_node(block: u64, subnet_id: u32, subnet_node_id: u32) {
    if let Ok(subnet_node) = SubnetNodesData::<T>::try_get(subnet_id, subnet_node_id) {
      let hotkey = subnet_node.hotkey;
      let peer_id = subnet_node.peer_id;

      // Remove from attestations
      let epoch_length: u64 = T::EpochLength::get();
			let epoch: u64 = block / epoch_length;

      let submittable_nodes: BTreeSet<T::AccountId> = Self::get_classified_hotkeys(subnet_id, &SubnetNodeClass::Validator, epoch);

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
            if attests.remove(&subnet_node_id).is_some() {
              params.attests = attests.clone();
            }
          };
          Ok(())
        }
      );
    
      let subnet_node = SubnetNodesData::<T>::take(subnet_id, subnet_node_id);

      if subnet_node.a.is_some() {
        SubnetNodeUniqueParam::<T>::remove(subnet_id, subnet_node.a.unwrap())
      }

      // Remove all subnet node elements
      SubnetNodeAccount::<T>::remove(subnet_id, peer_id.clone());
      HotkeySubnetNodeId::<T>::remove(subnet_id, hotkey.clone());
      SubnetNodeIdHotkey::<T>::remove(subnet_id, subnet_node_id);

      // Update total subnet peers by substracting 1
      TotalSubnetNodes::<T>::mutate(subnet_id, |n: &mut u32| n.saturating_dec());

      // Reset sequential absent subnet node count
      SubnetNodePenalties::<T>::remove(subnet_id, subnet_node_id);

			Self::deposit_event(Event::SubnetNodeRemoved { subnet_id: subnet_id, subnet_node_id: subnet_node_id });
    }
  }

  pub fn get_min_subnet_nodes(base_node_memory: u128, memory_mb: u128) -> u32 {
    // TODO: Needs to be updated for smoother curve
    //
    //
    //
    //

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

  pub fn get_subnet_initialization_cost(block: u64) -> u128 {
    T::SubnetInitializationCost::get()
  }

  pub fn registration_cost(epoch: u32) -> u128 {
    let period: u32 = SubnetRegistrationFeePeriod::<T>::get();
    let last_registration_epoch = LastSubnetRegistrationEpoch::<T>::get();
    let next_registration_epoch = Self::get_next_registration_epoch(last_registration_epoch);
    let fee_min: u128 = MinSubnetRegistrationFee::<T>::get();

    // If no registration within period, keep at `fee_min`
    if epoch >= next_registration_epoch + period {
      return fee_min
    }

    let fee_max: u128 = MaxSubnetRegistrationFee::<T>::get();

    // Epoch within the cycle
    let cycle_epoch = epoch % period;
    let decrease_per_epoch = (fee_max.saturating_sub(fee_min)).saturating_div(period as u128);
    
    let cost = fee_max.saturating_sub(decrease_per_epoch.saturating_mul(cycle_epoch as u128));
    // Ensures cost doesn't go below min
    cost.max(fee_min)
  }

  pub fn can_subnet_register(current_epoch: u32) -> bool {
    current_epoch >= Self::get_next_registration_epoch(current_epoch)
  }

  pub fn get_next_registration_epoch(current_epoch: u32) -> u32 {
    let last_registration_epoch: u32 = LastSubnetRegistrationEpoch::<T>::get();
    let subnet_registration_fee_period: u32 = SubnetRegistrationFeePeriod::<T>::get();
    // // --- Genesis handling
    // if last_registration_epoch < subnet_registration_fee_period {
    //   return 0
    // }
    let next_valid_epoch = last_registration_epoch + (
      subnet_registration_fee_period - (last_registration_epoch % subnet_registration_fee_period)
    );
    next_valid_epoch
  }

  pub fn do_epoch_preliminaries(block: u64, epoch: u32, epoch_length: u64) {
    let max_subnet_penalty_count = MaxSubnetPenaltyCount::<T>::get();
    let subnet_activation_enactment_period = SubnetActivationEnactmentPeriod::<T>::get();

    let subnets: Vec<_> = SubnetsData::<T>::iter().collect();
    let total_subnets: u32 = subnets.len() as u32;
    let excess_subnets: bool = total_subnets > MaxSubnets::<T>::get();
    let mut subnet_delegate_stake: Vec<(Vec<u8>, u128)> = Vec::new();

    for (subnet_id, data) in subnets {
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
      let subnet_node_ids: Vec<u32> = Self::get_classified_subnet_node_ids(subnet_id, &SubnetNodeClass::Validator, epoch as u64);
      let subnet_nodes_count = subnet_node_ids.len();
      
      // --- Ensure min nodes are active
      // Only choose validator if min nodes are present
      // The ``SubnetPenaltyCount`` when surpassed doesn't penalize anyone, only removes the subnet from the chain
      if (subnet_nodes_count as u32) < min_subnet_nodes {
        SubnetPenaltyCount::<T>::mutate(subnet_id, |n: &mut u32| *n += 1);
      }

      // --- Check penalties and remove subnet is threshold is breached
      let penalties = SubnetPenaltyCount::<T>::get(subnet_id);
      if penalties > max_subnet_penalty_count {
        Self::deactivate_subnet(
          data.path,
          SubnetRemovalReason::MaxPenalties,
        );
        continue
      }

      if excess_subnets {
        subnet_delegate_stake.push((data.path, subnet_delegate_stake_balance));
      }

      Self::choose_validator(
        block,
        subnet_id,
        subnet_node_ids.clone(),
        min_subnet_nodes,
        epoch,
      );
    }

    if excess_subnets {
      subnet_delegate_stake.sort_by_key(|&(_, value)| value);
      Self::deactivate_subnet(
        subnet_delegate_stake[0].0.clone(),
        SubnetRemovalReason::MaxSubnets,
      );
    }
  }
  
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

  pub fn get_classified_subnet_node_ids<C>(
    subnet_id: u32,
    classification: &SubnetNodeClass,
    epoch: u64,
  ) -> C
    where
      C: FromIterator<u32>,
  {
    SubnetNodesData::<T>::iter_prefix(subnet_id)
      .filter(|(_, subnet_node)| subnet_node.has_classification(classification, epoch))
      .map(|(subnet_node_id, _)| subnet_node_id)
      .collect()
  }

  
  /// Get subnet nodes by classification
  pub fn get_classified_subnet_nodes(subnet_id: u32, classification: &SubnetNodeClass, epoch: u64) -> Vec<SubnetNode<T::AccountId>> {
    SubnetNodesData::<T>::iter_prefix_values(subnet_id)
      .filter(|subnet_node| subnet_node.has_classification(classification, epoch))
      .collect()
  }

  pub fn get_classified_subnet_node_info(subnet_id: u32, classification: &SubnetNodeClass, epoch: u64) -> Vec<SubnetNodeInfo<T::AccountId>> {
    SubnetNodesData::<T>::iter_prefix(subnet_id)
      .filter(|(subnet_node_id, subnet_node)| subnet_node.has_classification(classification, epoch))
      .map(|(subnet_node_id, subnet_node)| {
        SubnetNodeInfo {
          subnet_node_id: subnet_node_id,
          coldkey: HotkeyOwner::<T>::get(subnet_node.hotkey.clone()),
          hotkey: subnet_node.hotkey,
          peer_id: subnet_node.peer_id,
          classification: subnet_node.classification,
          a: subnet_node.a,
          b: subnet_node.b,
          c: subnet_node.c,
        }
      })
      .collect()
  }

  // Get subnet node ``hotkeys`` by classification
  pub fn get_classified_hotkeys<C>(
    subnet_id: u32,
    classification: &SubnetNodeClass,
    epoch: u64,
  ) -> C
    where
      C: FromIterator<T::AccountId>,
  {
    SubnetNodesData::<T>::iter_prefix(subnet_id)
      .filter(|(_, subnet_node)| subnet_node.has_classification(classification, epoch))
      .map(|(_, subnet_node)| subnet_node.hotkey)
      .collect()
  }

  pub fn is_subnet_node_owner(subnet_id: u32, subnet_node_id: u32, hotkey: T::AccountId) -> bool {
    match SubnetNodesData::<T>::try_get(subnet_id, subnet_node_id) {
      Ok(data) => {
        data.hotkey == hotkey
      },
      Err(()) => false,
    }
  }

  /// Is hotkey or coldkey owner for functions that allow both
  pub fn get_hotkey_coldkey(
    subnet_id: u32, 
    subnet_node_id: u32, 
  ) -> Option<(T::AccountId, T::AccountId)> {
    let hotkey = SubnetNodeIdHotkey::<T>::try_get(subnet_id, subnet_node_id).ok()?;
    let coldkey = HotkeyOwner::<T>::try_get(hotkey.clone()).ok()?;

    Some((hotkey, coldkey))
  }

  pub fn is_keys_owner(
    subnet_id: u32, 
    subnet_node_id: u32, 
    key: T::AccountId, 
  ) -> bool {
    let (hotkey, coldkey) = match Self::get_hotkey_coldkey(subnet_id, subnet_node_id) {
      Some((hotkey, coldkey)) => {
        (hotkey, coldkey)
      }
      None => {
        return false
      }
    };

    key == hotkey || key == coldkey
  }

  pub fn is_subnet_node_coldkey(
    subnet_id: u32, 
    subnet_node_id: u32, 
    coldkey: T::AccountId, 
  ) -> bool {
    let hotkey = match SubnetNodeIdHotkey::<T>::try_get(subnet_id, subnet_node_id) {
      Ok(hotkey) => hotkey,
      Err(()) => return false
    };
    match HotkeyOwner::<T>::try_get(hotkey) {
      Ok(subnet_node_coldkey) => return subnet_node_coldkey == coldkey,
      Err(()) => return false
    }
  }

}
