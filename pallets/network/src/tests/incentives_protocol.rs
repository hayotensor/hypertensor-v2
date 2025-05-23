use super::mock::*;
use super::test_utils::*;
use crate::Event;
use sp_core::OpaquePeerId as PeerId;
use frame_support::{
	assert_noop, assert_ok, assert_err
};
use log::info;
use frame_support::traits::{OnInitialize, Currency};
use crate::{
  Error, 
  SubnetRewardsValidator,
  SubnetPaths, 
  TotalSubnetNodes,
  SubnetNodeClass,
  SubnetNodeData,
  SubnetsData,
  AccountSubnetStake,
  AccountSubnetDelegateStakeShares, 
  SubnetRewardsSubmission, 
  BaseValidatorReward,
  DelegateStakeRewardsPercentage,
  SubnetPenaltyCount, 
  MaxSubnetNodePenalties, 
  SubnetNodePenalties, 
  RegistrationSubnetData,
  SubnetRemovalReason,
  MaxSubnetPenaltyCount, 
  MinSubnetRegistrationBlocks, 
  SubnetActivationEnactmentBlocks,
  HotkeySubnetNodeId, 
  SubnetNodeIdHotkey, 
  PeerIdSubnetNode,
  MinStakeBalance,
  SubnetOwnerPercentage,
  SubnetNodesData,
  TotalNodeDelegateStakeShares,
};
use frame_support::BoundedVec;
use strum::IntoEnumIterator;
use sp_io::crypto::sr25519_sign;
use sp_runtime::{MultiSigner, MultiSignature};
use sp_io::crypto::sr25519_generate;
use frame_support::pallet_prelude::Encode;
use sp_runtime::traits::IdentifyAccount;
use sp_core::Pair;
use sp_std::collections::{btree_map::BTreeMap, btree_set::BTreeSet};


//
//
//
//
//
//
//
// Validate / Attest / Rewards
//
//
//
//
//
//
//

// Validate 

#[test]
fn test_validate() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 12, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    // increase_epochs(1);

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    Network::do_epoch_preliminaries(System::block_number(), epoch, epoch_length);

    let subnet_node_data_vec = subnet_node_data(0, total_subnet_nodes);

    let validator_id = SubnetRewardsValidator::<Test>::get(subnet_id, epoch);
    assert!(validator_id != None, "Validator is None");

    let hotkey = SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();
    // assert!(hotkey != None, "Validator is None");

    assert_ok!(
      Network::validate(
        RuntimeOrigin::signed(hotkey), 
        subnet_id,
        subnet_node_data_vec.clone(),
        None,
      )
    );

    let submission = SubnetRewardsSubmission::<Test>::get(subnet_id, epoch).unwrap();

    assert_eq!(submission.validator_id, validator_id.unwrap(), "Err: validator");
    assert_eq!(submission.data.len(), subnet_node_data_vec.len(), "Err: data len");
    let sum = submission.data.iter().fold(0, |acc, x| acc + x.score);
    assert_eq!(sum, DEFAULT_SCORE * total_subnet_nodes as u128, "Err: sum");
    assert_eq!(submission.attests.len(), 1, "Err: attests"); // validator auto-attests

    assert_err!(
      Network::validate(
        RuntimeOrigin::signed(account(1)), 
        subnet_id,
        subnet_node_data_vec.clone(),
        None,
      ),
      Error::<Test>::SubnetRewardsAlreadySubmitted
    );
  });
}

#[test]
fn test_validate_peer_with_0_score() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    Network::do_epoch_preliminaries(System::block_number(), epoch, epoch_length);

    let mut subnet_node_data_vec: Vec<SubnetNodeData> = Vec::new();
    for n in 0..total_subnet_nodes {
      let mut peer_subnet_node_data: SubnetNodeData = SubnetNodeData {
        peer_id: peer(n),
        score: DEFAULT_SCORE,
      };

      if n == total_subnet_nodes {
        peer_subnet_node_data.score = 0
      }

      subnet_node_data_vec.push(peer_subnet_node_data);
    }
  
    let validator_id = SubnetRewardsValidator::<Test>::get(subnet_id, epoch);
    assert!(validator_id != None, "Validator is None");

    let hotkey = SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

    assert_ok!(
      Network::validate(
        RuntimeOrigin::signed(account(1)), 
        subnet_id,
        subnet_node_data_vec.clone(),
        None,
      )
    );

    let submission = SubnetRewardsSubmission::<Test>::get(subnet_id, epoch).unwrap();
    let data = submission.data;

    // peer should be removed due to 0 score
    for n in data {
      if n.peer_id == peer(total_subnet_nodes) {
        assert!(false);
      }
    }
  });
}

#[test]
fn test_validate_invalid_validator() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    // increase_epochs(1);
    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    Network::do_epoch_preliminaries(System::block_number(), epoch, epoch_length);

    let subnet_node_data_vec = subnet_node_data(0, total_subnet_nodes);

    let validator_id = SubnetRewardsValidator::<Test>::get(subnet_id, epoch);
    assert!(validator_id != None, "Validator is None");

    let mut validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

    if validator.clone() == account(1) {
      validator = account(2);
    }
  
    assert_err!(
      Network::validate(
        RuntimeOrigin::signed(validator.clone()), 
        subnet_id,
        subnet_node_data_vec,
        None,
      ),
      Error::<Test>::InvalidValidator
    );
  });
}

// Attest

#[test]
fn test_attest() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    // increase_epochs(1);

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    Network::do_epoch_preliminaries(System::block_number(), epoch, epoch_length);

    let subnet_node_data_vec = subnet_node_data(0, total_subnet_nodes);

    let validator_id = SubnetRewardsValidator::<Test>::get(subnet_id, epoch);
    assert!(validator_id != None, "Validator is None");
    assert!(validator_id != Some(0), "Validator is 0");

    let mut validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id.unwrap()).unwrap();

    assert_ok!(
      Network::validate(
        RuntimeOrigin::signed(validator.clone()), 
        subnet_id,
        subnet_node_data_vec.clone(),
        None,
      )
    );

    // Attest
    for n in 1..total_subnet_nodes+1 {
      let attestor = SubnetNodeIdHotkey::<Test>::get(subnet_id, n).unwrap();
      if attestor == validator.clone() {
        continue
      }
      assert_ok!(
        Network::attest(
          RuntimeOrigin::signed(account(n)), 
          subnet_id,
        )
      );
    }
    
    let submission = SubnetRewardsSubmission::<Test>::get(subnet_id, epoch).unwrap();

    assert_eq!(submission.validator_id, validator_id.unwrap());
    assert_eq!(submission.data.len(), subnet_node_data_vec.len());
    assert_eq!(submission.attests.len(), total_subnet_nodes as usize);
    
    // for n in 1..total_subnet_nodes+1 {
    //   let attestor = SubnetNodeIdHotkey::<Test>::get(subnet_id, n).unwrap();
    //   if attestor == validator.clone() {
    //     assert_ne!(submission.attests.get(&n), None);
    //     assert_eq!(submission.attests.get(&n), Some(&System::block_number()));
    //   } else {
    //     assert_ne!(submission.attests.get(&n), None);
    //     assert_eq!(submission.attests.get(&n), Some(&System::block_number()));
    //   }
    // }

    // if account(1) == validator.clone() {
    //   assert_ne!(submission.attests.get(&validator_id.unwrap()), None);
    //   assert_eq!(submission.attests.get(&validator_id.unwrap()), Some(&System::block_number()));
    // } else {
    //   assert_ne!(submission.attests.get(&2), None);
    //   assert_eq!(submission.attests.get(&2), Some(&System::block_number()));
    // }
  });
}


