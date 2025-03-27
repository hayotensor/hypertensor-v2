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
//
// Enables accounts to delegate stake to subnets for a portion of emissions

use super::*;
use sp_runtime::Saturating;

impl<T: Config> Pallet<T> {
  pub fn do_transfer_from_node_to_subnet(
    origin: T::RuntimeOrigin,
    from_subnet_id: u32,
    from_subnet_node_id: u32,
    to_subnet_id: u32,
    node_delegate_stake_shares_to_be_switched: u128,
  ) -> DispatchResult {
    let account_id: T::AccountId = ensure_signed(origin)?;

    let (result, balance_removed, _) = Self::perform_do_remove_node_delegate_stake(
      &account_id,
      from_subnet_id,
      from_subnet_node_id,
      node_delegate_stake_shares_to_be_switched,
      false,
    );

    result?;

    let (result, _, _) = Self::perform_do_add_delegate_stake(
      &account_id,
      to_subnet_id,
      balance_removed,
      true
    );

    result?;

    Self::deposit_event(Event::DelegateNodeToSubnetDelegateStakeSwitched { 
			account_id: account_id, 
			from_subnet_id: from_subnet_id, 
			from_subnet_node_id: from_subnet_node_id, 
			to_subnet_id: to_subnet_id, 
			amount: balance_removed 
    });

    Ok(())
  }

  pub fn do_transfer_from_subnet_to_node(
    origin: T::RuntimeOrigin,
    from_subnet_id: u32,
    to_subnet_id: u32,
    to_subnet_node_id: u32,
    delegate_stake_shares_to_be_switched: u128,
  ) -> DispatchResult {
    let account_id: T::AccountId = ensure_signed(origin)?;

    let (result, balance_removed, _) = Self::perform_do_remove_delegate_stake(
      &account_id, 
      from_subnet_id,
      delegate_stake_shares_to_be_switched,
      false,
    );

    result?;

    let (result, _, _) = Self::perform_do_add_node_delegate_stake(
      &account_id,
      to_subnet_id,
      to_subnet_node_id,
      balance_removed,
      true
    );

    result?;

    Self::deposit_event(Event::SubnetDelegateToNodeDelegateStakeSwitched { 
			account_id: account_id, 
			from_subnet_id: from_subnet_id, 
			to_subnet_id: to_subnet_id, 
      to_subnet_node_id: to_subnet_node_id, 
			amount: balance_removed 
    });

    Ok(())
  }
}