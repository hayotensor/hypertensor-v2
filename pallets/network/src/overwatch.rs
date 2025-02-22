use super::*;
use sp_runtime::traits::BlakeTwo256;
use sp_core::blake2_256;
use sp_std::vec;
use libm::sqrt;

impl<T: Config> Pallet<T> {
  pub fn submit_benchmark_weights(
    origin: T::RuntimeOrigin,
    mut encrypted_weights: Vec<SubnetBenchmarkWeightCommitment>,
  ) -> DispatchResult {
    let hotkey: T::AccountId = ensure_signed(origin)?;

    let subnets: BTreeSet<_> = SubnetsData::<T>::iter().map(|(id, _)| id).collect();
    let total_subnets: u32 = subnets.len() as u32;
    let total_commitments: u32 = encrypted_weights.len() as u32;

    ensure!(
      total_commitments <= total_subnets && total_commitments > 0,
      Error::<T>::InvalidOverwatchCommitments
    );

    // Remove dups
    encrypted_weights.dedup_by(|a, b| a.subnet_id == b.subnet_id);

    // Qualify IDs
    encrypted_weights.retain(|x| subnets.contains(&x.subnet_id));

    ensure!(
      !encrypted_weights.is_empty(),
      Error::<T>::InvalidOverwatchCommitments
    );

    let epoch: u32 = Self::get_current_epoch_as_u32();

    SubnetBenchmarkCommitments::<T>::insert(epoch, hotkey, encrypted_weights);

    Ok(())
  }

  pub fn reveal_benchmark_weights(
    origin: T::RuntimeOrigin,
    mut revealed_weights: Vec<SubnetBenchmarkWeightReveal>,
    seed: &[u8],
  ) -> DispatchResult {
    let hotkey: T::AccountId = ensure_signed(origin)?;

    let epoch: u32 = Self::get_current_epoch_as_u32();

    let encrypted_weights = match SubnetBenchmarkCommitments::<T>::try_get(epoch, hotkey.clone()) {
      Ok(weights) => weights,
      Err(()) => return Err(Error::<T>::CommitNotExist.into()),
    };

    let revealed_weights_len = revealed_weights.len();

    ensure!(
      encrypted_weights.len() <= revealed_weights_len && revealed_weights_len > 0,
      Error::<T>::InvalidOverwatchCommitments
    );

    let stake_balance = AccountOverwatchStake::<T>::get(hotkey.clone());

    for commit in encrypted_weights.iter() {
      let subnet_id = commit.subnet_id;
      let encrypted_weight = &commit.weight;

      let revealed_weight: Option<&SubnetBenchmarkWeightReveal> = revealed_weights.iter().find(
        |e| e.subnet_id == subnet_id
      );

      if revealed_weight.is_none() {
        continue
      }

      // --- Ensure weight under 100%
      ensure!(
        revealed_weight.unwrap().weight <= Self::PERCENTAGE_FACTOR,
        Error::<T>::InvalidWeight
      );

      let reveal = Self::reveal(revealed_weight.unwrap().weight, &seed);

      // --- Ensure reveal matches encrypted commit
      ensure!(
        encrypted_weight == &reveal,
        Error::<T>::WeightRevealMismatch
      );

      SubnetBenchmarkReveals::<T>::mutate(epoch as u32, subnet_id, |weights| {
        weights.insert(stake_balance, revealed_weight.unwrap().weight);
      });
    }
    
    Ok(())
  }