#[test]
fn test_attest_remove_exiting_attester() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    // increase_epochs(1);

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    Network::do_epoch_preliminaries(System::block_number(), epoch, epoch_length);

    let subnet_node_data_vec = subnet_node_data(0, total_subnet_nodes);

    // --- Get validator
    let validator_id = SubnetRewardsValidator::<Test>::get(subnet_id, epoch).unwrap();
    let mut validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id).unwrap();

    assert_ok!(
      Network::validate(
        RuntimeOrigin::signed(account(1)), 
        subnet_id,
        subnet_node_data_vec.clone(),
        None,
      )
    );

    // Attest
    for n in 1..total_subnet_nodes+1 {
      let attestor = SubnetNodeIdHotkey::<Test>::get(subnet_id, n).unwrap();
      if attestor == validator.clone() {
        continue
      }
      assert_ok!(
        Network::attest(
          RuntimeOrigin::signed(account(n)), 
          subnet_id,
        )
      );
    }

    // for n in 1..total_subnet_nodes+1 {
    //   if account(n) == validator.clone() {
    //     continue
    //   }
    //   assert_ok!(
    //     Network::attest(
    //       RuntimeOrigin::signed(account(n)), 
    //       subnet_id,
    //     )
    //   );
    // }
    
    let submission = SubnetRewardsSubmission::<Test>::get(subnet_id, epoch).unwrap();

    assert_eq!(submission.validator_id, validator_id);
    assert_eq!(submission.data.len(), subnet_node_data_vec.len());
    assert_eq!(submission.attests.len(), total_subnet_nodes as usize);

    for n in 1..total_subnet_nodes+1 {
      let attestor = SubnetNodeIdHotkey::<Test>::get(subnet_id, n).unwrap();
      if attestor == validator.clone() {
        assert_ne!(submission.attests.get(&n), None);
        assert_eq!(submission.attests.get(&n), Some(&System::block_number()));
      } else {
        assert_ne!(submission.attests.get(&n), None);
        assert_eq!(submission.attests.get(&n), Some(&System::block_number()));
      }
    }

    // if account(1) == validator.clone() {
    //   assert_ne!(submission.attests.get(&0), None);
    //   assert_eq!(submission.attests.get(&0), Some(&System::block_number()));
    // } else {
    //   assert_ne!(submission.attests.get(&1), None);
    //   assert_eq!(submission.attests.get(&1), Some(&System::block_number()));
    // }

    let subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(1)).unwrap();
    assert_ok!(
      Network::remove_subnet_node(
        RuntimeOrigin::signed(account(1)), 
        subnet_id,
        subnet_node_id,
      )
    );

    post_remove_subnet_node_ensures(1, subnet_id);

    let submission = SubnetRewardsSubmission::<Test>::get(subnet_id, epoch).unwrap();
    assert_eq!(submission.attests.len(), (total_subnet_nodes - 1) as usize);
    assert_eq!(submission.attests.get(&subnet_node_id), None);
  });
}

#[test]
fn test_attest_no_submission_err() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    // increase_epochs(1);

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    Network::do_epoch_preliminaries(System::block_number(), epoch, epoch_length);

    let subnet_node_data_vec = subnet_node_data(0, total_subnet_nodes);

    // --- Get validator
    let validator_id = SubnetRewardsValidator::<Test>::get(subnet_id, epoch).unwrap();
    let mut validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id).unwrap();

    assert_err!(
      Network::attest(
        RuntimeOrigin::signed(validator), 
        subnet_id,
      ),
      Error::<Test>::InvalidSubnetRewardsSubmission
    );
  });
}

#[test]
fn test_attest_already_attested_err() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    // increase_epochs(1);

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    Network::do_epoch_preliminaries(System::block_number(), epoch, epoch_length);

    let subnet_node_data_vec = subnet_node_data(0, total_subnet_nodes);

    let validator_id = SubnetRewardsValidator::<Test>::get(subnet_id, epoch).unwrap();
    let mut validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id).unwrap();

    assert_ok!(
      Network::validate(
        RuntimeOrigin::signed(validator.clone()), 
        subnet_id,
        subnet_node_data_vec.clone(),
        None,
      )
    );

    // Attest
    for n in 1..total_subnet_nodes+1 {
      if account(n) == validator.clone() {
        continue
      }
      assert_ok!(
        Network::attest(
          RuntimeOrigin::signed(account(n)), 
          subnet_id,
        )
      );
    }
    
    let submission = SubnetRewardsSubmission::<Test>::get(subnet_id, epoch).unwrap();

    assert_eq!(submission.validator_id, validator_id);
    assert_eq!(submission.data.len(), subnet_node_data_vec.len());
    let sum = submission.data.iter().fold(0, |acc, x| acc + x.score);
    assert_eq!(sum, DEFAULT_SCORE * total_subnet_nodes as u128);
    assert_eq!(submission.attests.len(), total_subnet_nodes as usize);

    for n in 1..total_subnet_nodes+1 {
      if account(n) == validator.clone() {
        continue
      }
      assert_ne!(submission.attests.get(&n), None);
      assert_eq!(submission.attests.get(&n), Some(&System::block_number()));
    }

    for n in 1..total_subnet_nodes+1 {
      if account(n) == validator.clone() {
        continue
      }
      assert_err!(
        Network::attest(
          RuntimeOrigin::signed(account(n)), 
          subnet_id,
        ),
        Error::<Test>::AlreadyAttested
      );
    }
  });
}

//
//
//
//
//
//
//
// Rewards
//
//
//
//
//
//
//

