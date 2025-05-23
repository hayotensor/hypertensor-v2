#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
include!(concat!(env!("OUT_DIR"), "/wasm_binary.rs"));

pub mod impls;
extern crate alloc;
use alloc::{vec, vec::Vec};
use pallet_grandpa::AuthorityId as GrandpaId;
use sp_api::impl_runtime_apis;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::{crypto::KeyTypeId, OpaqueMetadata};
use sp_runtime::{
	create_runtime_str, generic, impl_opaque_keys,
	traits::{BlakeTwo256, Block as BlockT, IdentifyAccount, NumberFor, One, Verify, IdentityLookup},
	transaction_validity::{TransactionSource, TransactionValidity},
	ApplyExtrinsicResult, MultiSignature,
	RuntimeDebug
};
use codec::{Encode, Decode, MaxEncodedLen};
use sp_runtime::traits::Get;
#[cfg(feature = "std")]
use sp_version::NativeVersion;
use sp_version::RuntimeVersion;
use alloc::collections::BTreeMap;
pub use frame_support::{
	construct_runtime, derive_impl, parameter_types, ord_parameter_types,
	traits::{
		fungible::HoldConsideration,
		tokens::{ConversionFromAssetBalance, PaymentStatus, Pay, PayFromAccount, UnityAssetBalanceConversion},
		ConstBool, ConstU128, ConstU32, ConstU64, ConstU8, KeyOwnerProofSystem,
		StorageInfo,
		InstanceFilter,
		VariantCountOf,
		EitherOfDiverse,
		EqualPrivilegeOnly,
		LinearStoragePrice,
	},
	weights::{
		constants::{
			BlockExecutionWeight, ExtrinsicBaseWeight, RocksDbWeight, WEIGHT_REF_TIME_PER_SECOND,
		},
		IdentityFee, Weight,
	},
	dispatch::DispatchClass,
	StorageValue,
	PalletId,
	genesis_builder_helper::{build_state, get_preset},
	storage::bounded_vec::BoundedVec,
};
use core::{cell::RefCell, marker::PhantomData};
pub use frame_system::{EnsureRoot, EnsureRootWithSuccess, EnsureWithSuccess};
pub use pallet_balances::Call as BalancesCall;
pub use frame_system::Call as SystemCall;
pub use pallet_timestamp::Call as TimestampCall;
use pallet_transaction_payment::{ConstFeeMultiplier, FungibleAdapter, Multiplier};
#[cfg(any(feature = "std", test))]
pub use sp_runtime::BuildStorage;
pub use sp_runtime::{Perbill, Permill};

use pallet_network::DefaultSubnetNodeUniqueParamLimit;

pub use pallet_network;
pub use pallet_rewards;

/// An index to a block.
pub type BlockNumber = u32;

/// Alias to 512-bit hash when used in the context of a transaction signature on the chain.
pub type Signature = MultiSignature;

pub type AccountPublic = <Signature as Verify>::Signer;

/// Some way of identifying an account on the chain. We intentionally make it equivalent
/// to the public key of our transaction signing scheme.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// Balance of an account.
pub type Balance = u128;

/// Index of a transaction in the chain.
pub type Nonce = u32;

/// A hash of some data used by the chain.
pub type Hash = sp_core::H256;

/// Opaque types. These are used by the CLI to instantiate machinery that don't need to know
/// the specifics of the runtime. They can then be made to be agnostic over specific formats
/// of data like extrinsics, allowing for them to continue syncing the network through upgrades
/// to even the core data structures.
pub mod opaque {
	use super::*;

	pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

	/// Opaque block header type.
	pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
	/// Opaque block type.
	pub type Block = generic::Block<Header, UncheckedExtrinsic>;
	/// Opaque block identifier type.
	pub type BlockId = generic::BlockId<Block>;

	impl_opaque_keys! {
		pub struct SessionKeys {
			pub aura: Aura,
			pub grandpa: Grandpa,
		}
	}
}

// To learn more about runtime versioning, see:
// https://docs.substrate.io/main-docs/build/upgrade#runtime-versioning
#[sp_version::runtime_version]
pub const VERSION: RuntimeVersion = RuntimeVersion {
	spec_name: create_runtime_str!("solochain-template-runtime"),
	impl_name: create_runtime_str!("solochain-template-runtime"),
	authoring_version: 1,
	// The version of the runtime specification. A full node will not attempt to use its native
	//   runtime in substitute for the on-chain Wasm runtime unless all of `spec_name`,
	//   `spec_version`, and `authoring_version` are the same between Wasm and native.
	// This value is set to 100 to notify Polkadot-JS App (https://polkadot.js.org/apps) to use
	//   the compatible custom types.
	spec_version: 100,
	impl_version: 1,
	apis: RUNTIME_API_VERSIONS,
	transaction_version: 1,
	state_version: 1,
};

