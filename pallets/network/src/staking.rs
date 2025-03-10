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
use sp_runtime::Saturating;

impl<T: Config> Pallet<T> {
  pub fn do_add_stake(
    origin: T::RuntimeOrigin,
    subnet_id: u32,
    hotkey: T::AccountId,
    stake_to_be_added: u128,
  ) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    let stake_as_balance = Self::u128_to_balance(stake_to_be_added);

    ensure!(
      stake_as_balance.is_some(),
      Error::<T>::CouldNotConvertToBalance
    );

    let account_stake_balance: u128 = AccountSubnetStake::<T>::get(&hotkey, subnet_id);

    ensure!(
      account_stake_balance.saturating_add(stake_to_be_added) >= MinStakeBalance::<T>::get(),
      Error::<T>::MinStakeNotReached
    );

    ensure!(
      account_stake_balance.saturating_add(stake_to_be_added) <= MaxStakeBalance::<T>::get(),
      Error::<T>::MaxStakeReached
    );

    // --- Ensure the callers coldkey has enough stake to perform the transaction.
    ensure!(
      Self::can_remove_balance_from_coldkey_account(&coldkey, stake_as_balance.unwrap()),
      Error::<T>::NotEnoughBalanceToStake
    );
  
    // to-do: add AddStakeRateLimit instead of universal rate limiter
    //        this allows peers to come in freely
    let block: u64 = Self::get_current_block_as_u64();
    ensure!(
      !Self::exceeds_tx_rate_limit(Self::get_last_tx_block(&coldkey), block),
      Error::<T>::TxRateLimitExceeded
    );

    // --- Ensure the remove operation from the coldkey is a success.
    ensure!(
      Self::remove_balance_from_coldkey_account(&coldkey, stake_as_balance.unwrap()) == true,
      Error::<T>::BalanceWithdrawalError
    );
  
    Self::increase_account_stake(
      &hotkey,
      subnet_id, 
      stake_to_be_added,
    );

    // Set last block for rate limiting
    Self::set_last_tx_block(&coldkey, block);

    Self::deposit_event(Event::StakeAdded(subnet_id, coldkey, stake_to_be_added));

