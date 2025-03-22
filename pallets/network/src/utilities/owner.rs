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
  pub fn do_update_entry_interval(subnet_id: u32, value: u32) -> u32 {
    SubnetEntryInterval::<T>::insert(subnet_id, value);

    Self::deposit_event(Event::SubnetEntryIntervalUpdate { 
      subnet_id: subnet_id 
      owner: owner, 
      value: value 
    });
  }
}