/// This determines the average expected block time that we are targeting.
/// Blocks will be produced at a minimum duration defined by `SLOT_DURATION`.
/// `SLOT_DURATION` is picked up by `pallet_timestamp` which is in turn picked
/// up by `pallet_aura` to implement `fn slot_duration()`.
///
/// Change this to adjust the block time.
pub const MILLISECS_PER_BLOCK: u64 = 6000;

// NOTE: Currently it is not possible to change the slot duration after the chain has started.
//       Attempting to do so will brick block production.
pub const SLOT_DURATION: u64 = MILLISECS_PER_BLOCK;

// Time is measured by number of blocks.
// e.g. How many blocks in a minutes, hours, day, year
pub const MINUTES: BlockNumber = 60_000 / (MILLISECS_PER_BLOCK as BlockNumber); // 10
pub const HOURS: BlockNumber = MINUTES * 60; // 600
pub const DAYS: BlockNumber = HOURS * 24; // 14400
pub const YEAR: BlockNumber = DAYS * 365; // 5256000

// Rewards pallet variables
pub const BLOCKS_PER_HALVING: BlockNumber = YEAR * 2;
pub const TARGET_MAX_TOTAL_SUPPLY: u128 = 2_800_000_000_000_000_000_000_000;
pub const INITIAL_REWARD_PER_BLOCK: u128 = (TARGET_MAX_TOTAL_SUPPLY / 2) / BLOCKS_PER_HALVING as u128;

pub const SECS_PER_BLOCK: u32 = (MILLISECS_PER_BLOCK as BlockNumber) / 1000; // 6

// Blocks per epoch
pub const BLOCKS_PER_EPOCH: u32 = 10;
pub const EPOCHS_PER_YEAR: u32 = YEAR as u32 / BLOCKS_PER_EPOCH;

/// The version information used to identify this runtime when compiled natively.
#[cfg(feature = "std")]
pub fn native_version() -> NativeVersion {
	NativeVersion { runtime_version: VERSION, can_author_with: Default::default() }
}

const NORMAL_DISPATCH_RATIO: Perbill = Perbill::from_percent(75);

parameter_types! {
	pub const BlockHashCount: BlockNumber = 2400;
	pub const Version: RuntimeVersion = VERSION;
	/// We allow for 2 seconds of compute with a 6 second average block time.
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::with_sensible_defaults(
			Weight::from_parts(2u64 * WEIGHT_REF_TIME_PER_SECOND, u64::MAX),
			NORMAL_DISPATCH_RATIO,
		);
	pub BlockLength: frame_system::limits::BlockLength = frame_system::limits::BlockLength
		::max_with_normal_ratio(5 * 1024 * 1024, NORMAL_DISPATCH_RATIO);
	pub const SS58Prefix: u8 = 42;
}

/// The default types are being injected by [`derive_impl`](`frame_support::derive_impl`) from
/// [`SoloChainDefaultConfig`](`struct@frame_system::config_preludes::SolochainDefaultConfig`),
/// but overridden as needed.
#[derive_impl(frame_system::config_preludes::SolochainDefaultConfig)]
impl frame_system::Config for Runtime {
	/// The block type for the runtime.
	type Block = Block;
	/// Block & extrinsics weights: base values and limits.
	type BlockWeights = BlockWeights;
	/// The maximum length of a block (in bytes).
	type BlockLength = BlockLength;
	/// The identifier used to distinguish between accounts.
	type AccountId = AccountId;
	/// The type for storing how many extrinsics an account has signed.
	type Nonce = Nonce;
	/// The type for hashing blocks and tries.
	type Hash = Hash;
	/// Maximum number of block number to block hash mappings to keep (oldest pruned first).
	type BlockHashCount = BlockHashCount;
	/// The weight of database operations that the runtime can invoke.
	type DbWeight = RocksDbWeight;
	/// Version of the runtime.
	type Version = Version;
	/// The data to be stored in an account.
	type AccountData = pallet_balances::AccountData<Balance>;
	/// This is used as an identifier of the chain. 42 is the generic substrate prefix.
	type SS58Prefix = SS58Prefix;
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}

impl pallet_aura::Config for Runtime {
	type AuthorityId = AuraId;
	type DisabledValidators = ();
	type MaxAuthorities = ConstU32<32>;
	type AllowMultipleBlocksPerSlot = ConstBool<false>;
	type SlotDuration = pallet_aura::MinimumPeriodTimesTwo<Runtime>;
}

impl pallet_grandpa::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;

	type WeightInfo = ();
	type MaxAuthorities = ConstU32<32>;
	type MaxNominators = ConstU32<0>;
	type MaxSetIdSessionEntries = ConstU64<0>;

	type KeyOwnerProof = sp_core::Void;
	type EquivocationReportSystem = ();
}

impl pallet_timestamp::Config for Runtime {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = u64;
	type OnTimestampSet = Aura;
	type MinimumPeriod = ConstU64<{ SLOT_DURATION / 2 }>;
	type WeightInfo = ();
}

/// Existential deposit.
pub const EXISTENTIAL_DEPOSIT: u128 = 500;

