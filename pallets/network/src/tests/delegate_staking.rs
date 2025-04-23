use super::mock::*;
use crate::tests::test_utils::*;
use crate::Event;
use frame_support::{
	assert_noop, assert_ok, assert_err
};
use log::info;
use frame_support::traits::{OnInitialize, Currency};
use sp_std::collections::{btree_map::BTreeMap, btree_set::BTreeSet};
use frame_support::BoundedVec;
use sp_core::OpaquePeerId as PeerId;
use crate::{
  Error,
  SubnetPaths, 
  TotalSubnetNodes,
  AccountSubnetDelegateStakeShares, 
  TotalSubnetDelegateStakeShares, 
  TotalSubnetDelegateStakeBalance,
  SubnetRemovalReason,
  StakeUnbondingLedger,
  AccountNodeDelegateStakeShares,
  TotalNodeDelegateStakeBalance,
  TotalNodeDelegateStakeShares,
  MinStakeBalance,
};
use codec::Decode;
use sp_runtime::traits::TrailingZeroInput;
use sp_core::U256;

//
//
//
//
//
//
//
// Delegate staking
//
//
//
//
//
//
//

#[test]
fn test_delegate_math() {
  new_test_ext().execute_with(|| {
    let _ = env_logger::builder().is_test(true).try_init();

    let subnet_id = 0;
    let account_id = account(0);
    let delegate_stake_to_be_added = 1000e+18 as u128;

    let account_delegate_stake_shares: u128 = AccountSubnetDelegateStakeShares::<Test>::get(&account_id, subnet_id);
    // let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_shares = match TotalSubnetDelegateStakeShares::<Test>::get(subnet_id) {
      0 => {
        // --- Mitigate inflation attack
        // Network::increase_account_delegate_stake_shares(
        //   &<Test as frame_system::Config>::AccountId::decode(&mut TrailingZeroInput::zeroes()).unwrap(),
        //   subnet_id, 
        //   0,
        //   1000,
        // );
        TotalSubnetDelegateStakeShares::<Test>::mutate(subnet_id, |mut n| *n += 10000);
        0
      },
      shares => shares,
    };
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
    log::error!("total_subnet_delegated_stake_shares   {:?}", total_subnet_delegated_stake_shares);
    log::error!("total_subnet_delegated_stake_balance  {:?}", total_subnet_delegated_stake_balance);

    let mut delegate_stake_to_be_added_as_shares = Network::convert_to_shares(
      delegate_stake_to_be_added,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );
    log::error!("delegate_stake_to_be_added_as_shares  {:?}", delegate_stake_to_be_added_as_shares);

    Network::increase_account_delegate_stake_shares(
      &account_id,
      subnet_id, 
      delegate_stake_to_be_added,
      delegate_stake_to_be_added_as_shares,
    );

    let account_delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(&account_id, subnet_id);
    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
    
    log::error!(" ");
    log::error!("account_delegate_shares               {:?}", account_delegate_shares);
    log::error!("total_subnet_delegated_stake_shares   {:?}", total_subnet_delegated_stake_shares);
    log::error!("total_subnet_delegated_stake_balance  {:?}", total_subnet_delegated_stake_balance);

    let delegate_balance = Network::convert_to_balance(
      account_delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );
    
    log::error!("delegate_balance                      {:?}", delegate_balance);

    // Ensure balance is within <= 0.01% of deposited balance, and less than deposited balance
    assert!(
      (delegate_balance >= Network::percent_mul(delegate_stake_to_be_added, 990000000)) &&
      (delegate_balance < delegate_stake_to_be_added)
    );

    let delegate_balance2 = Network::convert_to_balance(
      account_delegate_shares,
      total_subnet_delegated_stake_shares + 9000,
      total_subnet_delegated_stake_balance
    );
    log::error!("delegate_balance2                     {:?}", delegate_balance2);
  });
}

#[test]
fn check_balances() {
  new_test_ext().execute_with(|| {
    let _ = env_logger::builder().is_test(true).try_init();
    
    let subnet_id = 1;
    let user = account(1);

    // Initial user tokens
    // const USER_INITIAL_TOKENS: u128 = 1000000000000000000; // 1
    // const USER_INITIAL_TOKENS: u128 = 10000000000000000000; // 10
    // const USER_INITIAL_TOKENS: u128 = 100000000000000000000; // 100

    // const USER_INITIAL_BALANCE: u128 = USER_INITIAL_TOKENS + 500;

    // Balances::make_free_balance_be(&user, USER_INITIAL_BALANCE);

    // // ---- Step 1: uSER deposits minimal amount ----
    // // The MinDelegateStakeBalance (deposit min) is 1000, otherwise reverts with CouldNotConvertToBalance
    // assert_ok!(
    //   Network::do_add_delegate_stake(
    //     RuntimeOrigin::signed(user.clone()),
    //     subnet_id,
    //     USER_INITIAL_TOKENS,
    //   )
    // );

    // let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    // let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

    // // Validate initial deposit
    // let user_balance = Network::convert_to_balance(
    //   AccountSubnetDelegateStakeShares::<Test>::get(&user, subnet_id),
    //   total_subnet_delegated_stake_shares,
    //   total_subnet_delegated_stake_balance
    // );
    // log::error!("USER_INITIAL_TOKENS  {:?}", USER_INITIAL_TOKENS);
    // log::error!("user_balance         {:?}", user_balance);
    // // assert!(false);

    // log::error!("10_u128.pow(1)         {:?}", 10_u128.pow(1));

    // let loss = 1.0 - user_balance as f64 / USER_INITIAL_TOKENS as f64;
    // log::error!("loss         {:?}", loss);


    for n in 3..28 {
      // reset everything
      let _ = AccountSubnetDelegateStakeShares::<Test>::remove(user.clone(), subnet_id);			
      let _ = TotalSubnetDelegateStakeShares::<Test>::remove(subnet_id);
      let _ = TotalSubnetDelegateStakeBalance::<Test>::remove(subnet_id);

      let USER_INITIAL_TOKENS: u128 = 10_u128.pow(n);
      let USER_INITIAL_BALANCE: u128 = USER_INITIAL_TOKENS + 500;
      Balances::make_free_balance_be(&user, USER_INITIAL_BALANCE);

      assert_ok!(
        Network::do_add_delegate_stake(
          RuntimeOrigin::signed(user.clone()),
          subnet_id,
          USER_INITIAL_TOKENS,
        )
      );
  
      let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
      let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
  
      // Validate initial deposit
      let user_balance = Network::convert_to_balance(
        AccountSubnetDelegateStakeShares::<Test>::get(&user, subnet_id),
        total_subnet_delegated_stake_shares,
        total_subnet_delegated_stake_balance
      );
      log::error!("USER_INITIAL_TOKENS  {:?}", USER_INITIAL_TOKENS);
      log::error!("user_balance         {:?}", user_balance);
      let loss = 1.0 - user_balance as f64 / USER_INITIAL_TOKENS as f64;
      log::error!("Initial Deposit   {}", USER_INITIAL_TOKENS as f64 / 1e+18 as f64);
      log::error!("Resulting Balance {}", user_balance as f64 / 1e+18 as f64);
      log::error!("Loss              {}", loss);

      log::error!(" ");


    }
    // assert!(false);

  });
}