#[test]
fn test_reward_subnets() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    // increase_epochs(1);

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    Network::do_epoch_preliminaries(System::block_number(), epoch, epoch_length);


    let subnet_node_data_vec = subnet_node_data(0, total_subnet_nodes);

    // --- Get validator
    let validator_id = SubnetRewardsValidator::<Test>::get(subnet_id, epoch).unwrap();
    let mut validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id).unwrap();

    assert_ok!(
      Network::validate(
        RuntimeOrigin::signed(validator.clone()), 
        subnet_id,
        subnet_node_data_vec.clone(),
        None,
      )
    );

    // Attest
    for n in 1..total_subnet_nodes+1 {
      if account(n) == validator.clone() {
        continue
      }
      assert_ok!(
        Network::attest(
          RuntimeOrigin::signed(account(n)), 
          subnet_id,
        )
      );
    }
    
    Network::reward_subnets(System::block_number(), epoch);
  });
}

#[test]
fn test_reward_subnets_v2() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    // increase_epochs(1);

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    Network::do_epoch_preliminaries(System::block_number(), epoch, epoch_length);


    let subnet_node_data_vec = subnet_node_data(0, total_subnet_nodes);

    // --- Get validator
    let validator_id = SubnetRewardsValidator::<Test>::get(subnet_id, epoch).unwrap();
    let mut validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id).unwrap();

    assert_ok!(
      Network::validate(
        RuntimeOrigin::signed(validator.clone()), 
        subnet_id,
        subnet_node_data_vec.clone(),
        None,
      )
    );

    // Attest
    for n in 1..total_subnet_nodes+1 {
      if account(n) == validator.clone() {
        continue
      }
      assert_ok!(
        Network::attest(
          RuntimeOrigin::signed(account(n)), 
          subnet_id,
        )
      );
    }
    
    Network::reward_subnets_v2(System::block_number(), epoch);
  });
}


// #[test]
// fn test_reward_subnets_remove_subnet_node() {
//   new_test_ext().execute_with(|| {
//     let max_absent = MaxSubnetNodePenalties::<Test>::get();
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;

// let stake_amount: u128 = MinStakeBalance::<Test>::get();

// build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
//     let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

//     increase_epochs(1);

//     let epoch_length = EpochLength::get();

//     // shift node classes
//     // validate n-1
//     // attest   n-1
//     // Simulate epochs
//     for num in 0..max_absent+1 {
//       let epoch = System::block_number() / epoch_length;
  
//       let subnet_node_data_vec = subnet_node_data(0, total_subnet_nodes-1);
    
//       // --- Insert validator
//       SubnetRewardsValidator::<Test>::insert(subnet_id, epoch, 1);
//       let validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, 1).unwrap();

//       // validate without n-1
//       assert_ok!(
//         Network::validate(
//           RuntimeOrigin::signed(account(1)), 
//           subnet_id,
//           subnet_node_data_vec.clone(),
//           None,
//         )
//       );
  
//       // Attest without n-1
//       for n in 1..total_subnet_nodes {
//         let attestor = SubnetNodeIdHotkey::<Test>::get(subnet_id, n).unwrap();
//         if attestor == validator.clone() {
//           continue
//         }  
//         assert_ok!(
//           Network::attest(
//             RuntimeOrigin::signed(account(n)), 
//             subnet_id,
//           )
//         );
//       }
      
//       // --- Get submission data and count before node is removed
//       // Check rewards
//       // Ensure only attestors, validators, and validated get rewards
//       let submission = SubnetRewardsSubmission::<Test>::get(subnet_id, epoch).unwrap();

//       // --- Any removals impact the following epochs attestation data unless removed ahead of rewards
//       let submission_nodes: BTreeSet<<Test as frame_system::Config>::AccountId> = Network::get_classified_hotkeys(
//         subnet_id, 
//         &SubnetNodeClass::Validator, 
//         epoch
//       );

//       let submission_nodes_count = submission_nodes.len() as u128;

//       Network::reward_subnets(System::block_number(), epoch);
//       let node_absent_count = SubnetNodePenalties::<Test>::get(subnet_id, total_subnet_nodes);

//       if num + 1 > max_absent {
//         post_remove_subnet_node_ensures(total_subnet_nodes, subnet_id);
//         // when node is removed they're SubnetNodePenalties is reset to zero
//         assert_eq!(node_absent_count, 0);  
//       } else {
//         assert_eq!(node_absent_count, num+1);  
//       }

//       let base_reward_per_mb: u128 = BaseRewardPerMB::<Test>::get();
//       let delegate_stake_rewards_percentage: u128 = DelegateStakeRewardsPercentage::<Test>::get();
//       let overall_subnet_reward: u128 = Network::percent_mul(base_reward_per_mb, DEFAULT_MEM_MB);
//       let delegate_stake_reward: u128 = Network::percent_mul(overall_subnet_reward, delegate_stake_rewards_percentage);
//       let subnet_reward: u128 = overall_subnet_reward.saturating_sub(delegate_stake_reward);
      
//       let sum = submission.data.iter().fold(0, |acc, x| acc + x.score);

//       let reward_ratio: u128 = Network::percent_div(DEFAULT_SCORE, sum);
//       let account_reward: u128 = Network::percent_mul(reward_ratio, subnet_reward);
  
//       let base_reward = BaseValidatorReward::<Test>::get();
  
//       let submission_attestations: u128 = submission.attests.len() as u128;
//       let attestation_percentage: u128 = Network::percent_div(submission_attestations, submission_nodes_count);

//       // check each subnet nodes balance increased
//       for n in 1..total_subnet_nodes+1 {
//         if n == 1 {
//           // validator
//           let stake_balance: u128 = AccountSubnetStake::<Test>::get(&account(n), subnet_id);
//           let validator_reward: u128 = Network::percent_mul(base_reward, attestation_percentage);
//           assert_eq!(stake_balance, amount + (account_reward * (num+1) as u128) + (validator_reward * (num+1) as u128));
//         } else if n == total_subnet_nodes {
//           // node removed | should have no rewards
//           let stake_balance: u128 = AccountSubnetStake::<Test>::get(&account(n), subnet_id);
//           assert!(stake_balance == amount, "Invalid subnet node staking rewards");
//         } else {
//           // attestors
//           let stake_balance: u128 = AccountSubnetStake::<Test>::get(&account(n), subnet_id);
//           assert!(stake_balance == amount + (account_reward * (num+1) as u128), "Invalid subnet node staking rewards");
//         }
//       }

//       increase_epochs(1);
//     }

//     // node should be removed
//     let subnet_node_id = HotkeySubnetNodeId::<Test>::try_get(subnet_id, account(total_subnet_nodes));
//     assert_eq!(subnet_node_id, Err(()));