impl pallet_balances::Config for Runtime {
	type MaxLocks = ConstU32<128>;
	type MaxReserves = ConstU32<128>;
	type ReserveIdentifier = [u8; 8];
	/// The type for recording an account's balance.
	type Balance = Balance;
	/// The ubiquitous event type.
	type RuntimeEvent = RuntimeEvent;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU128<EXISTENTIAL_DEPOSIT>;
	type AccountStore = System;
	type WeightInfo = pallet_balances::weights::SubstrateWeight<Runtime>;
	type FreezeIdentifier = RuntimeFreezeReason;
	// type MaxFreezes = VariantCountOf<RuntimeFreezeReason>;
	type MaxFreezes = ConstU32<50>;
	type RuntimeHoldReason = RuntimeHoldReason;
	type RuntimeFreezeReason = RuntimeHoldReason;
}

parameter_types! {
	pub FeeMultiplier: Multiplier = Multiplier::one();
}

impl pallet_transaction_payment::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type OnChargeTransaction = FungibleAdapter<Balances, ()>;
	type OperationalFeeMultiplier = ConstU8<5>;
	type WeightToFee = IdentityFee<Balance>;
	type LengthToFee = IdentityFee<Balance>;
	type FeeMultiplierUpdate = ConstFeeMultiplier<FeeMultiplier>;
}

impl pallet_sudo::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type WeightInfo = pallet_sudo::weights::SubstrateWeight<Runtime>;
}

parameter_types! {
	// One storage item; key size is 32; value is size 4+4+16+32 bytes = 56 bytes.
	pub const DepositBase: Balance = (1) as Balance * 2_000 * 10_000 + (88 as Balance) * 100 * 10_000;
	// Additional storage item size of 32 bytes.
	pub const DepositFactor: Balance = (0) as Balance * 2_000 * 10_000 + (32 as Balance) * 100 * 10_000;
	pub const MaxSignatories: u32 = 100;
}

impl pallet_multisig::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type DepositBase = DepositBase;
	type DepositFactor = DepositFactor;
	type MaxSignatories = MaxSignatories;
	type WeightInfo = pallet_multisig::weights::SubstrateWeight<Runtime>;
}

impl pallet_insecure_randomness_collective_flip::Config for Runtime {}

impl pallet_utility::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type PalletsOrigin = OriginCaller;
	type WeightInfo = pallet_utility::weights::SubstrateWeight<Runtime>;
}

pub const fn deposit(items: u32, bytes: u32) -> Balance {
	pub const ITEMS_FEE: Balance = 2_000 * 10_000;
	pub const BYTES_FEE: Balance = 100 * 10_000;
	(items as Balance)
			.saturating_mul(ITEMS_FEE)
			.saturating_add((bytes as Balance).saturating_mul(BYTES_FEE))
}

parameter_types! {
	// One storage item; key size 32, value size 8; .
	pub const ProxyDepositBase: Balance = deposit(1, 8);
	// Additional storage item size of 33 bytes.
	pub const ProxyDepositFactor: Balance = deposit(0, 33);
	pub const AnnouncementDepositBase: Balance = deposit(1, 8);
	pub const AnnouncementDepositFactor: Balance = deposit(0, 66);
}

/// The type used to represent the kinds of proxying allowed.
#[derive(
	Copy,
	Clone,
	Eq,
	PartialEq,
	Ord,
	PartialOrd,
	Encode,
	Decode,
	RuntimeDebug,
	MaxEncodedLen,
	scale_info::TypeInfo,
)]
pub enum ProxyType {
	Any,
	DelegateStaking,
	NonTransfer,
	// Governance,
	// Staking,
}
impl Default for ProxyType {
	fn default() -> Self {
		Self::Any
	}
}
impl InstanceFilter<RuntimeCall> for ProxyType {
	fn filter(&self, c: &RuntimeCall) -> bool {
		match self {
			ProxyType::Any => true,
			ProxyType::NonTransfer => !matches!(
				c,
				RuntimeCall::Balances(..)
			),
			ProxyType::DelegateStaking => matches!(
				c,
				RuntimeCall::Network(pallet_network::Call::add_to_delegate_stake { .. })
				| RuntimeCall::Network(pallet_network::Call::transfer_delegate_stake { .. })
				| RuntimeCall::Network(pallet_network::Call::remove_delegate_stake { .. })
			),
			// ProxyType::NonTransfer => !matches!(
			// 	c,
			// 	RuntimeCall::Balances(..) | 
			// 	RuntimeCall::Network(pallet_network::Call::add_to_delegate_stake { .. })  |
			// 	RuntimeCall::Network(pallet_network::Call::remove_delegate_stake { .. })  |
			// 	RuntimeCall::Network(pallet_network::Call::transfer_delegate_stake { .. })
			// ),
			// ProxyType::Governance => matches!(
			// 	c,
			// 	RuntimeCall::Democracy(..) |
			// 		RuntimeCall::Council(..) |
			// 		RuntimeCall::Society(..) |
			// 		RuntimeCall::TechnicalCommittee(..) |
			// 		RuntimeCall::Elections(..) |
			// 		RuntimeCall::Treasury(..)
			// ),
			// ProxyType::Staking => matches!(
			// 	c,
			// 	RuntimeCall::Network(pallet_network::Call::add_to_delegate_stake { .. })  |
			// 	RuntimeCall::Network(pallet_network::Call::remove_delegate_stake { .. })  |
			// 	RuntimeCall::Network(pallet_network::Call::transfer_delegate_stake { .. })
			// ),
			// ProxyType::Staking => {
			// 	matches!(c, RuntimeCall::Staking(..) | RuntimeCall::FastUnstake(..))
			// },
		}
	}
	fn is_superset(&self, o: &Self) -> bool {
		match (self, o) {
			(x, y) if x == y => true,
			(ProxyType::Any, _) => true,
			(_, ProxyType::Any) => false,
			// (ProxyType::NonTransfer, _) => true,
			_ => false,
		}
	}
}