#[test]
fn test_delegate_math_with_storage_deposit() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 1000000000000000000000000;
    let amount: u128 = 1000000000000000000000; // 1000
    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), amount + 500);
    let starting_delegator_balance = Balances::free_balance(&account(total_subnet_nodes+1));

    assert_ok!(
      Network::add_to_delegate_stake(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        amount,
      ) 
    );

    // ensure removes wallet balance
    let post_delegator_balance = Balances::free_balance(&account(total_subnet_nodes+1));
    assert_eq!(post_delegator_balance, starting_delegator_balance - amount);

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
    let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(total_subnet_nodes+1), subnet_id);
    let delegate_balance = Network::convert_to_balance(
      delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );

    // Ensure balance is within <= 0.01% of deposited balance, and less than deposited balance
    assert!(
      (delegate_balance >= Network::percent_mul(amount, 990000000)) &&
      (delegate_balance < amount)
    );

    let pre_balance = Balances::free_balance(&account(total_subnet_nodes+1));

    let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(total_subnet_nodes+1), subnet_id);
    let shares_to_remove = delegate_shares / 2;
    let expected_ledger_balance = Network::convert_to_balance(
      shares_to_remove,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );

    let epoch = System::block_number() / EpochLength::get();

    assert_ok!(
      Network::remove_delegate_stake(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        shares_to_remove,
      )
    );

    let post_balance = Balances::free_balance(&account(total_subnet_nodes+1));
    assert_eq!(pre_balance, post_balance);

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
    let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(total_subnet_nodes+1), subnet_id);
    let delegate_balance = Network::convert_to_balance(
      delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );

    let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(total_subnet_nodes+1));
    assert_eq!(unbondings.len(), 1);
    let (ledger_epoch, ledger_balance) = unbondings.iter().next().unwrap();
    assert_eq!(*ledger_epoch, &epoch + DelegateStakeCooldownEpochs::get());
    assert_eq!(*ledger_balance, expected_ledger_balance);
  });
}

#[test]
fn test_remove_claim_delegate_stake_after_remove_subnet() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 1000000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), amount + 500);
    let starting_delegator_balance = Balances::free_balance(&account(total_subnet_nodes+1));

    assert_ok!(
      Network::add_to_delegate_stake(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        amount,
      ) 
    );

    let post_delegator_balance = Balances::free_balance(&account(total_subnet_nodes+1));
    assert_eq!(post_delegator_balance, starting_delegator_balance - amount);

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
    let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(total_subnet_nodes+1), subnet_id);
    let delegate_balance = Network::convert_to_balance(
      delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );
    let expected_ledger_balance = Network::convert_to_balance(
      delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );
    // assert_eq!(amount, delegate_balance);
    assert!(
      (delegate_balance >= Network::percent_mul(amount, 990000000)) &&
      (delegate_balance < amount)
    );

    assert_ok!(
      Network::do_remove_subnet(
        subnet_path.clone().into(),
        SubnetRemovalReason::MinSubnetDelegateStake,
      )
    );

    let epoch = System::block_number() / EpochLength::get();

    assert_ok!(
      Network::remove_delegate_stake(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        delegate_shares,
      )
    );

    let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(total_subnet_nodes+1));
    assert_eq!(unbondings.len(), 1);
    let (ledger_epoch, ledger_balance) = unbondings.iter().next().unwrap();
    assert_eq!(*ledger_epoch, &epoch + DelegateStakeCooldownEpochs::get());
    assert_eq!(*ledger_balance, expected_ledger_balance);

    System::set_block_number(System::block_number() + ((EpochLength::get()  + 1) * DelegateStakeCooldownEpochs::get()));

    assert_ok!(
      Network::claim_unbondings(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
      )
    );

    let post_balance = Balances::free_balance(&account(total_subnet_nodes+1));

    assert!(
      (post_balance >= Network::percent_mul(starting_delegator_balance, 990000000)) &&
      (post_balance < starting_delegator_balance)
    );

    let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(total_subnet_nodes+1));
    assert_eq!(unbondings.len(), 0);
  });
}

#[test]
fn test_add_to_delegate_stake() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let n_account = total_subnet_nodes + 1;

    let _ = Balances::deposit_creating(&account(n_account), amount+500);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

    let mut delegate_stake_to_be_added_as_shares = Network::convert_to_shares(
      amount,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );

    if total_subnet_delegated_stake_shares == 0 {
      delegate_stake_to_be_added_as_shares = delegate_stake_to_be_added_as_shares.saturating_sub(1000);
    }

    System::set_block_number(System::block_number() + DelegateStakeCooldownEpochs::get() * EpochLength::get());

    let starting_total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

    let starting_delegator_balance = Balances::free_balance(&account(n_account));

    assert_ok!(
      Network::add_to_delegate_stake(
        RuntimeOrigin::signed(account(n_account)),
        subnet_id,
        amount,
      ) 
    );

    let post_delegator_balance = Balances::free_balance(&account(n_account));
    assert_eq!(post_delegator_balance, starting_delegator_balance - amount);

    let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(n_account), subnet_id);
    assert_eq!(delegate_shares, delegate_stake_to_be_added_as_shares);
    assert_ne!(delegate_shares, 0);

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

    assert_eq!(amount + starting_total_subnet_delegated_stake_balance, total_subnet_delegated_stake_balance);

    let delegate_balance = Network::convert_to_balance(
      delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );
    // The first depositor will lose a percentage of their deposit depending on the size
    // https://docs.openzeppelin.com/contracts/4.x/erc4626#inflation-attack
    // assert_eq!(delegate_balance, delegate_stake_to_be_added_as_shares);

    assert!(
      (delegate_balance >= Network::percent_mul(amount, 990000000)) &&
      (delegate_balance < amount)
    );
  });
}