//     let subnet_node_account = PeerIdSubnetNode::<Test>::try_get(subnet_id, peer(total_subnet_nodes));
//     assert_eq!(subnet_node_account, Err(()));
//   });
// }

// // #[test]
// // fn test_reward_subnets_absent_node_increment_decrement() {
// //   new_test_ext().execute_with(|| {
// //     let max_absent = MaxSubnetNodePenalties::<Test>::get();
// //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
// //     let deposit_amount: u128 = 10000000000000000000000;
// //     let amount: u128 = 1000000000000000000000;

// //     build_activated_subnet(subnet_path.clone(), 0, 15, deposit_amount, amount);

// //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
// //     let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

// //     increase_epochs(1);

// //     let epoch_length = EpochLength::get();
// //     let epochs = SubnetNodeClassEpochs::<Test>::get(SubnetNodeClass::Accountant);

// //     // simulate epochs
// //     for num in 0..10 {
// //       let epoch = System::block_number() / epoch_length;

// //       // --- Insert validator
// //       SubnetRewardsValidator::<Test>::insert(subnet_id, epoch, account(1));
    
// //       if num % 2 == 0 {
// //         // increment on even epochs

// //         let subnet_node_data_vec = subnet_node_data(0, total_subnet_nodes-1);
    
// //         assert_ok!(
// //           Network::validate(
// //             RuntimeOrigin::signed(account(1)), 
// //             subnet_id,
// //             subnet_node_data_vec.clone()
// //           )
// //         );
    
// //         // Attest
// //         for n in 1..total_subnet_nodes-1 {
// //           assert_ok!(
// //             Network::attest(
// //               RuntimeOrigin::signed(account(n)), 
// //               subnet_id,
// //             )
// //           );
// //         }
        
// //         Network::reward_subnets(System::block_number(), epoch);
  
// //         let node_absent_count = SubnetNodePenalties::<Test>::get(subnet_id, (total_subnet_nodes-1));
// //         assert_eq!(node_absent_count, 1);
// //       } else {
// //         // decrement on odd epochs
// //         let subnet_node_data_vec = subnet_node_data(0, total_subnet_nodes);
        
// //         assert_ok!(
// //           Network::validate(
// //             RuntimeOrigin::signed(account(1)), 
// //             subnet_id,
// //             subnet_node_data_vec.clone()
// //           )
// //         );
    
// //         // Attest
// //         for n in 1..total_subnet_nodes {
// //           assert_ok!(
// //             Network::attest(
// //               RuntimeOrigin::signed(account(n)), 
// //               subnet_id,
// //             )
// //           );
// //         }
        
// //         Network::reward_subnets(System::block_number(), epoch);
  
// //         let node_absent_count = SubnetNodePenalties::<Test>::get(subnet_id, (total_subnet_nodes-1));
// //         assert_eq!(node_absent_count, 0);  
// //       }

// //       increase_epochs(1);
// //     }
// //   });
// // }

// #[test]
// fn test_reward_subnets_check_balances() {
//   new_test_ext().execute_with(|| {
//     let max_absent = MaxSubnetNodePenalties::<Test>::get();

//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
//     let deposit_amount: u128 = 10000000000000000000000;
//     let amount: u128 = 1000000000000000000000;

//     build_activated_subnet(subnet_path.clone(), 0, 15, deposit_amount, amount);

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
//     let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

//     increase_epochs(1);

//     let epoch_length = EpochLength::get();
//     let epoch = System::block_number() / epoch_length;

//     let subnet_node_data_vec = subnet_node_data(0, total_subnet_nodes);
  
//     // --- Insert validator
//     SubnetRewardsValidator::<Test>::insert(subnet_id, epoch, 1);
//     let validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, 1).unwrap();

//     // validate without n-1
//     assert_ok!(
//       Network::validate(
//         RuntimeOrigin::signed(account(1)), 
//         subnet_id,
//         subnet_node_data_vec.clone(),
//         None,
//       )
//     );

//     // Attest without n-1
//     for n in 1..total_subnet_nodes {
//       let attestor = SubnetNodeIdHotkey::<Test>::get(subnet_id, n).unwrap();
//       if attestor == validator.clone() {
//         continue
//       }
//       assert_ok!(
//         Network::attest(
//           RuntimeOrigin::signed(account(n)), 
//           subnet_id,
//         )
//       );
//     }
    
//     // --- Get submission data and count before node is removed
//     // Check rewards
//     // Ensure only attestors, validators, and validated get rewards
//     let submission = SubnetRewardsSubmission::<Test>::get(subnet_id, epoch).unwrap();

//     // --- Any removals impact the following epochs attestation data unless removed ahead of rewards
//     let submission_nodes: BTreeSet<<Test as frame_system::Config>::AccountId> = Network::get_classified_hotkeys(subnet_id, &SubnetNodeClass::Validator, epoch);
//     let submission_nodes_count = submission_nodes.len() as u128;

//     Network::reward_subnets(System::block_number(), epoch);
//     let node_absent_count = SubnetNodePenalties::<Test>::get(subnet_id, total_subnet_nodes-1);
//     assert_eq!(node_absent_count, 0); 
          
//     let base_reward_per_mb: u128 = BaseRewardPerMB::<Test>::get();
//     let delegate_stake_rewards_percentage: u128 = DelegateStakeRewardsPercentage::<Test>::get();
//     let overall_subnet_reward: u128 = Network::percent_mul(base_reward_per_mb, DEFAULT_MEM_MB);
//     let delegate_stake_reward: u128 = Network::percent_mul(overall_subnet_reward, delegate_stake_rewards_percentage);
//     let subnet_reward: u128 = overall_subnet_reward.saturating_sub(delegate_stake_reward);

//     let sum = submission.data.iter().fold(0, |acc, x| acc + x.score);
//     let reward_ratio: u128 = Network::percent_div(DEFAULT_SCORE, sum);
//     let account_reward: u128 = Network::percent_mul(reward_ratio, subnet_reward);

//     let base_reward = BaseValidatorReward::<Test>::get();

//     let submission_attestations: u128 = submission.attests.len() as u128;
//     let attestation_percentage: u128 = Network::percent_div(submission_attestations, submission_nodes_count);

//     // check each subnet nodes balance increased
//     for n in 1..total_subnet_nodes {
//       if n == 1 {
//         // validator
//         let stake_balance: u128 = AccountSubnetStake::<Test>::get(&account(n), subnet_id);
//         let validator_reward: u128 = Network::percent_mul(base_reward, attestation_percentage);
//         assert_eq!(stake_balance, amount + (account_reward as u128) + (validator_reward as u128));
//       } else {
//         // attestors
//         let stake_balance: u128 = AccountSubnetStake::<Test>::get(&account(n), subnet_id);
//         assert_eq!(stake_balance, amount + (account_reward as u128));
//       }
//     }
//   });
// }