impl pallet_proxy::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type Currency = Balances;
	type ProxyType = ProxyType;
	type ProxyDepositBase = ProxyDepositBase;
	type ProxyDepositFactor = ProxyDepositFactor;
	type MaxProxies = ConstU32<32>;
	type WeightInfo = pallet_proxy::weights::SubstrateWeight<Runtime>;
	type MaxPending = ConstU32<32>;
	type CallHasher = BlakeTwo256;
	type AnnouncementDepositBase = AnnouncementDepositBase;
	type AnnouncementDepositFactor = AnnouncementDepositFactor;
}

parameter_types! {
	pub const PreimageBaseDeposit: Balance = deposit(2, 64);
	pub const PreimageByteDeposit: Balance = deposit(0, 1);
	pub const PreimageHoldReason: RuntimeHoldReason = RuntimeHoldReason::Preimage(pallet_preimage::HoldReason::Preimage);
}

impl pallet_preimage::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = pallet_preimage::weights::SubstrateWeight<Runtime>;
	type Currency = Balances;
	type ManagerOrigin = EnsureRoot<AccountId>;
	type Consideration = HoldConsideration<
		AccountId,
		Balances,
		PreimageHoldReason,
		LinearStoragePrice<PreimageBaseDeposit, PreimageByteDeposit, Balance>,
	>;
}

parameter_types! {
	pub MaximumSchedulerWeight: Weight = Perbill::from_percent(80) *
		BlockWeights::get().max_block;
}

impl pallet_scheduler::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeOrigin = RuntimeOrigin;
	type PalletsOrigin = OriginCaller;
	type RuntimeCall = RuntimeCall;
	type MaximumWeight = MaximumSchedulerWeight;
	type ScheduleOrigin = EnsureRoot<AccountId>;
	type MaxScheduledPerBlock = ConstU32<100>;
	type WeightInfo = pallet_scheduler::weights::SubstrateWeight<Runtime>;
	type OriginPrivilegeCmp = EqualPrivilegeOnly;
	type Preimages = Preimage;
}

// parameter_types! {
//   pub const MaxWellKnownNodes: u32 = 8;
//   pub const MaxPeerIdLength: u32 = 128;
// }

// impl pallet_node_authorization::Config for Runtime {
// 	type RuntimeEvent = RuntimeEvent;
// 	type MaxWellKnownNodes = MaxWellKnownNodes;
// 	type MaxPeerIdLength = MaxPeerIdLength;
// 	type AddOrigin = EnsureRoot<AccountId>;
// 	type RemoveOrigin = EnsureRoot<AccountId>;
// 	type SwapOrigin = EnsureRoot<AccountId>;
// 	type ResetOrigin = EnsureRoot<AccountId>;
// 	type WeightInfo = ();
// }


// /// We assume that ~10% of the block weight is consumed by `on_initialize` handlers.
// /// This is used to limit the maximal weight of a single extrinsic.
// const AVERAGE_ON_INITIALIZE_RATIO: Perbill = Perbill::from_percent(10);
// /// We allow for 2 seconds of compute with a 6 second average block time, with maximum proof size.
// const MAXIMUM_BLOCK_WEIGHT: Weight =
// 	Weight::from_parts(WEIGHT_REF_TIME_PER_SECOND.saturating_mul(2), u64::MAX);

parameter_types! {
	pub const CouncilMotionDuration: BlockNumber = 5 * DAYS;
	pub const CouncilMaxProposals: u32 = 100;
	pub const CouncilMaxMembers: u32 = 100;
	pub MaxCollectivesProposalWeight: Weight = Perbill::from_percent(50) * BlockWeights::get().max_block;
}

type CouncilCollective = pallet_collective::Instance1;
impl pallet_collective::Config<CouncilCollective> for Runtime {
	type RuntimeOrigin = RuntimeOrigin;
	type Proposal = RuntimeCall;
	type RuntimeEvent = RuntimeEvent;
	type MotionDuration = CouncilMotionDuration;
	type MaxProposals = CouncilMaxProposals;
	type MaxMembers = CouncilMaxMembers;
	type DefaultVote = pallet_collective::PrimeDefaultVote;
	type WeightInfo = pallet_collective::weights::SubstrateWeight<Runtime>;
	type SetMembersOrigin = EnsureRoot<AccountId>;
	type MaxProposalWeight = MaxCollectivesProposalWeight;
}