#[test]
fn test_add_to_delegate_stake_increase_pool_check_balance() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let n_account = total_subnet_nodes + 1;

    let _ = Balances::deposit_creating(&account(n_account), amount+500);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

    let mut delegate_stake_to_be_added_as_shares = Network::convert_to_shares(
      amount,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );

    if total_subnet_delegated_stake_shares == 0 {
      delegate_stake_to_be_added_as_shares = delegate_stake_to_be_added_as_shares.saturating_sub(1000);
    }

    System::set_block_number(System::block_number() + DelegateStakeCooldownEpochs::get() * EpochLength::get());

    assert_ok!(
      Network::add_to_delegate_stake(
        RuntimeOrigin::signed(account(n_account)),
        subnet_id,
        amount,
      ) 
    );

    let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(n_account), subnet_id);
    assert_eq!(delegate_shares, delegate_stake_to_be_added_as_shares);
    assert_ne!(delegate_shares, 0);

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
    log::error!("total_subnet_delegated_stake_shares  {:?}", total_subnet_delegated_stake_shares);
    log::error!("total_subnet_delegated_stake_balance {:?}", total_subnet_delegated_stake_balance);

    let delegate_balance = Network::convert_to_balance(
      delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );
    log::error!("delegate_balance                     {:?}", delegate_balance);

    // The first depositor will lose a percentage of their deposit depending on the size
    // https://docs.openzeppelin.com/contracts/4.x/erc4626#inflation-attack
    // assert_eq!(delegate_balance, delegate_stake_to_be_added_as_shares);
    assert!(
      (delegate_balance >= Network::percent_mul(amount, 990000000)) &&
      (delegate_balance < amount)
    );

    let increase_delegate_stake_amount: u128 = 1000000000000000000000;

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);
    let expected_post_delegate_balance = Network::convert_to_balance(
      delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance + increase_delegate_stake_amount
    );

    Network::do_increase_delegate_stake(
      subnet_id,
      increase_delegate_stake_amount,
    );

    // ensure balance has increase
    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

    let post_delegate_balance = Network::convert_to_balance(
      delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );
    log::error!("post_delegate_balance      {:?}", post_delegate_balance);
    assert_eq!(post_delegate_balance, expected_post_delegate_balance);
  });
}

#[test]
fn test_claim_removal_of_delegate_stake() {
  new_test_ext().execute_with(|| {
    let _ = env_logger::builder().is_test(true).try_init();

    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let n_account = total_subnet_nodes + 1;

    let _ = Balances::deposit_creating(&account(n_account), amount+500);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

    let mut delegate_stake_to_be_added_as_shares = Network::convert_to_shares(
      amount,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );

    if total_subnet_delegated_stake_shares == 0 {
      delegate_stake_to_be_added_as_shares = delegate_stake_to_be_added_as_shares.saturating_sub(1000);
    }

    let starting_delegator_balance = Balances::free_balance(&account(n_account));

    assert_ok!(
      Network::add_to_delegate_stake(
        RuntimeOrigin::signed(account(n_account)),
        subnet_id,
        amount,
      ) 
    );

    let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(n_account), subnet_id);
    assert_eq!(delegate_shares, delegate_stake_to_be_added_as_shares);
    assert_ne!(delegate_shares, 0);

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

    let mut delegate_balance = Network::convert_to_balance(
      delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );
    // The first depositor will lose a percentage of their deposit depending on the size
    // https://docs.openzeppelin.com/contracts/4.x/erc4626#inflation-attack
    // assert_eq!(delegate_balance, delegate_stake_to_be_added_as_shares);
    assert!(
      (delegate_balance >= Network::percent_mul(amount, 990000000)) &&
      (delegate_balance < amount)
    );

    let epoch_length = EpochLength::get();
    let cooldown_epochs = DelegateStakeCooldownEpochs::get();

    System::set_block_number(System::block_number() + epoch_length * cooldown_epochs);

    let balance = Balances::free_balance(&account(n_account));
    let epoch = System::block_number() / epoch_length;

    assert_ok!(
      Network::remove_delegate_stake(
        RuntimeOrigin::signed(account(n_account)),
        subnet_id,
        delegate_shares,
      )
    );
    let post_balance = Balances::free_balance(&account(n_account));
    assert_eq!(post_balance, balance);

    let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(n_account));
    assert_eq!(unbondings.len(), 1);
    let (ledger_epoch, ledger_balance) = unbondings.iter().next().unwrap();
    assert_eq!(*ledger_epoch, &epoch + DelegateStakeCooldownEpochs::get());
    assert!(*ledger_balance <= delegate_balance);

    assert_err!(
      Network::claim_unbondings(
        RuntimeOrigin::signed(account(n_account)),
      ),
      Error::<Test>::NoStakeUnbondingsOrCooldownNotMet
    );

    System::set_block_number(System::block_number() + ((epoch_length  + 1) * cooldown_epochs));

    let pre_claim_balance = Balances::free_balance(&account(n_account));

    assert_ok!(
      Network::claim_unbondings(
        RuntimeOrigin::signed(account(n_account)),
      )
    );

    let after_claim_balance = Balances::free_balance(&account(n_account));

    assert_eq!(after_claim_balance, pre_claim_balance + *ledger_balance);

    log::error!("starting_delegator_balance {:?}", starting_delegator_balance);
    log::error!("after_claim_balance        {:?}", after_claim_balance);
    log::error!("post_balance               {:?}", post_balance);
    log::error!("ledger_balance             {:?}", ledger_balance);

    let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(n_account));
    assert_eq!(unbondings.len(), 0);
  });
}