#[test]
fn test_reward_subnets_validator_slash() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 15, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    // increase_epochs(1);

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    Network::do_epoch_preliminaries(System::block_number(), epoch, epoch_length);

    let subnet_node_data_vec = subnet_node_data(0, total_subnet_nodes);

    // --- Get validator
    let validator_id = SubnetRewardsValidator::<Test>::get(subnet_id, epoch).unwrap();
    let mut validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id).unwrap();

    assert_ok!(
      Network::validate(
        RuntimeOrigin::signed(validator.clone()), 
        subnet_id,
        subnet_node_data_vec.clone(),
        None,
      )
    );

    // No attests to ensure validator is slashed
    
    let before_slash_validator_stake_balance: u128 = AccountSubnetStake::<Test>::get(&validator.clone(), subnet_id);

    Network::reward_subnets(System::block_number(), epoch);

    let slashed_validator_stake_balance: u128 = AccountSubnetStake::<Test>::get(&validator.clone(), subnet_id);

    // Ensure validator was slashed
    assert!(before_slash_validator_stake_balance > slashed_validator_stake_balance, "Validator was not slashed")
  });
}

#[test]
fn test_reward_subnets_v2_validator_slash() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 15, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    // increase_epochs(1);

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    Network::do_epoch_preliminaries(System::block_number(), epoch, epoch_length);

    let subnet_node_data_vec = subnet_node_data(0, total_subnet_nodes);

    // --- Get validator
    let validator_id = SubnetRewardsValidator::<Test>::get(subnet_id, epoch).unwrap();
    let mut validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, validator_id).unwrap();

    assert_ok!(
      Network::validate(
        RuntimeOrigin::signed(validator.clone()), 
        subnet_id,
        subnet_node_data_vec.clone(),
        None,
      )
    );

    // No attests to ensure validator is slashed
    
    let before_slash_validator_stake_balance: u128 = AccountSubnetStake::<Test>::get(&validator.clone(), subnet_id);

    Network::reward_subnets_v2(System::block_number(), epoch);

    let slashed_validator_stake_balance: u128 = AccountSubnetStake::<Test>::get(&validator.clone(), subnet_id);

    // Ensure validator was slashed
    assert!(before_slash_validator_stake_balance > slashed_validator_stake_balance, "Validator was not slashed")
  });
}

#[test]
fn test_reward_subnets_subnet_penalty_count() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 15, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    increase_epochs(1);

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    let subnet_node_data_vec = subnet_node_data(0, total_subnet_nodes);

    // --- Insert validator
    SubnetRewardsValidator::<Test>::insert(subnet_id, epoch, 1);
    let validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, 1).unwrap();

    assert_ok!(
      Network::validate(
        RuntimeOrigin::signed(account(1)), 
        subnet_id,
        Vec::new(),
        None,
      )
    );

    // Attest
    for n in 1..total_subnet_nodes+1 {
      let attestor = SubnetNodeIdHotkey::<Test>::get(subnet_id, n).unwrap();
      if attestor == validator.clone() {
        continue
      }
      assert_ok!(
        Network::attest(
          RuntimeOrigin::signed(account(n)), 
          subnet_id,
        )
      );
    }
    
    Network::reward_subnets(System::block_number(), epoch);

    let subnet_penalty_count = SubnetPenaltyCount::<Test>::get(subnet_id);
    assert_eq!(subnet_penalty_count, 1);

    let subnet_node_penalty_count = SubnetNodePenalties::<Test>::get(subnet_id, 0);
    assert_eq!(subnet_node_penalty_count, 0);
  });
}

#[test]
fn test_reward_subnets_v2_subnet_penalty_count() {
  new_test_ext().execute_with(|| {
    let _ = env_logger::builder().is_test(true).try_init();

    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 15, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    increase_epochs(1);

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    let subnet_node_data_vec = subnet_node_data(0, total_subnet_nodes);

    // --- Insert validator
    SubnetRewardsValidator::<Test>::insert(subnet_id, epoch, 1);
    let validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, 1).unwrap();

    assert_ok!(
      Network::validate(
        RuntimeOrigin::signed(account(1)), 
        subnet_id,
        Vec::new(),
        None,
      )
    );

    // Attest
    for n in 1..total_subnet_nodes+1 {
      let attestor = SubnetNodeIdHotkey::<Test>::get(subnet_id, n).unwrap();
      if attestor == validator.clone() {
        continue
      }
      assert_ok!(
        Network::attest(
          RuntimeOrigin::signed(account(n)), 
          subnet_id,
        )
      );
    }
    
    Network::reward_subnets_v2(System::block_number(), epoch);

    let subnet_penalty_count = SubnetPenaltyCount::<Test>::get(subnet_id);
    assert_eq!(subnet_penalty_count, 1);

    let subnet_node_penalty_count = SubnetNodePenalties::<Test>::get(subnet_id, 0);
    assert_eq!(subnet_node_penalty_count, 0);
  });
}

#[test]
fn test_reward_subnets_account_penalty_count() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 15, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    increase_epochs(1);

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    let subnet_node_data_vec = subnet_node_data(0, total_subnet_nodes);

    // --- Insert validator
    SubnetRewardsValidator::<Test>::insert(subnet_id, epoch, 1);

    assert_ok!(
      Network::validate(
        RuntimeOrigin::signed(account(1)), 
        subnet_id,
        Vec::new(),
        None,
      )
    );

    // No Attest

    Network::reward_subnets(System::block_number(), epoch);

    let subnet_penalty_count = SubnetPenaltyCount::<Test>::get(subnet_id);
    assert_eq!(subnet_penalty_count, 1);

    let subnet_node_penalty_count = SubnetNodePenalties::<Test>::get(subnet_id, 1);
    assert_eq!(subnet_node_penalty_count, 1);
  });
}

#[test]
fn test_reward_subnets_v2_account_penalty_count() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 15, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    increase_epochs(1);

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    let subnet_node_data_vec = subnet_node_data(0, total_subnet_nodes);

    // --- Insert validator
    SubnetRewardsValidator::<Test>::insert(subnet_id, epoch, 1);

    assert_ok!(
      Network::validate(
        RuntimeOrigin::signed(account(1)), 
        subnet_id,
        Vec::new(),
        None,
      )
    );

    // No Attest

    Network::reward_subnets_v2(System::block_number(), epoch);

    let subnet_penalty_count = SubnetPenaltyCount::<Test>::get(subnet_id);
    assert_eq!(subnet_penalty_count, 1);

    let subnet_node_penalty_count = SubnetNodePenalties::<Test>::get(subnet_id, 1);
    assert_eq!(subnet_node_penalty_count, 1);
  });
}

