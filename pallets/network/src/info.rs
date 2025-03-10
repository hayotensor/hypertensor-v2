use super::*;

impl<T: Config> Pallet<T> {
  pub fn get_subnet_nodes(
    subnet_id: u32,
  ) -> Vec<SubnetNode<T::AccountId>> {
    if !SubnetsData::<T>::contains_key(subnet_id) {
      return Vec::new();
    }
    let block: u64 = Self::get_current_block_as_u64();
    let epoch_length: u64 = T::EpochLength::get();
    let epoch: u64 = block / epoch_length;
    Self::get_classified_subnet_nodes(subnet_id, &SubnetNodeClass::Idle, epoch)
  }

  pub fn get_subnet_nodes_included(
    subnet_id: u32,
  ) -> Vec<SubnetNode<T::AccountId>> {
    if !SubnetsData::<T>::contains_key(subnet_id) {
      return Vec::new();
    }
    let block: u64 = Self::get_current_block_as_u64();
    let epoch_length: u64 = T::EpochLength::get();
    let epoch: u64 = block / epoch_length;
    Self::get_classified_subnet_nodes(subnet_id, &SubnetNodeClass::Included, epoch)
  }

  pub fn get_subnet_nodes_submittable(
    subnet_id: u32,
  ) -> Vec<SubnetNode<T::AccountId>> {
    if !SubnetsData::<T>::contains_key(subnet_id) {
      return Vec::new();
    }
    let block: u64 = Self::get_current_block_as_u64();
    let epoch_length: u64 = T::EpochLength::get();
    let epoch: u64 = block / epoch_length;
    Self::get_classified_subnet_nodes(subnet_id, &SubnetNodeClass::Validator, epoch)
  }

  pub fn get_subnet_node_info(
    subnet_id: u32,
  ) -> Vec<SubnetNodeInfo<T::AccountId>> {
    if !SubnetsData::<T>::contains_key(subnet_id) {
      return Vec::new();
    }
    let block: u64 = Self::get_current_block_as_u64();
    let epoch_length: u64 = T::EpochLength::get();
    let epoch: u64 = block / epoch_length;
    Self::get_classified_subnet_node_info(subnet_id, &SubnetNodeClass::Validator, epoch)
  }

  pub fn get_subnet_nodes_subnet_unconfirmed_count(
    subnet_id: u32,
  ) -> u32 {
    if !SubnetsData::<T>::contains_key(subnet_id) {
      return 0;
    }

    0
  }

  pub fn get_subnet_node_by_params(
    subnet_id: u32,
    a: BoundedVec<u8, DefaultSubnetNodeUniqueParamLimit>,
  ) -> Option<SubnetNode<T::AccountId>> {
    if !SubnetsData::<T>::contains_key(subnet_id) {
      return None
    }

    SubnetNodesData::<T>::iter_prefix_values(subnet_id)
      .find(|x| {
        // Find by ``a``, a unique parameter
        x.a == Some(a.clone())
      })
  }

  // id is consensus ID
  pub fn get_consensus_data(
    subnet_id: u32,
    epoch: u32
  ) -> Option<RewardsData<T::AccountId>> {
    let data = SubnetRewardsSubmission::<T>::get(subnet_id, epoch);
    Some(data?)
  }

  pub fn get_minimum_subnet_nodes(memory_mb: u128) -> u32 {
    Self::get_min_subnet_nodes(BaseSubnetNodeMemoryMB::<T>::get(), memory_mb)
  }

  pub fn get_minimum_delegate_stake(memory_mb: u128) -> u128 {
    let min_nodes = Self::get_min_subnet_nodes(BaseSubnetNodeMemoryMB::<T>::get(), memory_mb);
    Self::get_min_subnet_delegate_stake_balance(min_nodes)
  }

  pub fn get_subnet_node_stake_by_peer_id(subnet_id: u32, peer_id: PeerId) -> u128 {
    match SubnetNodeAccount::<T>::try_get(subnet_id, peer_id.clone()) {
      Ok(hotkey) => {
        AccountSubnetStake::<T>::get(hotkey, subnet_id)
      },
      Err(()) => 0,
    }
  }

  // TODO: Make this only return true is Validator subnet node
  pub fn is_subnet_node_by_peer_id(subnet_id: u32, peer_id: Vec<u8>) -> bool {
    match SubnetNodeAccount::<T>::try_get(subnet_id, PeerId(peer_id)) {
      Ok(account_id) => true,
      Err(()) => false,
    }
  }

  pub fn are_subnet_nodes_by_peer_id(subnet_id: u32, peer_ids: Vec<Vec<u8>>) -> BTreeMap<Vec<u8>, bool> {
    let mut subnet_nodes: BTreeMap<Vec<u8>, bool> = BTreeMap::new();

    for peer_id in peer_ids.iter() {
      let is = match SubnetNodeAccount::<T>::try_get(subnet_id, PeerId(peer_id.clone())) {
        Ok(_) => true,
        Err(()) => false,
      };
      subnet_nodes.insert(peer_id.clone(), is);
    }

    subnet_nodes
  }

  /// If subnet node exists under unique subnet node parameter ``a``
  pub fn is_subnet_node_by_a(
    subnet_id: u32, 
    a: BoundedVec<u8, DefaultSubnetNodeUniqueParamLimit>
  ) -> bool {
    if !SubnetsData::<T>::contains_key(subnet_id) {
      return false
    }

    match SubnetNodeUniqueParam::<T>::try_get(subnet_id, a) {
      Ok(_) => true,
      Err(()) => false,
    }
  }
}