  pub fn compute_benchmark_weights(epoch: u32) {
    // Factored subnet weights {subnet_id: [..., commit*stake_weight, ...]}
    let mut subnet_node_weights: BTreeMap<u32, Vec<u128>> = BTreeMap::new();

    for (subnet_id, _) in SubnetsData::<T>::iter() {
      let mut reveals = match SubnetBenchmarkReveals::<T>::try_get(
        epoch,
        subnet_id, 
      ) {
        Ok(reveals) => reveals,
        Err(()) => BTreeMap::new(),
      };

      if reveals.is_empty() {
        continue
      }

      // Run IQR and remove outliers

      
      
      let total_overwatch_stake = 0;
      let mut subnet_total_weight: u128 = 0;
      let mut weights: Vec<u128> = Vec::new();
      for reveal in reveals.iter() {
        let stake_balance = reveal.0;
        let weight = reveal.1;

        let stake_weight = Self::percent_div(*stake_balance, total_overwatch_stake);

        let node_subnet_weight = Self::percent_mul(*weight as u128, stake_weight);

        subnet_total_weight = subnet_total_weight.saturating_add(node_subnet_weight);
        weights.push(subnet_total_weight);
      }

      subnet_node_weights.insert(subnet_id, weights);
    }

    let subnet_node_weights_sum: u128 = subnet_node_weights.values()
      .flat_map(|v| v.iter())
      .copied()
      .sum();

    let mut subnet_weights: BTreeMap<u32, u128> = BTreeMap::new();

    // Get percentages
    for (subnet_id, weights) in subnet_node_weights.iter() {
      let weight_sum: u128 = weights.iter().sum();
      let subnet_weight = Self::percent_div(weight_sum, subnet_node_weights_sum);
      subnet_weights.insert(*subnet_id, subnet_weight);
    }

    // u128 => f64 precision loss starts at around 9007199254740992
    // We max the possible weight at 1e9 so we can handle over 9 million entries
    // before any precision loss begins
    // (9.007m = 9007199254740992 / 1e9)

    // Get fair weights
    let mut subnet_weights_sum: u128 = subnet_weights.values().copied().sum();
    let mut subnet_fair_weights: BTreeMap<u32, u128> = BTreeMap::new();

    for (subnet_id, weight) in subnet_weights.iter() {
      let fair_weight: f64 = Self::adjusted_sqrt(*weight as f64, Self::PERCENTAGE_FACTOR as f64, 100.0);
      subnet_fair_weights.insert(*subnet_id, fair_weight as u128);
    }

    // Normalize fair weights
    let mut subnet_normalized_weights_sum: u128 = subnet_fair_weights.values().copied().sum();
    let mut subnet_normalized_fair_weights: BTreeMap<u32, u128> = BTreeMap::new();
    for (subnet_id, weight) in subnet_fair_weights.iter() {
      let subnet_normalized_weight = Self::percent_div(*weight, subnet_normalized_weights_sum);
      subnet_normalized_fair_weights.insert(*subnet_id, subnet_normalized_weight);
    }

  }

  pub fn reveal(value: u128, seed: &[u8]) -> [u8; 32] {
    let mut data = vec![];
    data.extend_from_slice(&value.to_le_bytes());
    data.extend_from_slice(&seed);
    blake2_256(&data)
  }

  // Helper function
  pub fn generate_commitment(value: u128, seed: &[u8]) -> [u8; 32] {
    let mut data = vec![];
    data.extend_from_slice(&value.to_le_bytes());
    data.extend_from_slice(&seed);
    blake2_256(&data)
  }

  /// Returns square root of factored number.
  ///
  /// # Arguments
  ///
  /// * `value` - The factored number, e.g. `0.01 * 1e9` as 1e9 being the factor.
  /// * `factor` - The factor, e.g. 1e9.
  /// * `base` - The number the factor represent, e.g. 1e9 represents 100.0.
  ///
  /// # Returns
  ///
  /// The normalized square root of the factored number in the factored format.
  ///
  /// # Example
  ///
  /// let result = Self::adjusted_sqrt(20000000.0, 1e9, 100.0);
  /// // (scale_correction) 0.02 = 20000000.0 / 1e9
  /// // (adjusted_value)   2.0 = 0.02 * 100.0
  /// // (value)            14142135.62373095 = 1.4142135623730951 / 100.0 * 1e9
  /// // 0.7 = 1.4142135623730951 / 2.0
  /// // 0.7 = 14142135.62373095 / 20000000.0
  /// assert_eq!(result, 14142135.62373095);
  ///
  pub fn adjusted_sqrt(value: f64, factor: f64, base: f64) -> f64 {
    if value == 0.0 || factor == 0.0 || base == 0.0 {
      return 0.0
    }
    let scale_correction = value / factor;
    let adjusted_value = scale_correction * base;
    sqrt(adjusted_value) / base * factor
  }
}