// // #[test]
// // fn test_remove_to_delegate_stake_max_unlockings_per_epoch_err() {
// //   new_test_ext().execute_with(|| {
// //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;

// //     let stake_amount: u128 = MinStakeBalance::<Test>::get();

// //     build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
// //     let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

// //     let n_account = total_subnet_nodes + 1;

// //     let _ = Balances::deposit_creating(&account(n_account), amount+500);

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

// //     let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
// //     let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

// //     let mut delegate_stake_to_be_added_as_shares = Network::convert_to_shares(
// //       amount,
// //       total_subnet_delegated_stake_shares,
// //       total_subnet_delegated_stake_balance
// //     );

// //     if total_subnet_delegated_stake_shares == 0 {
// //       delegate_stake_to_be_added_as_shares = delegate_stake_to_be_added_as_shares.saturating_sub(1000);
// //     }

// //     System::set_block_number(System::block_number() + DelegateStakeCooldownEpochs::get() * EpochLength::get());

// //     let starting_delegator_balance = Balances::free_balance(&account(n_account));

// //     assert_ok!(
// //       Network::add_to_delegate_stake(
// //         RuntimeOrigin::signed(account(n_account)),
// //         subnet_id,
// //         amount,
// //       ) 
// //     );

// //     let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(n_account), subnet_id);

// //     assert_ok!(
// //       Network::remove_delegate_stake(
// //         RuntimeOrigin::signed(account(n_account)),
// //         subnet_id,
// //         delegate_shares/2,
// //       )
// //     );
// //     let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(n_account));
// //     assert_eq!(unbondings.len(), 1);

// //     assert_err!(
// //       Network::remove_delegate_stake(
// //         RuntimeOrigin::signed(account(n_account)),
// //         subnet_id,
// //         delegate_shares/2,
// //       ),
// //       Error::<Test>::MaxUnlockingsPerEpochReached
// //     );
// //   });
// // }

#[test]
fn test_remove_to_delegate_stake_max_unlockings_reached_err() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let n_account = total_subnet_nodes + 1;

    let _ = Balances::deposit_creating(&account(n_account), amount+500);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

    let mut delegate_stake_to_be_added_as_shares = Network::convert_to_shares(
      amount,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );

    if total_subnet_delegated_stake_shares == 0 {
      delegate_stake_to_be_added_as_shares = delegate_stake_to_be_added_as_shares.saturating_sub(1000);
    }

    System::set_block_number(System::block_number() + DelegateStakeCooldownEpochs::get() * EpochLength::get());

    let starting_delegator_balance = Balances::free_balance(&account(n_account));

    assert_ok!(
      Network::add_to_delegate_stake(
        RuntimeOrigin::signed(account(n_account)),
        subnet_id,
        amount,
      ) 
    );

    let max_unlockings = MaxDelegateStakeUnlockings::get();
    for n in 1..max_unlockings+2 {
      System::set_block_number(System::block_number() + EpochLength::get() + 1);
      if n > max_unlockings {
        assert_err!(
          Network::remove_delegate_stake(
            RuntimeOrigin::signed(account(n_account)),
            subnet_id,
            1000,
          ),
          Error::<Test>::MaxUnlockingsReached
        );    
      } else {
        assert_ok!(
          Network::remove_delegate_stake(
            RuntimeOrigin::signed(account(n_account)),
            subnet_id,
            1000,
          )
        );
        let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(n_account));
        assert_eq!(unbondings.len() as u32, n);
      }
    }
  });
}

#[test]
fn test_switch_delegate_stake() {
  new_test_ext().execute_with(|| {
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    let from_subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    build_activated_subnet(from_subnet_path.clone(), 0, 0, deposit_amount, stake_amount);
    let from_subnet_id = SubnetPaths::<Test>::get(from_subnet_path.clone()).unwrap();

    let to_subnet_path: Vec<u8> = "petals-team/StableBeluga3".into();
    build_activated_subnet(to_subnet_path.clone(), 0, 0, deposit_amount, stake_amount);
    let to_subnet_id = SubnetPaths::<Test>::get(to_subnet_path.clone()).unwrap();

    let n_account = 255;

    let _ = Balances::deposit_creating(&account(n_account), amount+500);

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(from_subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(from_subnet_id);

    let mut delegate_stake_to_be_added_as_shares = Network::convert_to_shares(
      amount,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );

    if total_subnet_delegated_stake_shares == 0 {
      delegate_stake_to_be_added_as_shares = delegate_stake_to_be_added_as_shares.saturating_sub(1000);
    }

    System::set_block_number(System::block_number() + DelegateStakeCooldownEpochs::get() * EpochLength::get());

    let starting_delegator_balance = Balances::free_balance(&account(n_account));

    assert_ok!(
      Network::add_to_delegate_stake(
        RuntimeOrigin::signed(account(n_account)),
        from_subnet_id,
        amount,
      ) 
    );

    let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(n_account), from_subnet_id);
    assert_eq!(delegate_shares, delegate_stake_to_be_added_as_shares);
    assert_ne!(delegate_shares, 0);

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(from_subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(from_subnet_id);

    let mut from_delegate_balance = Network::convert_to_balance(
      delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );
    // The first depositor will lose a percentage of their deposit depending on the size
    // https://docs.openzeppelin.com/contracts/4.x/erc4626#inflation-attack
    // assert_eq!(from_delegate_balance, delegate_stake_to_be_added_as_shares);

    assert_ok!(
      Network::transfer_delegate_stake(
        RuntimeOrigin::signed(account(n_account)),
        from_subnet_id,
        to_subnet_id,
        delegate_shares,
      ) 
    );
    let from_delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(n_account), from_subnet_id);
    assert_eq!(from_delegate_shares, 0);

    let to_delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(n_account), to_subnet_id);
    assert_ne!(to_delegate_shares, 0);

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(to_subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(to_subnet_id);

    let mut to_delegate_balance = Network::convert_to_balance(
      to_delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );
    // The first depositor will lose a percentage of their deposit depending on the size
    // https://docs.openzeppelin.com/contracts/4.x/erc4626#inflation-attack
    // Will lose about .01% of the transfer value on first transfer into a pool
    // The balance should be about ~99% of the ``from`` subnet to the ``to`` subnet
    assert!(
      (to_delegate_balance >= Network::percent_mul(from_delegate_balance, 990000000)) &&
      (to_delegate_balance < from_delegate_balance)
    );
  });
}