// ///

// ///



// #[test]
// fn test_do_epoch_preliminaries_deactivate_subnet_enactment_period() {
//   new_test_ext().execute_with(|| {
//     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

//     let epoch_length = EpochLength::get();
//     let block_number = System::block_number();
//     let epoch = System::block_number().saturating_div(epoch_length);
  
//     let cost = Network::registration_cost(epoch);
  
//     let _ = Balances::deposit_creating(&account(1), cost+1000);
  
//     let add_subnet_data = RegistrationSubnetData {
//       path: subnet_path.clone().into(),
//       max_node_registration_epochs: 16,
//       node_registration_interval: 0,
//       node_queue_period: 1,
      // coldkey_whitelist: Some(BTreeSet::new()),
      // coldkey_whitelist: None,
//     };
  
//     let epoch_length = EpochLength::get();
//     let block_number = System::block_number();
//     let epoch = System::block_number().saturating_div(epoch_length);
//     let next_registration_epoch = Network::get_next_registration_epoch(epoch);
//     increase_epochs(next_registration_epoch - epoch);

//     // --- Register subnet for activation
//     assert_ok!(
//       Network::register_subnet(
//         RuntimeOrigin::signed(account(1)),
//         add_subnet_data,
//       )
//     );

//     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
//     let subnet = SubnetsData::<Test>::get(subnet_id).unwrap();

//     let min_subnet_delegate_stake = Network::get_min_subnet_delegate_stake_balance();
//     let _ = Balances::deposit_creating(&account(1), min_subnet_delegate_stake+1000);
  
//     let mut subnet_registering = true;
//     let subnet_activation_enactment_blocks = SubnetActivationEnactmentBlocks::<Test>::get();

//     while subnet_registering {
//       increase_epochs(1);
//       let block_number = System::block_number();

//       let epoch_length = EpochLength::get();
//       let epoch = System::block_number() / epoch_length;  

//       Network::do_epoch_preliminaries(block_number, epoch, epoch_length);
      
//       if block_number > max_registration_block + subnet_activation_enactment_blocks {
//         assert_eq!(
//           *network_events().last().unwrap(),
//           Event::SubnetDeactivated {
//             subnet_id: subnet_id, 
//             reason: SubnetRemovalReason::EnactmentPeriod
//           }
//         );

//         let removed_subnet = SubnetsData::<Test>::try_get(subnet_id);
//         assert_eq!(removed_subnet, Err(()));
//         subnet_registering = false;
//       } else {
//         let registered_subnet = SubnetsData::<Test>::try_get(subnet_id).unwrap();
//         assert_eq!(registered_subnet.id, subnet_id);
//       }
//     }
//   });
// }

#[test]
fn test_do_epoch_preliminaries_deactivate_min_subnet_delegate_stake() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    // --- Remove delegate stake to force MinSubnetDelegateStake removal reason
    let delegate_shares = AccountSubnetDelegateStakeShares::<Test>::get(account(1000), subnet_id);
    assert_ok!(
      Network::remove_delegate_stake(
        RuntimeOrigin::signed(account(1000)),
        subnet_id,
        delegate_shares,
      ) 
    );

    increase_epochs(1);
    let block_number = System::block_number();

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;  

    Network::do_epoch_preliminaries(block_number, epoch, epoch_length);
    assert_eq!(
      *network_events().last().unwrap(),
      Event::SubnetDeactivated {
        subnet_id: subnet_id, 
        reason: SubnetRemovalReason::MinSubnetDelegateStake
      }
    ); 
  });
}

#[test]
fn test_do_epoch_preliminaries_deactivate_max_penalties() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let max_subnet_penalty_count = MaxSubnetPenaltyCount::<Test>::get();
    SubnetPenaltyCount::<Test>::insert(subnet_id, max_subnet_penalty_count + 1);

    increase_epochs(1);
    let block_number = System::block_number();

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    Network::do_epoch_preliminaries(block_number, epoch, epoch_length);
    assert_eq!(
      *network_events().last().unwrap(),
      Event::SubnetDeactivated {
        subnet_id: subnet_id, 
        reason: SubnetRemovalReason::MaxPenalties
      }
    ); 
  });
}

#[test]
fn test_do_epoch_preliminaries_choose_validator() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    increase_epochs(1);
    let block_number = System::block_number();

    let epoch_length = EpochLength::get();
    let epoch = System::block_number() / epoch_length;

    Network::do_epoch_preliminaries(block_number, epoch, epoch_length);
    let validator = SubnetRewardsValidator::<Test>::get(subnet_id, epoch);
    assert_ne!(validator, None);
  });
}

// // // #[test]
// // // fn test_add_subnet_node_signature() {
// // //   new_test_ext().execute_with(|| {
// // //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

// // //     build_subnet(subnet_path.clone());
// // //     assert_eq!(Network::total_subnets(), 1);

// // // let mut n_peers: u32 = Network::max_subnet_nodes();
// // // if n_peers > MAX_SUBNET_NODES {
// // //   n_peers = MAX_SUBNET_NODES
// // // }

// // //     let deposit_amount: u128 = 1000000000000000000000000;
// // //     let amount: u128 = 1000000000000000000000;
// // //     let mut amount_staked: u128 = 0;

// // //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

// // //     let encoded_peer_id = Encode::encode(&peer(1).0.to_vec());
// // //     let public = sr25519_generate(0.into(), None);
// // //     let who_account: AccountIdOf<Test> = MultiSigner::Sr25519(public).into_account().into();
// // //     let signature =
// // //       MultiSignature::Sr25519(sr25519_sign(0.into(), &public, &encoded_peer_id).unwrap());

// // //     assert_ok!(
// // //       Network::add_subnet_node(
// // //         RuntimeOrigin::signed(account(1)),
// // account(1),
// // //         subnet_id,
// // //         peer(1),
// // //         amount,
// // //         // signature,
// // //         // who_account
// // //       ) 
// // //     );

// // //     let node_set = SubnetNodesClasses::<Test>::get(subnet_id, SubnetNodeClass::Queue);
// // //     assert_eq!(node_set.len(), n_peers as usize);

// // //   })
// // // }