parameter_types! {
	pub const InitialTxRateLimit: u32 = 0;
	pub const EpochLength: u32 = BLOCKS_PER_EPOCH; // Testnet 600 blocks per erpoch / 69 mins per epoch, Local 10
	pub const EpochsPerYear: u32 = EPOCHS_PER_YEAR; // Testnet 600 blocks per erpoch / 69 mins per epoch, Local 10
	pub const NetworkPalletId: PalletId = PalletId(*b"/network");
	pub const MinProposalStake: u128 = 1_000_000_000_000_000_000; // 1 * 1e18
	pub const DelegateStakeCooldownEpochs: u32 = 100;
	pub const NodeDelegateStakeCooldownEpochs: u32 = 100;
	pub const StakeCooldownEpochs: u32 = 100;
	pub const DelegateStakeEpochsRemovalWindow: u32 = 10;
	pub const MaxDelegateStakeUnlockings: u32 = 32;
	pub const MaxStakeUnlockings: u32 = 32;
}

impl pallet_network::Config for Runtime {
	type WeightInfo = ();
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type MajorityCollectiveOrigin = pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 2, 3>;
	type SuperMajorityCollectiveOrigin = pallet_collective::EnsureProportionAtLeast<AccountId, CouncilCollective, 4, 5>;
	type EpochLength = EpochLength;
	type EpochsPerYear = EpochsPerYear;
	type StringLimit = ConstU32<12288>;
	type InitialTxRateLimit = InitialTxRateLimit;
// 	type OffchainSignature = Signature;
// 	type OffchainPublic = AccountPublic;
	type PalletId = NetworkPalletId;
  type DelegateStakeCooldownEpochs = DelegateStakeCooldownEpochs;
	type NodeDelegateStakeCooldownEpochs = NodeDelegateStakeCooldownEpochs;
	type DelegateStakeEpochsRemovalWindow = DelegateStakeEpochsRemovalWindow;
	type MaxDelegateStakeUnlockings = MaxDelegateStakeUnlockings;
	type MaxStakeUnlockings = MaxStakeUnlockings;
	type StakeCooldownEpochs = StakeCooldownEpochs;
	type Randomness = InsecureRandomnessCollectiveFlip;
	type MinProposalStake = MinProposalStake;
	type TreasuryAccount = TreasuryAccount;
}

pub struct AuraAccountAdapter;
impl frame_support::traits::FindAuthor<AccountId> for AuraAccountAdapter {
	fn find_author<'a, I>(digests: I) -> Option<AccountId>
		where I: 'a + IntoIterator<Item=(frame_support::ConsensusEngineId, &'a [u8])>
	{
		pallet_aura::AuraAuthorId::<Runtime>::find_author(digests).and_then(|k| {
			AccountId::try_from(k.as_ref()).ok()
		})
	}
}

impl pallet_authorship::Config for Runtime {
	type FindAuthor = AuraAccountAdapter;
	type EventHandler =  ();
}

parameter_types! {
	pub const MaxNameLen: u32 = 50;
}

impl pallet_tx_pause::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeCall = RuntimeCall;
	type PauseOrigin = EnsureRoot<AccountId>;
	type UnpauseOrigin = EnsureRoot<AccountId>;
	type WhitelistedCalls = ();
	type MaxNameLen = MaxNameLen;
	type WeightInfo = ();
}

parameter_types! {
	pub const HalvingInterval: BlockNumber = BLOCKS_PER_HALVING;
	pub const InitialBlockSubsidy: u128 = INITIAL_REWARD_PER_BLOCK;
}

impl pallet_rewards::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type FindAuthor = AuraAccountAdapter;
	type HalvingInterval = HalvingInterval;
	type InitialBlockSubsidy = InitialBlockSubsidy;
	type IncreaseStakeVault = Network;
}

impl pallet_atomic_swap::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type SwapAction = pallet_atomic_swap::BalanceSwapAction<AccountId, Balances>;
	type ProofLimit = ConstU32<1024>;
}

parameter_types! {
	pub const Burn: Permill = Permill::from_percent(50);
	pub const TreasuryPalletId: PalletId = PalletId(*b"py/trsry");
	pub const SpendLimit: Balance = u128::MAX;
	pub TreasuryAccount: AccountId = Treasury::account_id();
}

impl pallet_treasury::Config for Runtime {
	type PalletId = TreasuryPalletId;
	type Currency = Balances;
	type RejectOrigin = EnsureRoot<AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type SpendPeriod = ConstU32<2>;
	type Burn = Burn;
	type BurnDestination = (); // Just gets burned.
	type WeightInfo = ();
	type SpendFunds = ();
	type MaxApprovals = ConstU32<100>;
	type SpendOrigin = EnsureRootWithSuccess<AccountId, SpendLimit>;
	type AssetKind = ();
	type Beneficiary = AccountId;
	type BeneficiaryLookup = IdentityLookup<Self::Beneficiary>;
	type Paymaster = PayFromAccount<Balances, TreasuryAccount>;
	type BalanceConverter = UnityAssetBalanceConversion;
	type PayoutPeriod = ConstU32<10>;
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}