#[test]
fn test_switch_delegate_stake_not_enough_stake_err() {
  new_test_ext().execute_with(|| {
    let _ = env_logger::builder().is_test(true).try_init();

    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    let from_subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    build_activated_subnet(from_subnet_path.clone(), 0, 0, deposit_amount, stake_amount);
    let from_subnet_id = SubnetPaths::<Test>::get(from_subnet_path.clone()).unwrap();

    let to_subnet_path: Vec<u8> = "petals-team/StableBeluga3".into();
    build_activated_subnet(to_subnet_path.clone(), 0, 0, deposit_amount, stake_amount);
    let to_subnet_id = SubnetPaths::<Test>::get(to_subnet_path.clone()).unwrap();

    // let n_account = 255;

    // let _ = Balances::deposit_creating(&account(n_account), amount+500);

    // assert_err!(
    //   Network::transfer_delegate_stake(
    //     RuntimeOrigin::signed(account(n_account)),
    //     from_subnet_id,
    //     to_subnet_id,
    //     0,
    //   ),
    //   Error::<Test>::NotEnoughStakeToWithdraw
    // );

    // assert_err!(
    //   Network::transfer_delegate_stake(
    //     RuntimeOrigin::signed(account(n_account)),
    //     from_subnet_id,
    //     to_subnet_id,
    //     1000,
    //   ),
    //   Error::<Test>::NotEnoughStakeToWithdraw
    // );
  });
}

// // #[test]
// // fn test_remove_to_delegate_stake_epochs_not_met_err() {
// //   new_test_ext().execute_with(|| {
// //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

// //     build_subnet(subnet_path.clone());
// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;
// //     let _ = Balances::deposit_creating(&account(0), amount+500);

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

// //     let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
// //     let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

// //     let mut delegate_stake_to_be_added_as_shares = Network::convert_to_shares(
// //       amount,
// //       total_subnet_delegated_stake_shares,
// //       total_subnet_delegated_stake_balance
// //     );

// //     if total_subnet_delegated_stake_shares == 0 {
// //       delegate_stake_to_be_added_as_shares = delegate_stake_to_be_added_as_shares.saturating_sub(1000);
// //     }

// //     System::set_block_number(System::block_number() + DelegateStakeCooldownEpochs::get() * EpochLength::get());

// //     assert_ok!(
// //       Network::add_to_delegate_stake(
// //         RuntimeOrigin::signed(account(0)),
// //         subnet_id,
// //         amount,
// //       ) 
// //     );

// //     let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(0), subnet_id);
// //     assert_eq!(delegate_shares, delegate_stake_to_be_added_as_shares);
// //     assert_ne!(delegate_shares, 0);

// //     let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
// //     let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

// //     let mut delegate_balance = Network::convert_to_balance(
// //       delegate_shares,
// //       total_subnet_delegated_stake_shares,
// //       total_subnet_delegated_stake_balance
// //     );
// //     // The first depositor will lose a percentage of their deposit depending on the size
// //     // https://docs.openzeppelin.com/contracts/4.x/erc4626#inflation-attack
// //     assert_eq!(delegate_balance, delegate_stake_to_be_added_as_shares);
// //     assert!(
// //       (delegate_balance >= Network::percent_mul(amount, 990000000)) &&
// //       (delegate_balance < amount)
// //     );

// //     // assert_err!(
// //     //   Network::remove_delegate_stake(
// //     //     RuntimeOrigin::signed(account(0)),
// //     //     subnet_id,
// //     //     delegate_shares,
// //     //   ),
// //     //   Error::<Test>::InsufficientCooldown
// //     // );
// //   });
// // }

#[test]
fn test_remove_delegate_stake_after_subnet_remove() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let n_account = total_subnet_nodes + 1;

    let _ = Balances::deposit_creating(&account(n_account), amount+500);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

    let mut delegate_stake_to_be_added_as_shares = Network::convert_to_shares(
      amount,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );

    if total_subnet_delegated_stake_shares == 0 {
      delegate_stake_to_be_added_as_shares = delegate_stake_to_be_added_as_shares.saturating_sub(1000);
    }

    System::set_block_number(System::block_number() + DelegateStakeCooldownEpochs::get() * EpochLength::get());

    let starting_delegator_balance = Balances::free_balance(&account(n_account));

    assert_ok!(
      Network::add_to_delegate_stake(
        RuntimeOrigin::signed(account(n_account)),
        subnet_id,
        amount,
      ) 
    );

    let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(n_account), subnet_id);
    assert_eq!(delegate_shares, delegate_stake_to_be_added_as_shares);
    assert_ne!(delegate_shares, 0);

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

    let mut delegate_balance = Network::convert_to_balance(
      delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );
    // The first depositor will lose a percentage of their deposit depending on the size
    // https://docs.openzeppelin.com/contracts/4.x/erc4626#inflation-attack
    // assert_eq!(delegate_balance, delegate_stake_to_be_added_as_shares);
    assert!(
      (delegate_balance >= Network::percent_mul(amount, 990000000)) &&
      (delegate_balance < amount)
    );

    let epoch_length = EpochLength::get();
    let cooldown_epochs = DelegateStakeCooldownEpochs::get();

    assert_ok!(
      Network::do_remove_subnet( 
        subnet_path.clone().into(),
        SubnetRemovalReason::MinSubnetDelegateStake,
      )
    );

    // System::set_block_number(System::block_number() + epoch_length * cooldown_epochs);

    let balance = Balances::free_balance(&account(n_account));
    let epoch = System::block_number() / epoch_length;

    assert_ok!(
      Network::remove_delegate_stake(
        RuntimeOrigin::signed(account(n_account)),
        subnet_id,
        delegate_shares,
      )
    );
    let post_balance = Balances::free_balance(&account(n_account));
    assert_eq!(post_balance, balance);

    let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(n_account));
    assert_eq!(unbondings.len(), 1);
    let (ledger_epoch, ledger_balance) = unbondings.iter().next().unwrap();
    assert_eq!(*ledger_epoch, &epoch + DelegateStakeCooldownEpochs::get());
    assert!(*ledger_balance <= delegate_balance);


    assert_err!(
      Network::claim_unbondings(
        RuntimeOrigin::signed(account(n_account)),
      ),
      Error::<Test>::NoStakeUnbondingsOrCooldownNotMet
    );

    System::set_block_number(System::block_number() + ((epoch_length  + 1) * cooldown_epochs));

    assert_ok!(
      Network::claim_unbondings(
        RuntimeOrigin::signed(account(n_account)),
      )
    );

    let post_balance = Balances::free_balance(&account(n_account));

    assert!(
      (post_balance >= Network::percent_mul(starting_delegator_balance, 990000000)) &&
      (post_balance < starting_delegator_balance)
    );

    let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(n_account));
    assert_eq!(unbondings.len(), 0);
  });
}

