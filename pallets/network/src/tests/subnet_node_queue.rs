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
  TotalStake, 
  SubnetRewardsValidator,
  SubnetPaths, 
  TotalSubnetNodes,
  SubnetNodeClass,
  SubnetNode,
  SubnetsData,
  AccountSubnetStake,
  RegistrationSubnetData,
  StakeUnbondingLedger, 
  TotalSubnetStake, 
  MinSubnetRegistrationBlocks,
  DefaultSubnetNodeUniqueParamLimit,
  HotkeyOwner, 
  TotalSubnetNodeUids, 
  HotkeySubnetNodeId, 
  SubnetNodeIdHotkey, 
  SubnetNodesData, 
  BootstrapPeerIdSubnetNode,
  PeerIdSubnetNode,
  DeactivationLedger, 
  SubnetNodeDeactivation, 
  MaxRewardRateDecrease,
  RewardRateUpdatePeriod,
  SubnetRegistrationEpochs,
  MinStakeBalance,
  RegisteredStakeCooldownEpochs,
  QueuedSubnetNodes,
  SubnetNodeClassification,
};

///
///
///
///
///
///
///
/// Subnet Nodes Add/Remove
///
///
///
///
///
///
///

#[test]
fn test_register_subnet_node() {
  new_test_ext().execute_with(|| {
    let subnet_path: Vec<u8> = "petals-team/StableBeluga2".into();
    
    let deposit_amount: u128 = 10000000000000000000000;
    let amount: u128 = 1000000000000000000000;

    let stake_amount: u128 = MinStakeBalance::<Test>::get();

    build_activated_subnet(subnet_path.clone(), 0, 0, deposit_amount, stake_amount);

    let subnet_id = SubnetPaths::<Test>::get(subnet_path.clone()).unwrap();
    let total_subnet_nodes = TotalSubnetNodes::<Test>::get(subnet_id);

    let _ = Balances::deposit_creating(&account(total_subnet_nodes+1), deposit_amount);

    let block = System::block_number();
    let epoch = get_epoch();

    assert_ok!(
      Network::register_subnet_node_2(
        RuntimeOrigin::signed(account(total_subnet_nodes+1)),
        subnet_id,
        account(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        peer(total_subnet_nodes+1),
        0,
        amount,
        None,
        None,
        None,
      )
    );

    let total_subnet_node_uids = TotalSubnetNodeUids::<Test>::get(subnet_id);
    let hotkey_subnet_node_id = HotkeySubnetNodeId::<Test>::get(subnet_id, account(total_subnet_nodes+1)).unwrap();
    assert_eq!(total_subnet_node_uids, hotkey_subnet_node_id);

    let hotkey = SubnetNodeIdHotkey::<Test>::get(subnet_id, hotkey_subnet_node_id);
    assert_eq!(hotkey, Some(account(total_subnet_nodes+1)));

    let coldkey = HotkeyOwner::<Test>::get(account(total_subnet_nodes+1));
    assert_eq!(account(total_subnet_nodes+1), coldkey);

    // assumed subnet node parameters
    let subnet_node: SubnetNode<<Test as frame_system::Config>::AccountId> = SubnetNode {
      id: hotkey_subnet_node_id,
      hotkey: account(total_subnet_nodes+1),
      peer_id: peer(total_subnet_nodes+1),
      bootstrap_peer_id: peer(total_subnet_nodes+1),
			classification: SubnetNodeClassification {
				class: SubnetNodeClass::Queue,
				start_epoch: epoch,
			},
      delegate_reward_rate: 0,
      last_delegate_reward_rate_update: 0,
      a: None,
      b: None,
      c: None,
    };

    let queue = QueuedSubnetNodes::<Test>::get(subnet_id);
    assert_eq!(queue.get_key_value(&hotkey_subnet_node_id), Some((&hotkey_subnet_node_id, &subnet_node)));

    let last_queue = queue.last_key_value().unwrap();
    let queue_subnet_node = last_queue.1;

    assert_eq!(queue_subnet_node, &subnet_node);
    assert_eq!(queue_subnet_node.hotkey, account(total_subnet_nodes+1));
    assert_eq!(queue_subnet_node.peer_id, peer(total_subnet_nodes+1));
    assert_eq!(queue_subnet_node.bootstrap_peer_id, peer(total_subnet_nodes+1));
    assert_eq!(queue_subnet_node.delegate_reward_rate, 0);
    assert_eq!(queue_subnet_node.last_delegate_reward_rate_update, 0);
    assert_eq!(queue_subnet_node.classification.class, SubnetNodeClass::Queue);
    assert_eq!(queue_subnet_node.classification.start_epoch, epoch);



    // let subnet_node = SubnetNodesData::<Test>::get(subnet_id, hotkey_subnet_node_id);

    let peer_subnet_node_id = PeerIdSubnetNode::<Test>::get(subnet_id, peer(total_subnet_nodes+1));
    assert_eq!(peer_subnet_node_id, hotkey_subnet_node_id);

    let bootstrap_peer_subnet_node_id = BootstrapPeerIdSubnetNode::<Test>::get(subnet_id, peer(total_subnet_nodes+1));
    assert_eq!(bootstrap_peer_subnet_node_id, hotkey_subnet_node_id);

    let account_subnet_stake = AccountSubnetStake::<Test>::get(account(total_subnet_nodes+1), subnet_id);
    assert_eq!(account_subnet_stake, amount);


    log::error!("start_epoch {:?}", epoch);

    let queue_epochs: u32 = 16;
    let epoch = get_epoch();
    let block = System::block_number();
    log::error!("lastepoch {:?}", epoch);


    Network::do_queue(block);

    // no increase in blocks, should still be in queue
    let queue = QueuedSubnetNodes::<Test>::get(subnet_id);
    assert_eq!(queue.get_key_value(&hotkey_subnet_node_id), Some((&hotkey_subnet_node_id, &subnet_node)));


    // let epoch_length: u32 = EpochLength::get();

    // let mut max_nodes_per_epoch = Network::get_subnet_churn_limit(subnet_id);
    // if max_nodes_per_epoch > epoch_length {
    //   max_nodes_per_epoch = epoch_length;
    // }
    // let registration_interval = epoch_length / max_nodes_per_epoch;

  })
}