// Create the runtime by composing the FRAME pallets that were previously configured.
#[frame_support::runtime]
mod runtime {
	#[runtime::runtime]
	#[runtime::derive(
		RuntimeCall,
		RuntimeEvent,
		RuntimeError,
		RuntimeOrigin,
		RuntimeFreezeReason,
		RuntimeHoldReason,
		RuntimeSlashReason,
		RuntimeLockId,
		RuntimeTask
	)]
	pub struct Runtime;

	#[runtime::pallet_index(0)]
	pub type System = frame_system;

	#[runtime::pallet_index(1)]
	pub type Timestamp = pallet_timestamp;

	#[runtime::pallet_index(2)]
	pub type Aura = pallet_aura;

	#[runtime::pallet_index(3)]
	pub type Grandpa = pallet_grandpa;

	#[runtime::pallet_index(4)]
	pub type Balances = pallet_balances;

	#[runtime::pallet_index(5)]
	pub type TransactionPayment = pallet_transaction_payment;

	#[runtime::pallet_index(6)]
	pub type Sudo = pallet_sudo;

	#[runtime::pallet_index(7)]
	pub type Authorship = pallet_authorship;

	#[runtime::pallet_index(8)]
	pub type Multisig = pallet_multisig;

	#[runtime::pallet_index(9)]
	pub type InsecureRandomnessCollectiveFlip = pallet_insecure_randomness_collective_flip;

	#[runtime::pallet_index(10)]
	pub type Network = pallet_network;

	#[runtime::pallet_index(11)]
	pub type Rewards = pallet_rewards;

	#[runtime::pallet_index(12)]
	pub type Utility = pallet_utility;

	#[runtime::pallet_index(13)]
	pub type Proxy = pallet_proxy;

	#[runtime::pallet_index(14)]
	pub type Preimage = pallet_preimage;

	#[runtime::pallet_index(15)]
	pub type Scheduler = pallet_scheduler;

	#[runtime::pallet_index(16)]
	pub type Collective = pallet_collective::Pallet<Runtime, Instance1>;

	#[runtime::pallet_index(17)]
	pub type AtomicSwap = pallet_atomic_swap;

	// #[runtime::pallet_index(18)]
	// pub type NodeAuthorization = pallet_node_authorization;

	#[runtime::pallet_index(18)]
	pub type Treasury = pallet_treasury;

	#[runtime::pallet_index(19)]
	pub type TxPause = pallet_tx_pause;	
}

/// The address format for describing accounts.
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;
/// Block header type as expected by this runtime.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type as expected by this runtime.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// The SignedExtension to the basic transaction logic.
pub type SignedExtra = (
	frame_system::CheckNonZeroSender<Runtime>,
	frame_system::CheckSpecVersion<Runtime>,
	frame_system::CheckTxVersion<Runtime>,
	frame_system::CheckGenesis<Runtime>,
	frame_system::CheckEra<Runtime>,
	frame_system::CheckNonce<Runtime>,
	frame_system::CheckWeight<Runtime>,
	pallet_transaction_payment::ChargeTransactionPayment<Runtime>,
);

/// All migrations of the runtime, aside from the ones declared in the pallets.
///
/// This can be a tuple of types, each implementing `OnRuntimeUpgrade`.
#[allow(unused_parens)]
type Migrations = ();

/// Unchecked extrinsic type as expected by this runtime.
pub type UncheckedExtrinsic =
	generic::UncheckedExtrinsic<Address, RuntimeCall, Signature, SignedExtra>;
/// The payload being signed in transactions.
pub type SignedPayload = generic::SignedPayload<RuntimeCall, SignedExtra>;
/// Executive: handles dispatch to the various modules.
pub type Executive = frame_executive::Executive<
	Runtime,
	Block,
	frame_system::ChainContext<Runtime>,
	Runtime,
	AllPalletsWithSystem,
	Migrations,
>;

#[cfg(feature = "runtime-benchmarks")]
mod benches {
	frame_benchmarking::define_benchmarks!(
		[frame_benchmarking, BaselineBench::<Runtime>]
		[frame_system, SystemBench::<Runtime>]
		[pallet_balances, Balances]
		[pallet_timestamp, Timestamp]
		[pallet_sudo, Sudo]
		[pallet_network, Network]
		[pallet_collective, Collective]
		[pallet_treasury, Treasury]
	);
}