// // // #[test]
// // // fn validate_signature() {
// // // 	new_test_ext().execute_with(|| {
// // // 		let user_1_pair = sp_core::sr25519::Pair::from_string("//Alice", None).unwrap();
// // // 		let user_1_signer = MultiSigner::Sr25519(user_1_pair.public());
// // //     log::error!("user_1_signer {:?}", user_1_signer);
// // // 		let user_1 = user_1_signer.clone().into_account();
// // //     log::error!("user_1 {:?}", user_1);
// // // 		let peer_id: PeerId = peer(1);
// // // 		let encoded_data = Encode::encode(&peer_id);
// // // 		let signature = MultiSignature::Sr25519(user_1_pair.sign(&encoded_data));
// // // 		assert_ok!(Network::validate_signature(&encoded_data, &signature, &user_1));

// // // 		let mut wrapped_data: Vec<u8> = Vec::new();
// // // 		wrapped_data.extend(b"<Bytes>");
// // // 		wrapped_data.extend(&encoded_data);
// // // 		wrapped_data.extend(b"</Bytes>");

// // // 		let signature = MultiSignature::Sr25519(user_1_pair.sign(&wrapped_data));
// // // 		assert_ok!(Network::validate_signature(&encoded_data, &signature, &user_1));
// // // 	})
// // // }

// // // #[test]
// // // fn validate_signature_and_peer() {
// // // 	new_test_ext().execute_with(|| {
// // //     // validate signature
// // // 		let user_1_pair = sp_core::sr25519::Pair::from_string("//Alice", None).unwrap();
// // // 		let user_1_signer = MultiSigner::Sr25519(user_1_pair.public());
// // // 		let user_1 = user_1_signer.clone().into_account();
// // // 		let peer_id: PeerId = peer(1);
// // // 		let encoded_data = Encode::encode(&peer_id);
// // // 		let signature = MultiSignature::Sr25519(user_1_pair.sign(&encoded_data));
// // // 		assert_ok!(Network::validate_signature(&encoded_data, &signature, &user_1));

// // // 		let mut wrapped_data: Vec<u8> = Vec::new();
// // // 		wrapped_data.extend(b"<Bytes>");
// // // 		wrapped_data.extend(&encoded_data);
// // // 		wrapped_data.extend(b"</Bytes>");

// // // 		let signature = MultiSignature::Sr25519(user_1_pair.sign(&wrapped_data));
// // // 		assert_ok!(Network::validate_signature(&encoded_data, &signature, &user_1));

// // //     // validate signature is the owner of the peer_id
// // //     let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();

// // //     build_subnet(subnet_path.clone());

// // //     let deposit_amount: u128 = 10000000000000000000000;
// // //     let amount: u128 = 1000000000000000000000;

// // //     let mut total_staked: u128 = 0;

// // //     let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();

// // //     let _ = Balances::deposit_creating(&user_1, deposit_amount);
    
// // //     assert_ok!(
// // //       Network::add_subnet_node(
// // //         RuntimeOrigin::signed(user_1),
// // //         subnet_id,
// // //         peer(1),
// // //         amount,
// // //       ) 
// // //     );
// // // 	})
// // // }

#[test]
fn test_reward_subnets_check_balances() {
  new_test_ext().execute_with(|| {
    let _ = env_logger::builder().is_test(true).try_init();

    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet_with_delegator_rewards(
      subnet_path.clone(), 
      0, 
      16, 
      deposit_amount, 
      stake_amount,
      DEFAULT_DELEGATE_REWARD_RATE,
    );

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), amount+500);

    assert_ok!(
      Network::add_to_node_delegate_stake(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)), 
        subnet_id,
        0,
        amount,
      )
    );

    increase_epochs(1);

    let epoch = get_epoch();

    let subnet_node_data_vec = subnet_node_data(0, total_subnet_nodes);
  
    // --- Insert validator
    SubnetRewardsValidator::<Test>::insert(subnet_id, epoch, 1);
    let validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, 1).unwrap();

    // validate without n-1
    assert_ok!(
      Network::validate(
        RuntimeOrigin::signed(account(1)), 
        subnet_id,
        subnet_node_data_vec.clone(),
        None,
      )
    );

    // Attest without n-1
    for n in 1..total_subnet_nodes+1 {
      let attestor = SubnetNodeIdHotkey::<Test>::get(subnet_id, n).unwrap();
      if attestor == validator.clone() {
        continue
      }
      assert_ok!(
        Network::attest(
          RuntimeOrigin::signed(account(n)), 
          subnet_id,
        )
      );
    }
    
    // --- Get submission data and count before node is removed
    // Check rewards
    // Ensure only attestors, validators, and validated get rewards
    let submission = SubnetRewardsSubmission::<Test>::get(subnet_id, epoch).unwrap();

    assert_ok!(Network::reward_subnets_v2(System::block_number(), epoch));

    let node_absent_count = SubnetNodePenalties::<Test>::get(subnet_id, total_subnet_nodes-1);
    assert_eq!(node_absent_count, 0); 

    let mut rewards: u128 = Network::get_epoch_emissions(epoch);

    let total_issuance: u128 = Network::get_total_network_issuance();

    let subnet_owner_percentage = SubnetOwnerPercentage::<Test>::get();
    let delegate_stake_rewards_percentage: u128 = DelegateStakeRewardsPercentage::<Test>::get();

    let weight = 1e+9 as u128;

    let overall_subnet_reward: u128 = Network::percent_mul(rewards, weight);

    // --- Get owner rewards
    let subnet_owner_reward: u128 = Network::percent_mul(overall_subnet_reward, subnet_owner_percentage);

    // --- Get subnet rewards minus owner cut
    let subnet_reward: u128 = overall_subnet_reward.saturating_sub(subnet_owner_reward);

    // --- Get delegators rewards
    let delegate_stake_reward: u128 = Network::percent_mul(subnet_reward, delegate_stake_rewards_percentage);

    // --- Get subnet nodes rewards
    let subnet_node_reward: u128 = subnet_reward.saturating_sub(delegate_stake_reward);

    // --- Any removals impact the following epochs attestation data unless removed ahead of rewards
    let submission_nodes: BTreeSet<<Test as frame_system::Config>::AccountId> = Network::get_classified_hotkeys(subnet_id, &SubnetNodeClass::Validator, epoch);
    let submission_nodes_count = submission_nodes.len() as u128;

    let attestations: u128 = submission.attests.len() as u128;
    let attestation_percentage: u128 = Network::percent_div(attestations, submission_nodes_count);
    assert_eq!(attestation_percentage, 1e+9 as u128);

    let sum = submission.data.iter().fold(0, |acc, x| acc + x.score);
    let score_percentage: u128 = Network::percent_div(DEFAULT_SCORE, sum);

    let mut account_reward: u128 = Network::percent_mul(score_percentage, subnet_node_reward);

    
    for n in 1..total_subnet_nodes+1 {
      let hotkey_subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(n)).unwrap();
      let subnet_node_id_hotkey = SubnetNodeIdHotkey::<Test>::get(subnet_id, hotkey_subnet_node_id).unwrap();
      let subnet_node = SubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
      let stake_balance: u128 = AccountSubnetStake::<Test>::get(&account(n), subnet_id);

      if subnet_node_id_hotkey == validator.clone() {
        // validator
        let validator_reward: u128 = Network::get_validator_reward(attestation_percentage);
        let validator_total_reward: u128 = (account_reward as u128) + (validator_reward as u128);
        assert_eq!(stake_balance, amount + validator_total_reward);
      } else {
        assert_eq!(stake_balance, amount + account_reward);
      }
    }
  });
}

