
//! Autogenerated weights for `pallet_network`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 42.0.0
//! DATE: 2025-02-09, STEPS: `5`, REPEAT: `2`, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! WORST CASE MAP SIZE: `1000000`
//! HOSTNAME: `Bob`, CPU: `11th Gen Intel(R) Core(TM) i7-11800H @ 2.30GHz`
//! WASM-EXECUTION: `Compiled`, CHAIN: `Some("dev")`, DB CACHE: `1024`

// Executed Command:
// ./target/release/solochain-template-node
// benchmark
// pallet
// --chain=dev
// --wasm-execution=compiled
// --pallet=pallet_network
// --extrinsic=*
// --steps=5
// --repeat=2
// --output=pallets/network/src/weights.rs
// --template
// ./.maintain/frame-weight-template.hbs

#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]
#![allow(missing_docs)]

use frame_support::{traits::Get, weights::{Weight, constants::RocksDbWeight}};
use core::marker::PhantomData;

/// Weight functions needed for `pallet_network`.
pub trait WeightInfo {
	fn do_deactivation_ledger(x: u32, d: u32, ) -> Weight;
}

/// Weights for `pallet_network` using the Substrate node and recommended hardware.
pub struct SubstrateWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for SubstrateWeight<T> {
	/// Storage: `Network::DeactivationLedger` (r:1 w:1)
	/// Proof: `Network::DeactivationLedger` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::SubnetsData` (r:65 w:0)
	/// Proof: `Network::SubnetsData` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::SubnetNodeRegistrationEpochs` (r:1 w:0)
	/// Proof: `Network::SubnetNodeRegistrationEpochs` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::MaxDeactivations` (r:1 w:0)
	/// Proof: `Network::MaxDeactivations` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::SubnetNodesData` (r:128 w:128)
	/// Proof: `Network::SubnetNodesData` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `x` is `[0, 64]`.
	/// The range of component `d` is `[0, 128]`.
	fn do_deactivation_ledger(x: u32, d: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `67770 + x * (64 ±0)`
		//  Estimated: `391960 + d * (198 ±61) + x * (3112 ±28)`
		// Minimum execution time: 7_722_000 picoseconds.
		Weight::from_parts(7_722_000, 391960)
			// Standard Error: 3_785_468
			.saturating_add(Weight::from_parts(17_994_109, 0).saturating_mul(x.into()))
			// Standard Error: 1_892_734
			.saturating_add(Weight::from_parts(5_478_598, 0).saturating_mul(d.into()))
			.saturating_add(T::DbWeight::get().reads(4_u64))
			.saturating_add(T::DbWeight::get().reads((2_u64).saturating_mul(x.into())))
			.saturating_add(T::DbWeight::get().reads((1_u64).saturating_mul(d.into())))
			.saturating_add(T::DbWeight::get().writes(1_u64))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(x.into())))
			.saturating_add(T::DbWeight::get().writes((1_u64).saturating_mul(d.into())))
			.saturating_add(Weight::from_parts(0, 198).saturating_mul(d.into()))
			.saturating_add(Weight::from_parts(0, 3112).saturating_mul(x.into()))
	}
}

// For backwards compatibility and tests.
impl WeightInfo for () {
	/// Storage: `Network::DeactivationLedger` (r:1 w:1)
	/// Proof: `Network::DeactivationLedger` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::SubnetsData` (r:65 w:0)
	/// Proof: `Network::SubnetsData` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// Storage: `Network::SubnetNodeRegistrationEpochs` (r:1 w:0)
	/// Proof: `Network::SubnetNodeRegistrationEpochs` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::MaxDeactivations` (r:1 w:0)
	/// Proof: `Network::MaxDeactivations` (`max_values`: Some(1), `max_size`: None, mode: `Measured`)
	/// Storage: `Network::SubnetNodesData` (r:128 w:128)
	/// Proof: `Network::SubnetNodesData` (`max_values`: None, `max_size`: None, mode: `Measured`)
	/// The range of component `x` is `[0, 64]`.
	/// The range of component `d` is `[0, 128]`.
	fn do_deactivation_ledger(x: u32, d: u32, ) -> Weight {
		// Proof Size summary in bytes:
		//  Measured:  `67770 + x * (64 ±0)`
		//  Estimated: `391960 + d * (198 ±61) + x * (3112 ±28)`
		// Minimum execution time: 7_722_000 picoseconds.
		Weight::from_parts(7_722_000, 391960)
			// Standard Error: 3_785_468
			.saturating_add(Weight::from_parts(17_994_109, 0).saturating_mul(x.into()))
			// Standard Error: 1_892_734
			.saturating_add(Weight::from_parts(5_478_598, 0).saturating_mul(d.into()))
			.saturating_add(RocksDbWeight::get().reads(4_u64))
			.saturating_add(RocksDbWeight::get().reads((2_u64).saturating_mul(x.into())))
			.saturating_add(RocksDbWeight::get().reads((1_u64).saturating_mul(d.into())))
			.saturating_add(RocksDbWeight::get().writes(1_u64))
			.saturating_add(RocksDbWeight::get().writes((1_u64).saturating_mul(x.into())))
			.saturating_add(RocksDbWeight::get().writes((1_u64).saturating_mul(d.into())))
			.saturating_add(Weight::from_parts(0, 198).saturating_mul(d.into()))
			.saturating_add(Weight::from_parts(0, 3112).saturating_mul(x.into()))
	}
}