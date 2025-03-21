// This file is part of Substrate.

// Copyright (C) Parity Technologies (UK) Ltd.
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

#![cfg(test)]

use super::*;
use crate as pallet_atomic_swap;

use frame_support::{derive_impl, traits::ConstU32};
use sp_runtime::BuildStorage;

type Block = frame_system::mocking::MockBlock<Test>;

frame_support::construct_runtime!(
	pub enum Test
	{
		System: frame_system,
		Balances: pallet_balances,
		AtomicSwap: pallet_atomic_swap,
	}
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type Block = Block;
	type AccountData = pallet_balances::AccountData<u64>;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
	type AccountStore = System;
}

impl Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type SwapAction = BalanceSwapAction<u64, Balances>;
	type ProofLimit = ConstU32<1024>;
}

const A: u64 = 1;
const B: u64 = 2;

pub fn new_test_ext() -> sp_io::TestExternalities {
	let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
	let genesis = pallet_balances::GenesisConfig::<Test> { balances: vec![(A, 100), (B, 200)] };
	genesis.assimilate_storage(&mut t).unwrap();
	t.into()
}

#[test]
fn two_party_successful_swap_blake2_256() {
	let mut chain1 = new_test_ext();
	let mut chain2 = new_test_ext();

	// A generates a random proof. Keep it secret.
	let proof: [u8; 2] = [4, 2];
	// The hashed proof is the blake2_256 hash of the proof. This is public.
	let hashed_proof = blake2_256(&proof);

	// A creates the swap on chain1.
	chain1.execute_with(|| {
		AtomicSwap::create_swap(
			RuntimeOrigin::signed(A),
			B,
			hashed_proof,
			HashType::Blake2_256,
			BalanceSwapAction::new(50),
			1000,
		)
		.unwrap();

		assert_eq!(Balances::free_balance(A), 100 - 50);
		assert_eq!(Balances::free_balance(B), 200);
	});

	// B creates the swap on chain2.
	chain2.execute_with(|| {
		AtomicSwap::create_swap(
			RuntimeOrigin::signed(B),
			A,
			hashed_proof,
			HashType::Blake2_256,
			BalanceSwapAction::new(75),
			1000,
		)
		.unwrap();

		assert_eq!(Balances::free_balance(A), 100);
		assert_eq!(Balances::free_balance(B), 200 - 75);
	});

	// A reveals the proof and claims the swap on chain2.
	chain2.execute_with(|| {
		AtomicSwap::claim_swap(
			RuntimeOrigin::signed(A),
			proof.to_vec(),
			HashType::Blake2_256,
			BalanceSwapAction::new(75),
		)
		.unwrap();

		assert_eq!(Balances::free_balance(A), 100 + 75);
		assert_eq!(Balances::free_balance(B), 200 - 75);
	});

	// B use the revealed proof to claim the swap on chain1.
	chain1.execute_with(|| {
		AtomicSwap::claim_swap(
			RuntimeOrigin::signed(B),
			proof.to_vec(),
			HashType::Blake2_256,
			BalanceSwapAction::new(50),
		)
		.unwrap();

		assert_eq!(Balances::free_balance(A), 100 - 50);
		assert_eq!(Balances::free_balance(B), 200 + 50);
	});
}

#[test]
fn two_party_successful_swap_keccak_256() {
	let mut chain1 = new_test_ext();
	let mut chain2 = new_test_ext();

	// A generates a random proof. Keep it secret.
	let proof: [u8; 2] = [4, 2];
	// The hashed proof is the keccak_256 hash of the proof. This is public.
	let hashed_proof = keccak_256(&proof);

	// A creates the swap on chain1.
	chain1.execute_with(|| {
		AtomicSwap::create_swap(
			RuntimeOrigin::signed(A),
			B,
			hashed_proof,
			HashType::Keccak_256,
			BalanceSwapAction::new(50),
			1000,
		)
		.unwrap();

		assert_eq!(Balances::free_balance(A), 100 - 50);
		assert_eq!(Balances::free_balance(B), 200);
	});

	// B creates the swap on chain2.
	chain2.execute_with(|| {
		AtomicSwap::create_swap(
			RuntimeOrigin::signed(B),
			A,
			hashed_proof,
			HashType::Keccak_256,
			BalanceSwapAction::new(75),
			1000,
		)
		.unwrap();

		assert_eq!(Balances::free_balance(A), 100);
		assert_eq!(Balances::free_balance(B), 200 - 75);
	});

	// A reveals the proof and claims the swap on chain2.
	chain2.execute_with(|| {
		AtomicSwap::claim_swap(
			RuntimeOrigin::signed(A),
			proof.to_vec(),
			HashType::Keccak_256,
			BalanceSwapAction::new(75),
		)
		.unwrap();

		assert_eq!(Balances::free_balance(A), 100 + 75);
		assert_eq!(Balances::free_balance(B), 200 - 75);
	});

	// B use the revealed proof to claim the swap on chain1.
	chain1.execute_with(|| {
		AtomicSwap::claim_swap(
			RuntimeOrigin::signed(B),
			proof.to_vec(),
			HashType::Keccak_256,
			BalanceSwapAction::new(50),
		)
		.unwrap();

		assert_eq!(Balances::free_balance(A), 100 - 50);
		assert_eq!(Balances::free_balance(B), 200 + 50);
	});
}

#[test]
fn two_party_successful_swap_sha2_256() {
	let mut chain1 = new_test_ext();
	let mut chain2 = new_test_ext();

	// A generates a random proof. Keep it secret.
	let proof: [u8; 2] = [4, 2];
	// The hashed proof is the sha2_256 hash of the proof. This is public.
	let hashed_proof = sha2_256(&proof);

	// A creates the swap on chain1.
	chain1.execute_with(|| {
		AtomicSwap::create_swap(
			RuntimeOrigin::signed(A),
			B,
			hashed_proof,
			HashType::Sha2_256,
			BalanceSwapAction::new(50),
			1000,
		)
		.unwrap();

		assert_eq!(Balances::free_balance(A), 100 - 50);
		assert_eq!(Balances::free_balance(B), 200);
	});

	// B creates the swap on chain2.
	chain2.execute_with(|| {
		AtomicSwap::create_swap(
			RuntimeOrigin::signed(B),
			A,
			hashed_proof,
			HashType::Sha2_256,
			BalanceSwapAction::new(75),
			1000,
		)
		.unwrap();

		assert_eq!(Balances::free_balance(A), 100);
		assert_eq!(Balances::free_balance(B), 200 - 75);
	});

	// A reveals the proof and claims the swap on chain2.
	chain2.execute_with(|| {
		AtomicSwap::claim_swap(
			RuntimeOrigin::signed(A),
			proof.to_vec(),
			HashType::Sha2_256,
			BalanceSwapAction::new(75),
		)
		.unwrap();

		assert_eq!(Balances::free_balance(A), 100 + 75);
		assert_eq!(Balances::free_balance(B), 200 - 75);
	});

	// B use the revealed proof to claim the swap on chain1.
	chain1.execute_with(|| {
		AtomicSwap::claim_swap(
			RuntimeOrigin::signed(B),
			proof.to_vec(),
			HashType::Sha2_256,
			BalanceSwapAction::new(50),
		)
		.unwrap();

		assert_eq!(Balances::free_balance(A), 100 - 50);
		assert_eq!(Balances::free_balance(B), 200 + 50);
	});
}