#[test]
fn test_reward_subnets_with_delegate_node_staking_check_balances() {
  new_test_ext().execute_with(|| {
    let _ = env_logger::builder().is_test(true).try_init();

    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;
    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet_with_delegator_rewards(
      subnet_path.clone(), 
      0, 
      16, 
      deposit_amount, 
      stake_amount,
      DEFAULT_DELEGATE_REWARD_RATE,
    );

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);
    
    for n in 1..total_subnet_nodes+1 {
      let _ = Balances::deposit_creating(&account(total_subnet_nodes+2), amount+500);

      assert_ok!(
        Network::add_to_node_delegate_stake(
          RuntimeOrigin::signed(account(total_subnet_nodes+2)), 
          subnet_id,
          n,
          amount,
        )
      );  
    }

    increase_epochs(1);

    let epoch = get_epoch();

    let subnet_node_data_vec = subnet_node_data(0, total_subnet_nodes);
  
    // --- Insert validator
    SubnetRewardsValidator::<Test>::insert(subnet_id, epoch, 1);
    let validator = SubnetNodeIdHotkey::<Test>::get(subnet_id, 1).unwrap();

    // validate without n-1
    assert_ok!(
      Network::validate(
        RuntimeOrigin::signed(account(1)), 
        subnet_id,
        subnet_node_data_vec.clone(),
        None,
      )
    );

    // Attest without n-1
    for n in 1..total_subnet_nodes+1 {
      let attestor = SubnetNodeIdHotkey::<Test>::get(subnet_id, n).unwrap();
      if attestor == validator.clone() {
        continue
      }
      assert_ok!(
        Network::attest(
          RuntimeOrigin::signed(account(n)), 
          subnet_id,
        )
      );
    }
    
    // --- Get submission data and count before node is removed
    // Check rewards
    // Ensure only attestors, validators, and validated get rewards
    let submission = SubnetRewardsSubmission::<Test>::get(subnet_id, epoch).unwrap();

    assert_ok!(Network::reward_subnets_v2(System::block_number(), epoch));

    let node_absent_count = SubnetNodePenalties::<Test>::get(subnet_id, total_subnet_nodes-1);
    assert_eq!(node_absent_count, 0); 

    let mut rewards: u128 = Network::get_epoch_emissions(epoch);

    let total_issuance: u128 = Network::get_total_network_issuance();

    let subnet_owner_percentage = SubnetOwnerPercentage::<Test>::get();
    let delegate_stake_rewards_percentage: u128 = DelegateStakeRewardsPercentage::<Test>::get();

    let weight = 1e+9 as u128;

    let overall_subnet_reward: u128 = Network::percent_mul(rewards, weight);

    // --- Get owner rewards
    let subnet_owner_reward: u128 = Network::percent_mul(overall_subnet_reward, subnet_owner_percentage);

    // --- Get subnet rewards minus owner cut
    let subnet_reward: u128 = overall_subnet_reward.saturating_sub(subnet_owner_reward);

    // --- Get delegators rewards
    let delegate_stake_reward: u128 = Network::percent_mul(subnet_reward, delegate_stake_rewards_percentage);

    // --- Get subnet nodes rewards
    let subnet_node_reward: u128 = subnet_reward.saturating_sub(delegate_stake_reward);

    // --- Any removals impact the following epochs attestation data unless removed ahead of rewards
    let submission_nodes: BTreeSet<<Test as frame_system::Config>::AccountId> = Network::get_classified_hotkeys(subnet_id, &SubnetNodeClass::Validator, epoch);
    let submission_nodes_count = submission_nodes.len() as u128;

    let attestations: u128 = submission.attests.len() as u128;
    let attestation_percentage: u128 = Network::percent_div(attestations, submission_nodes_count);
    assert_eq!(attestation_percentage, 1e+9 as u128);

    let sum = submission.data.iter().fold(0, |acc, x| acc + x.score);
    let score_percentage: u128 = Network::percent_div(DEFAULT_SCORE, sum);

    let mut account_reward: u128 = Network::percent_mul(score_percentage, subnet_node_reward);
    log::error!("account_reward:            {:?}", account_reward);

    
    for n in 1..total_subnet_nodes+1 {
      let mut node_reward = account_reward;
      let hotkey_subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(n)).unwrap();
      let subnet_node_id_hotkey = SubnetNodeIdHotkey::<Test>::get(subnet_id, hotkey_subnet_node_id).unwrap();
      let subnet_node = SubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);
      let stake_balance: u128 = AccountSubnetStake::<Test>::get(&account(n), subnet_id);

      log::error!(" ");
      log::error!("subnet_node ID:        {:?}", hotkey_subnet_node_id);
      log::error!("subnet_node.delegate_reward_rate:        {:?}", subnet_node.delegate_reward_rate);

      assert_ne!(subnet_node.delegate_reward_rate, 0);

      if subnet_node_id_hotkey == validator.clone() {
        // validator
        let validator_reward: u128 = Network::get_validator_reward(attestation_percentage);

        if subnet_node.delegate_reward_rate != 0 {
          let total_node_delegated_stake_shares = TotalNodeDelegateStakeShares::<Test>::get(subnet_id, hotkey_subnet_node_id);
          if total_node_delegated_stake_shares != 0 {
            let node_delegate_reward = Network::percent_mul(node_reward, subnet_node.delegate_reward_rate);
            node_reward = node_reward - node_delegate_reward;
          }
        }
        let validator_total_reward: u128 = (node_reward as u128) + (validator_reward as u128);

        assert_eq!(stake_balance, amount + validator_total_reward);
      } else {
        if subnet_node.delegate_reward_rate != 0 {
          let total_node_delegated_stake_shares = TotalNodeDelegateStakeShares::<Test>::get(subnet_id, hotkey_subnet_node_id);
          if total_node_delegated_stake_shares != 0 {
            let node_delegate_reward = Network::percent_mul(node_reward, subnet_node.delegate_reward_rate);
            node_reward = node_reward - node_delegate_reward;
          }
        }
        assert_eq!(stake_balance, amount + node_reward);
      }
    }
  });
}