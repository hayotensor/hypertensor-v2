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

impl<T: Config> Pallet<T> {
  pub fn do_owner_remove_subnet_node(origin: T::RuntimeOrigin, subnet_id: u32, subnet_node_id: u32) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );


    Ok(())
  }

  pub fn do_update_entry_interval(origin: T::RuntimeOrigin, subnet_id: u32, value: u32) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    ensure!(
      Self::is_subnet_owner(&coldkey, subnet_id),
      Error::<T>::NotSubnetOwner
    );

    SubnetEntryInterval::<T>::insert(subnet_id, value);

    Self::deposit_event(Event::SubnetEntryIntervalUpdate { 
      subnet_id: subnet_id,
      owner: coldkey, 
      value: value 
    });

    Ok(())
  }
}