#[test]
fn test_switch_delegate_stake_node_to_subnet() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 =         1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet_with_delegator_rewards(
      subnet_path.clone(), 
      0, 
      16, 
      deposit_amount, 
      stake_amount,
      DEFAULT_DELEGATE_REWARD_RATE,
    );

    let from_subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_from_subnet_nodes = TotalSubnetNodes::<Test>::get(from_subnet_id);

    let to_subnet_path: Vec<u8> = "petals-team/StableBeluga3".into();

    build_activated_subnet_with_delegator_rewards(
      to_subnet_path.clone(), 
      0, 
      16, 
      deposit_amount, 
      stake_amount,
      DEFAULT_DELEGATE_REWARD_RATE,
    );

    let to_subnet_id = SubnetPaths::<Test>::get(to_subnet_path.clone()).unwrap();

    let _ = Balances::deposit_creating(&account(total_from_subnet_nodes+1), amount+500);

    assert_ok!(
      Network::add_to_node_delegate_stake(
        RuntimeOrigin::signed(account(total_from_subnet_nodes+1)), 
        from_subnet_id,
        1,
        amount,
      )
    );

    let account_node_delegate_stake_shares = AccountNodeDelegateStakeShares::<Test>::get((account(total_from_subnet_nodes+1), from_subnet_id, 1));
    let total_node_delegate_stake_balance = TotalNodeDelegateStakeBalance::<Test>::get(from_subnet_id, 1);
    let total_node_delegate_stake_shares = TotalNodeDelegateStakeShares::<Test>::get(from_subnet_id, 1);

    let account_node_delegate_stake_balance = Network::convert_to_balance(
      account_node_delegate_stake_shares,
      total_node_delegate_stake_shares,
      total_node_delegate_stake_balance
    );

    assert!(
      (account_node_delegate_stake_balance >= Network::percent_mul(amount, 990000000)) &&
      (account_node_delegate_stake_balance < amount)
    );

    let account_node_delegate_stake_shares_to_be_removed = account_node_delegate_stake_shares / 2;

    let expected_balance_to_be_removed = Network::convert_to_balance(
      account_node_delegate_stake_shares_to_be_removed,
      total_node_delegate_stake_shares,
      total_node_delegate_stake_balance
    );

    let before_transfer_tensor = Balances::free_balance(&account(total_from_subnet_nodes+1));

    let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(total_from_subnet_nodes+1));
    assert_eq!(unbondings.len(), 0);

    assert_ok!(
      Network::transfer_from_node_to_subnet(
        RuntimeOrigin::signed(account(total_from_subnet_nodes+1)),
        from_subnet_id,
        1,
        to_subnet_id,
        account_node_delegate_stake_shares_to_be_removed,
      )
    );

    let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(total_from_subnet_nodes+1));
    assert_eq!(unbondings.len(), 0);

    let after_transfer_tensor = Balances::free_balance(&account(total_from_subnet_nodes+1));
    assert_eq!(after_transfer_tensor, before_transfer_tensor);

    let from_delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(total_from_subnet_nodes+1), from_subnet_id);
    assert_eq!(from_delegate_shares, 0);

    let to_delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(total_from_subnet_nodes+1), to_subnet_id);
    assert_ne!(to_delegate_shares, 0);

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(to_subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(to_subnet_id);

    let mut to_delegate_balance = Network::convert_to_balance(
      to_delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );
    // The first depositor will lose a percentage of their deposit depending on the size
    // https://docs.openzeppelin.com/contracts/4.x/erc4626#inflation-attack
    // Will lose about .01% of the transfer value on first transfer into a pool
    // The balance should be about ~99% of the ``from`` subnet to the ``to`` subnet
    assert!(
      (to_delegate_balance >= Network::percent_mul(expected_balance_to_be_removed, 990000000)) &&
      (to_delegate_balance < expected_balance_to_be_removed)
    );
  });
}