    Ok(())
  }

  pub fn do_remove_stake(
    origin: T::RuntimeOrigin, 
    subnet_id: u32,
    hotkey: T::AccountId,
    is_subnet_node: bool,
    stake_to_be_removed: u128,
  ) -> DispatchResult {
    let coldkey: T::AccountId = ensure_signed(origin)?;

    // --- Ensure that the stake amount to be removed is above zero.
    ensure!(
      stake_to_be_removed > 0,
      Error::<T>::NotEnoughStakeToWithdraw
    );

    let account_stake_balance: u128 = AccountSubnetStake::<T>::get(&hotkey, subnet_id);

    // --- Ensure that the account has enough stake to withdraw.
    ensure!(
      account_stake_balance >= stake_to_be_removed,
      Error::<T>::NotEnoughStakeToWithdraw
    );
    
    // if user is still a subnet node they must keep the required minimum balance
    if is_subnet_node {
      ensure!(
        account_stake_balance.saturating_sub(stake_to_be_removed) >= MinStakeBalance::<T>::get(),
        Error::<T>::MinStakeNotReached
      );  
    }
  
    // --- Ensure that we can convert this u128 to a balance.
    let stake_to_be_removed_as_currency = Self::u128_to_balance(stake_to_be_removed);
    ensure!(
      stake_to_be_removed_as_currency.is_some(),
        Error::<T>::CouldNotConvertToBalance
    );

    let block: u64 = Self::get_current_block_as_u64();
    ensure!(
      !Self::exceeds_tx_rate_limit(Self::get_last_tx_block(&coldkey), block),
      Error::<T>::TxRateLimitExceeded
    );

    // --- 7. We remove the balance from the hotkey.
    Self::decrease_account_stake(&hotkey, subnet_id, stake_to_be_removed);

    // --- 9. We add the balancer to the coldkey.  If the above fails we will not credit this coldkey.
    Self::add_balance_to_stake_unbonding_ledger(&coldkey, subnet_id, stake_to_be_removed, block).map_err(|e| e)?;

    // Set last block for rate limiting
    Self::set_last_tx_block(&coldkey, block);

    Self::deposit_event(Event::StakeRemoved(subnet_id, coldkey, stake_to_be_removed));

    Ok(())
  }

  pub fn do_swap_hotkey_balance(
    origin: T::RuntimeOrigin, 
    subnet_id: u32,
    old_hotkey: &T::AccountId,
    new_hotkey: &T::AccountId,
  ) {
    Self::swap_account_stake(
      old_hotkey,
      new_hotkey,
      subnet_id, 
    )
  }

  pub fn add_balance_to_stake_unbonding_ledger(
    coldkey: &T::AccountId,
    subnet_id: u32, 
    amount: u128,
    block: u64,
  ) -> DispatchResult {
    let epoch_length: u64 = T::EpochLength::get();
    let epoch: u64 = block / epoch_length;

    let unbondings = SubnetStakeUnbondingLedger::<T>::get(coldkey.clone(), subnet_id);

    // One unlocking per epoch
    ensure!(
      unbondings.get(&epoch) == None,
      Error::<T>::MaxUnlockingsPerEpochReached
    );

    // --- Ensure we don't surpass max unlockings by attempting to unlock unbondings
    if unbondings.len() as u32 == T::MaxStakeUnlockings::get() {
      Self::do_claim_stake_unbondings(&coldkey, subnet_id);
    }

    // --- Get updated unbondings after claiming unbondings
    let mut unbondings = SubnetStakeUnbondingLedger::<T>::get(coldkey.clone(), subnet_id);

    // We're about to add another unbonding to the ledger - it must be n-1
    ensure!(
      unbondings.len() < T::MaxStakeUnlockings::get() as usize,
      Error::<T>::MaxUnlockingsReached
    );

    unbondings.insert(epoch, amount);
    SubnetStakeUnbondingLedger::<T>::insert(coldkey.clone(), subnet_id, unbondings);

    Ok(())
  }

  // Infallible
  pub fn do_claim_stake_unbondings(coldkey: &T::AccountId, subnet_id: u32) -> u32 {
    let block: u64 = Self::get_current_block_as_u64();
    let epoch_length: u64 = T::EpochLength::get();
    let epoch: u64 = block / epoch_length;
    let unbondings = SubnetStakeUnbondingLedger::<T>::get(coldkey.clone(), subnet_id);
    let mut unbondings_copy = unbondings.clone();

    let mut successful_unbondings = 0;

    for (unbonding_epoch, amount) in unbondings.iter() {
      if epoch <= unbonding_epoch + T::StakeCooldownEpochs::get() {
        continue
      }

      let stake_to_be_added_as_currency = Self::u128_to_balance(*amount);
      if !stake_to_be_added_as_currency.is_some() {
        // Redundant
        unbondings_copy.remove(&unbonding_epoch);
        continue
      }
      
      unbondings_copy.remove(&unbonding_epoch);
      Self::add_balance_to_coldkey_account(&coldkey, stake_to_be_added_as_currency.unwrap());
      successful_unbondings += 1;
    }

    if unbondings.len() != unbondings_copy.len() {
      SubnetStakeUnbondingLedger::<T>::insert(coldkey.clone(), subnet_id, unbondings_copy);
    }
    successful_unbondings
  }

  pub fn increase_account_stake(
    hotkey: &T::AccountId,
    subnet_id: u32, 
    amount: u128,
  ) {
    // -- increase account subnet staking balance
    AccountSubnetStake::<T>::mutate(hotkey, subnet_id, |mut n| n.saturating_accrue(amount));

    // -- increase total subnet stake
    TotalSubnetStake::<T>::mutate(subnet_id, |mut n| n.saturating_accrue(amount));

    // -- increase total stake overall
    TotalStake::<T>::mutate(|mut n| n.saturating_accrue(amount));
  }
  
  pub fn decrease_account_stake(
    hotkey: &T::AccountId,
    subnet_id: u32, 
    amount: u128,
  ) {
    // -- decrease account subnet staking balance
    AccountSubnetStake::<T>::mutate(hotkey, subnet_id, |mut n| n.saturating_reduce(amount));

    // -- decrease total subnet stake
    TotalSubnetStake::<T>::mutate(subnet_id, |mut n| n.saturating_reduce(amount));

    // -- decrease total stake overall
    TotalStake::<T>::mutate(|mut n| n.saturating_reduce(amount));
  }

  fn swap_account_stake(
    old_hotkey: &T::AccountId,
    new_hotkey: &T::AccountId,
    subnet_id: u32, 
  ) {
    // -- swap old_hotkey subnet staking balance
    let old_hotkey_stake_balance = AccountSubnetStake::<T>::take(old_hotkey, subnet_id);
    // --- Redundant take of new hotkeys stake balance
    // --- New hotkey is always checked before updating
    let new_hotkey_stake_balance = AccountSubnetStake::<T>::take(new_hotkey, subnet_id);
    AccountSubnetStake::<T>::insert(
      new_hotkey, 
      subnet_id, 
      old_hotkey_stake_balance.saturating_add(new_hotkey_stake_balance)
    );
  }

  pub fn can_remove_balance_from_coldkey_account(
    coldkey: &T::AccountId,
    amount: <<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance,
  ) -> bool {
    let current_balance = Self::get_coldkey_balance(coldkey);
    if amount > current_balance {
      return false;
    }

    // This bit is currently untested. @todo
    let new_potential_balance = current_balance - amount;
    let can_withdraw = T::Currency::ensure_can_withdraw(
      &coldkey,
      amount,
      WithdrawReasons::except(WithdrawReasons::TIP),
      new_potential_balance,
    )
    .is_ok();
    can_withdraw
  }

  pub fn remove_balance_from_coldkey_account(
    coldkey: &T::AccountId,
    amount: <<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance,
  ) -> bool {
    return match T::Currency::withdraw(
      &coldkey,
      amount,
      WithdrawReasons::except(WithdrawReasons::TIP),
      ExistenceRequirement::KeepAlive,
    ) {
      Ok(_result) => true,
      Err(_error) => false,
    };
  }

  pub fn add_balance_to_coldkey_account(
    coldkey: &T::AccountId,
    amount: <<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance,
  ) {
    T::Currency::deposit_creating(&coldkey, amount);
  }

  pub fn get_coldkey_balance(
    coldkey: &T::AccountId,
  ) -> <<T as pallet::Config>::Currency as Currency<<T as system::Config>::AccountId>>::Balance {
    return T::Currency::free_balance(&coldkey);
  }

  // pub fn u64_to_balance(
  //   input: u64,
  // ) -> Option<
  //   <<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance,
  // > {
  //   input.try_into().ok()
  // }

  pub fn u128_to_balance(
    input: u128,
  ) -> Option<
    <<T as pallet::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance,
  > {
    input.try_into().ok()
  }
}