impl_runtime_apis! {
	impl sp_api::Core<Block> for Runtime {
		fn version() -> RuntimeVersion {
			VERSION
		}

		fn execute_block(block: Block) {
			Executive::execute_block(block);
		}

		fn initialize_block(header: &<Block as BlockT>::Header) -> sp_runtime::ExtrinsicInclusionMode {
			Executive::initialize_block(header)
		}
	}

	impl sp_api::Metadata<Block> for Runtime {
		fn metadata() -> OpaqueMetadata {
			OpaqueMetadata::new(Runtime::metadata().into())
		}

		fn metadata_at_version(version: u32) -> Option<OpaqueMetadata> {
			Runtime::metadata_at_version(version)
		}

		fn metadata_versions() -> Vec<u32> {
			Runtime::metadata_versions()
		}
	}

	impl sp_block_builder::BlockBuilder<Block> for Runtime {
		fn apply_extrinsic(extrinsic: <Block as BlockT>::Extrinsic) -> ApplyExtrinsicResult {
			Executive::apply_extrinsic(extrinsic)
		}

		fn finalize_block() -> <Block as BlockT>::Header {
			Executive::finalize_block()
		}

		fn inherent_extrinsics(data: sp_inherents::InherentData) -> Vec<<Block as BlockT>::Extrinsic> {
			data.create_extrinsics()
		}

		fn check_inherents(
			block: Block,
			data: sp_inherents::InherentData,
		) -> sp_inherents::CheckInherentsResult {
			data.check_extrinsics(&block)
		}
	}

	impl sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block> for Runtime {
		fn validate_transaction(
			source: TransactionSource,
			tx: <Block as BlockT>::Extrinsic,
			block_hash: <Block as BlockT>::Hash,
		) -> TransactionValidity {
			Executive::validate_transaction(source, tx, block_hash)
		}
	}

	impl sp_offchain::OffchainWorkerApi<Block> for Runtime {
		fn offchain_worker(header: &<Block as BlockT>::Header) {
			Executive::offchain_worker(header)
		}
	}

	impl sp_consensus_aura::AuraApi<Block, AuraId> for Runtime {
		fn slot_duration() -> sp_consensus_aura::SlotDuration {
			sp_consensus_aura::SlotDuration::from_millis(Aura::slot_duration())
		}

		fn authorities() -> Vec<AuraId> {
			pallet_aura::Authorities::<Runtime>::get().into_inner()
		}
	}

	impl sp_session::SessionKeys<Block> for Runtime {
		fn generate_session_keys(seed: Option<Vec<u8>>) -> Vec<u8> {
			opaque::SessionKeys::generate(seed)
		}

		fn decode_session_keys(
			encoded: Vec<u8>,
		) -> Option<Vec<(Vec<u8>, KeyTypeId)>> {
			opaque::SessionKeys::decode_into_raw_public_keys(&encoded)
		}
	}

	impl sp_consensus_grandpa::GrandpaApi<Block> for Runtime {
		fn grandpa_authorities() -> sp_consensus_grandpa::AuthorityList {
			Grandpa::grandpa_authorities()
		}

		fn current_set_id() -> sp_consensus_grandpa::SetId {
			Grandpa::current_set_id()
		}

		fn submit_report_equivocation_unsigned_extrinsic(
			_equivocation_proof: sp_consensus_grandpa::EquivocationProof<
				<Block as BlockT>::Hash,
				NumberFor<Block>,
			>,
			_key_owner_proof: sp_consensus_grandpa::OpaqueKeyOwnershipProof,
		) -> Option<()> {
			None
		}

		fn generate_key_ownership_proof(
			_set_id: sp_consensus_grandpa::SetId,
			_authority_id: GrandpaId,
		) -> Option<sp_consensus_grandpa::OpaqueKeyOwnershipProof> {
			// NOTE: this is the only implementation possible since we've
			// defined our key owner proof type as a bottom type (i.e. a type
			// with no values).
			None
		}
	}

	impl frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce> for Runtime {
		fn account_nonce(account: AccountId) -> Nonce {
			System::account_nonce(account)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance> for Runtime {
		fn query_info(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_info(uxt, len)
		}
		fn query_fee_details(
			uxt: <Block as BlockT>::Extrinsic,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_fee_details(uxt, len)
		}
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	impl pallet_transaction_payment_rpc_runtime_api::TransactionPaymentCallApi<Block, Balance, RuntimeCall>
		for Runtime
	{
		fn query_call_info(
			call: RuntimeCall,
			len: u32,
		) -> pallet_transaction_payment::RuntimeDispatchInfo<Balance> {
			TransactionPayment::query_call_info(call, len)
		}
		fn query_call_fee_details(
			call: RuntimeCall,
			len: u32,
		) -> pallet_transaction_payment::FeeDetails<Balance> {
			TransactionPayment::query_call_fee_details(call, len)
		}
		fn query_weight_to_fee(weight: Weight) -> Balance {
			TransactionPayment::weight_to_fee(weight)
		}
		fn query_length_to_fee(length: u32) -> Balance {
			TransactionPayment::length_to_fee(length)
		}
	}

	impl network_custom_rpc_runtime_api::NetworkRuntimeApi<Block> for Runtime {
		fn get_subnet_nodes(subnet_id: u32) -> Vec<u8> {
			let result = Network::get_subnet_nodes(subnet_id);
			result.encode()
		}
		fn get_subnet_nodes_included(subnet_id: u32) -> Vec<u8> {
			let result = Network::get_subnet_nodes_included(subnet_id);
			result.encode()
		}
		fn get_subnet_nodes_submittable(subnet_id: u32) -> Vec<u8> {
			let result = Network::get_subnet_nodes_submittable(subnet_id);
			result.encode()
		}
		fn get_subnet_nodes_subnet_unconfirmed_count(subnet_id: u32) -> u32 {
			let result = Network::get_subnet_nodes_subnet_unconfirmed_count(subnet_id);
			result
		}
		fn get_consensus_data(subnet_id: u32, epoch: u32) -> Vec<u8> {
			let result = Network::get_consensus_data(subnet_id, epoch);
			result.encode()
		}
		fn get_minimum_subnet_nodes(memory_mb: u128) -> u32 {
			let result = Network::get_minimum_subnet_nodes(memory_mb);
			result
		}
		fn get_minimum_delegate_stake(memory_mb: u128) -> u128 {
			let result = Network::get_minimum_delegate_stake(memory_mb);
			result
		}
		fn get_subnet_node_info(subnet_id: u32) -> Vec<u8> {
			let result = Network::get_subnet_node_info(subnet_id);
			result.encode()
		}
		fn is_subnet_node_by_peer_id(subnet_id: u32, peer_id: Vec<u8>) -> bool {
			let result = Network::is_subnet_node_by_peer_id(subnet_id, peer_id);
			result
		}	
		fn are_subnet_nodes_by_peer_id(subnet_id: u32, peer_ids: Vec<Vec<u8>>) -> Vec<u8> {
			let result = Network::are_subnet_nodes_by_peer_id(subnet_id, peer_ids);
			result.encode()
		}
		fn is_subnet_node_by_a(subnet_id: u32, a: BoundedVec<u8, DefaultSubnetNodeUniqueParamLimit>) -> bool {
			let result = Network::is_subnet_node_by_a(subnet_id, a);
			result
		}
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl frame_benchmarking::Benchmark<Block> for Runtime {
		fn benchmark_metadata(extra: bool) -> (
			Vec<frame_benchmarking::BenchmarkList>,
			Vec<frame_support::traits::StorageInfo>,
		) {
			use frame_benchmarking::{baseline, Benchmarking, BenchmarkList};
			use frame_support::traits::StorageInfoTrait;
			use frame_system_benchmarking::Pallet as SystemBench;
			use baseline::Pallet as BaselineBench;

			let mut list = Vec::<BenchmarkList>::new();
			list_benchmarks!(list, extra);

			let storage_info = AllPalletsWithSystem::storage_info();

			(list, storage_info)
		}

		fn dispatch_benchmark(
			config: frame_benchmarking::BenchmarkConfig
		) -> Result<Vec<frame_benchmarking::BenchmarkBatch>, sp_runtime::RuntimeString> {
			use frame_benchmarking::{baseline, Benchmarking, BenchmarkBatch};
			use sp_storage::TrackedStorageKey;
			use frame_system_benchmarking::Pallet as SystemBench;
			use baseline::Pallet as BaselineBench;

			impl frame_system_benchmarking::Config for Runtime {}
			impl baseline::Config for Runtime {}

			use frame_support::traits::WhitelistedStorageKeys;
			let whitelist: Vec<TrackedStorageKey> = AllPalletsWithSystem::whitelisted_storage_keys();

			let mut batches = Vec::<BenchmarkBatch>::new();
			let params = (&config, &whitelist);
			add_benchmarks!(params, batches);

			Ok(batches)
		}
	}

	#[cfg(feature = "try-runtime")]
	impl frame_try_runtime::TryRuntime<Block> for Runtime {
		fn on_runtime_upgrade(checks: frame_try_runtime::UpgradeCheckSelect) -> (Weight, Weight) {
			// NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
			// have a backtrace here. If any of the pre/post migration checks fail, we shall stop
			// right here and right now.
			let weight = Executive::try_runtime_upgrade(checks).unwrap();
			(weight, BlockWeights::get().max_block)
		}

		fn execute_block(
			block: Block,
			state_root_check: bool,
			signature_check: bool,
			select: frame_try_runtime::TryStateSelect
		) -> Weight {
			// NOTE: intentional unwrap: we don't want to propagate the error backwards, and want to
			// have a backtrace here.
			Executive::try_execute_block(block, state_root_check, signature_check, select).expect("execute-block failed")
		}
	}

	impl sp_genesis_builder::GenesisBuilder<Block> for Runtime {
		fn build_state(config: Vec<u8>) -> sp_genesis_builder::Result {
			build_state::<RuntimeGenesisConfig>(config)
		}

		fn get_preset(id: &Option<sp_genesis_builder::PresetId>) -> Option<Vec<u8>> {
			get_preset::<RuntimeGenesisConfig>(id, |_| None)
		}

		fn preset_names() -> Vec<sp_genesis_builder::PresetId> {
			vec![]
		}
	}
}