#[test]
fn test_switch_delegate_stake_subnet_to_node() {
  new_test_ext().execute_with(|| {
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    let from_subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    build_activated_subnet(from_subnet_path.clone(), 0, 16, deposit_amount, stake_amount);
    let from_subnet_id = SubnetPaths::<Test>::get(from_subnet_path.clone()).unwrap();

    let to_subnet_path: Vec<u8> = "petals-team/StableBeluga3".into();
    build_activated_subnet(to_subnet_path.clone(), 0, 16, deposit_amount, stake_amount);
    let to_subnet_id = SubnetPaths::<Test>::get(to_subnet_path.clone()).unwrap();

    let n_account = 255;

    let _ = Balances::deposit_creating(&account(n_account), amount+500);

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(from_subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(from_subnet_id);

    let mut delegate_stake_to_be_added_as_shares = Network::convert_to_shares(
      amount,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );

    if total_subnet_delegated_stake_shares == 0 {
      delegate_stake_to_be_added_as_shares = delegate_stake_to_be_added_as_shares.saturating_sub(1000);
    }

    System::set_block_number(System::block_number() + DelegateStakeCooldownEpochs::get() * EpochLength::get());

    let starting_delegator_balance = Balances::free_balance(&account(n_account));

    assert_ok!(
      Network::add_to_delegate_stake(
        RuntimeOrigin::signed(account(n_account)),
        from_subnet_id,
        amount,
      ) 
    );

    let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(n_account), from_subnet_id);
    assert_eq!(delegate_shares, delegate_stake_to_be_added_as_shares);
    assert_ne!(delegate_shares, 0);

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(from_subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(from_subnet_id);

    let mut from_delegate_balance = Network::convert_to_balance(
      delegate_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );
    // The first depositor will lose a percentage of their deposit depending on the size
    // https://docs.openzeppelin.com/contracts/4.x/erc4626#inflation-attack

    let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(n_account));
    assert_eq!(unbondings.len(), 0);
    let before_transfer_tensor = Balances::free_balance(&account(n_account));

    assert_ok!(
      Network::transfer_from_subnet_to_node(
        RuntimeOrigin::signed(account(n_account)),
        from_subnet_id,
        to_subnet_id,
        1,
        delegate_shares,
      )
    );

    let unbondings: BTreeMap<u32, u128> = StakeUnbondingLedger::<Test>::get(account(n_account));
    assert_eq!(unbondings.len(), 0);
    let after_transfer_tensor = Balances::free_balance(&account(n_account));
    assert_eq!(after_transfer_tensor, before_transfer_tensor);

    let account_node_delegate_stake_shares = AccountNodeDelegateStakeShares::<Test>::get((account(n_account), to_subnet_id, 1));
    let total_node_delegate_stake_balance = TotalNodeDelegateStakeBalance::<Test>::get(to_subnet_id, 1);
    let total_node_delegate_stake_shares = TotalNodeDelegateStakeShares::<Test>::get(to_subnet_id, 1);

    let account_node_delegate_stake_balance = Network::convert_to_balance(
      account_node_delegate_stake_shares,
      total_node_delegate_stake_shares,
      total_node_delegate_stake_balance
    );

    assert_ne!(account_node_delegate_stake_balance, 0);

    assert!(
      (account_node_delegate_stake_balance >= Network::percent_mul(from_delegate_balance, 990000000)) &&
      (account_node_delegate_stake_balance < from_delegate_balance)
    );
  });
}

#[test]
fn test_inflation_exploit_mitigation_dead_shares() {
  new_test_ext().execute_with(|| {
    let subnet_id = 1;
    let first_user = account(1);
    let second_user = account(2);
    let stake = 1_000_000_000_000;

    // Give both users balances to stake
    Balances::deposit_creating(&first_user, stake * 10);
    Balances::deposit_creating(&second_user, stake * 10);

    // First user delegates stake
    // assert_ok!(Network::do_add_delegate_stake(
    //   RuntimeOrigin::signed(first_user.clone()),
    //   subnet_id,
    //   stake
    // ));

    Network::do_add_delegate_stake(
      RuntimeOrigin::signed(first_user.clone()),
      subnet_id,
      stake
    );

    // Get shares after first stake

    let first_user_shares = AccountSubnetDelegateStakeShares::<Test>::get(&first_user, subnet_id);
    let total_shares_after_first = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);

    // Ensure that shares given are less than 100% of total because of pre-injected 1000 shares
    assert!(first_user_shares < total_shares_after_first);

    // Second user adds same stake
    // assert_ok!(Network::add_to_delegate_stake(
    //     RuntimeOrigin::signed(second_user.clone()),
    //     subnet_id,
    //     stake
    // ));
    Network::do_add_delegate_stake(
      RuntimeOrigin::signed(second_user.clone()),
      subnet_id,
      stake
    );

    // Get second user shares
    let second_user_shares = AccountSubnetDelegateStakeShares::<Test>::get(&second_user, subnet_id);
    let total_shares_after_both = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_balance_after_both = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

    log::error!("first_user_shares  {:?}", first_user_shares);
    log::error!("second_user_shares {:?}", second_user_shares);

    // Check that second user also received a fair share
    assert!(second_user_shares > 0);
    assert!(first_user_shares <= second_user_shares);

    let first_user_balance = Network::convert_to_balance(
      first_user_shares,
      total_shares_after_both,
      total_balance_after_both,
    );

    let second_user_balance = Network::convert_to_balance(
      second_user_shares,
      total_shares_after_both,
      total_balance_after_both,
    );

    log::error!("first_user_balance  {:?}", first_user_balance);
    log::error!("second_user_balance {:?}", second_user_balance);

    assert!(first_user_balance < second_user_balance);

    // Check that total shares increased correctly
    assert_eq!(first_user_shares + second_user_shares + 1000, total_shares_after_both);
  });
}

#[test]
fn test_no_inflation_exploit_via_increase_delegate_stake() {
  new_test_ext().execute_with(|| {
    let subnet_id = 1;
    let attacker = account(1);
    let initial_balance = 1_000_000;
    let stake_amount = 100_000;
    let reward_amount = 100_000;

    // Step 0: Fund attacker
    Balances::make_free_balance_be(&attacker, initial_balance);

    // Step 1: Attacker stakes
    assert_ok!(Network::do_add_delegate_stake(
      RuntimeOrigin::signed(attacker.clone()),
      subnet_id,
      stake_amount
    ));

    let shares_before = AccountSubnetDelegateStakeShares::<Test>::get(&attacker, subnet_id);
    let shares_total_before = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let pool_balance_before = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

    // Step 2: Attacker deposits reward (donation-style increase)
    Network::do_increase_delegate_stake(subnet_id, reward_amount);

    // Step 3: Check that no new shares were minted
    let shares_after_reward = AccountSubnetDelegateStakeShares::<Test>::get(&attacker, subnet_id);
    let shares_total_after_reward = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let pool_balance_before = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

    assert_eq!(shares_after_reward, shares_before);
    assert_eq!(shares_total_after_reward, shares_total_before);

    // Step 4: Unstake all
    assert_ok!(Network::do_remove_delegate_stake(
      RuntimeOrigin::signed(attacker.clone()),
      subnet_id,
      shares_after_reward
    ));

    // Step 5: Check final balance — should not exceed stake + reward
    let final_balance = Balances::free_balance(&attacker);
    let expected_max_balance = initial_balance; // he started with this

    // attacker should never receive more than they fairly deserve
    assert!(final_balance <= expected_max_balance + reward_amount);

    // In fact, he should end up with exactly stake + reward back
    assert!(final_balance <= initial_balance); // restaked and unstaked exactly once, reward goes to share value

    // ✅ Passed: No inflation exploit
  });
}

// ——————————————————————————————————————————————————————————————
// ERC‑4626 Donation Attack Scenario:
//
// 1) totalAssets=0, totalShares=0
// 2) Attacker deposits 1 → totalAssets=1, totalShares=1
// 3) Attacke "donates" 10_000 via do_increase_delegate_stake
//    → totalAssets=10_001, totalShares=1
// 4) Innocent LP deposits 10_000 → would mint
//    floor(10_000 * 1 / 10_001) = 0 shares
//    → WITHOUT mitigation: they get 0 shares silently
//    → WITH our mitigation: we detect zero shares and return Err(CouldNotConvertToShares)
//
// Inflation exploits are mitigated via:
//  - Min deposit of 1000 TENSOR
//  - minting of dead shares when at zero shares
//  - use of virtual shares using decimal offset is converting assets/shares
//
//
// ——————————————————————————————————————————————————————————————
#[test]
fn test_donation_attack_simulation() {
  new_test_ext().execute_with(|| {
    let _ = env_logger::builder().is_test(true).try_init();

    let subnet_id = 1;
    let attacker = account(1);
    let victim = account(2);

    // Initial attacker tokens
    // const ATTACKER_INITIAL_TOKENS: u128 = 10000;
    const ATTACKER_INITIAL_TOKENS: u128 = 10000000;
    // Small amount to initially deposit
    // const ATTACKER_INITIAL_DEPOSIT: u128 = 1;
    const ATTACKER_INITIAL_DEPOSIT: u128 = 1000;
    // Large amount to donate directly
    // const ATTACKER_DONATION: u128 = 9999;
    const ATTACKER_DONATION: u128 = 9999000;
    // Victim deposit amount
    // const VICTIM_DEPOSIT: u128 = 1000;
    const VICTIM_DEPOSIT: u128 = 1000000;

    Balances::make_free_balance_be(&attacker, ATTACKER_INITIAL_TOKENS);
    Balances::make_free_balance_be(&victim, VICTIM_DEPOSIT + 500);


    // ---- Step 1: Attacker deposits minimal amount ----
    // The MinDelegateStakeBalance (deposit min) is 1000, otherwise reverts with CouldNotConvertToBalance
    assert_ok!(
      Network::do_add_delegate_stake(
        RuntimeOrigin::signed(attacker.clone()),
        subnet_id,
        ATTACKER_INITIAL_DEPOSIT,
      )
    );

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

    // Validate initial deposit
    let attacker_balance = Network::convert_to_balance(
      AccountSubnetDelegateStakeShares::<Test>::get(&attacker, subnet_id),
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );
    log::error!("attacker_balance         {:?}", attacker_balance);

    assert_eq!(AccountSubnetDelegateStakeShares::<Test>::get(&attacker, subnet_id), ATTACKER_INITIAL_DEPOSIT);
    // assert_eq!(TotalSubnetDelegateStakeShares::<Test>::get(subnet_id), ATTACKER_INITIAL_DEPOSIT);
    // ---- We mint 1000 dead shares so we check against this
    assert_eq!(TotalSubnetDelegateStakeShares::<Test>::get(subnet_id), ATTACKER_INITIAL_DEPOSIT + 1000);
    assert_eq!(TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id), ATTACKER_INITIAL_DEPOSIT);


    // ---- Step 2: Attacker donates to inflate share price ----
    Network::do_increase_delegate_stake(subnet_id, ATTACKER_DONATION);

    // Vault now has 10000 tokens (1 + 9999999)
    // assert_eq!(TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id), ATTACKER_INITIAL_TOKENS);

    // ---- Step 3: Victim deposits and gets almost no shares ----
    // We ensure they get shares
    assert_ok!(
      Network::do_add_delegate_stake(
        RuntimeOrigin::signed(victim.clone()),
        subnet_id,
        VICTIM_DEPOSIT,
      )
    );

    let victim_shares = AccountSubnetDelegateStakeShares::<Test>::get(&victim, subnet_id);
    log::error!("victim_shares  {:?}", victim_shares);

    let total_subnet_delegated_stake_shares = TotalSubnetDelegateStakeShares::<Test>::get(subnet_id);
    let total_subnet_delegated_stake_balance = TotalSubnetDelegateStakeBalance::<Test>::get(subnet_id);

    let victim_balance = Network::convert_to_balance(
      victim_shares,
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );
    log::error!("VICTIM_DEPOSIT {:?}", VICTIM_DEPOSIT);
    log::error!("victim_balance {:?}", victim_balance);

    assert!(
      (victim_balance >= Network::percent_mul(VICTIM_DEPOSIT, 990000000)) &&
      (victim_balance <= VICTIM_DEPOSIT)
    );

    let attacker_balance = Network::convert_to_balance(
      AccountSubnetDelegateStakeShares::<Test>::get(&attacker, subnet_id),
      total_subnet_delegated_stake_shares,
      total_subnet_delegated_stake_balance
    );
    log::error!("attacker_balance         {:?}", attacker_balance);
    log::error!("ATTACKER_INITIAL_DEPOSIT {:?}", ATTACKER_INITIAL_DEPOSIT + ATTACKER_DONATION);

    assert!(attacker_balance < ATTACKER_INITIAL_DEPOSIT + ATTACKER_DONATION);

    // ---- Step 4: Attacker withdraws and gets profit ----
    // We ensure they do not profit from this attack
    assert_ok!(Network::do_remove_delegate_stake(
      RuntimeOrigin::signed(attacker.clone()),
      subnet_id,
      AccountSubnetDelegateStakeShares::<Test>::get(&attacker, subnet_id)
    ));

    let attacker_final_balance = Balances::free_balance(&attacker);

    assert!(attacker_final_balance < ATTACKER_INITIAL_TOKENS);

    // assert!(false);
  });
}
