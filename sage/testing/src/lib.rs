// Copyright (c) 2019 Alain Brenzikofer
// This file is part of Encointer
//
// Encointer is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Encointer is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Encointer.  If not, see <http://www.gnu.org/licenses/>.

//! Helpers to easily create a mocked runtime that is able to host the complete SAGE stack.
//!
//! You still need the following boilerplate code in the test suite, which can unfortunately not be
//! avoided as we need to get some types from there.
//!
//! ```rust
//! use sage_testing::{impl_ajuna_seasons, impl_core_pallets, impl_pallet_sage, impl_test_runtime_sage_api, TestRandomness};
//! use sp_core::H256;
//!
//! // The game logic implemented by the game dev
//! use example_game::{GameAssetId, GameAsset, GameTransition, TransitionConfig};
//!
//! type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<TestRuntime>;
//!
//! frame_support::construct_runtime!(
//!    pub enum TestRuntime
//!      {
//!         System: frame_system,
//!         Timestamp: pallet_timestamp,
//!         Balances: pallet_balances,
//!         Assets: pallet_assets,
//!         AjunaSeasons: pallet_ajuna_seasons,
//!         Sage: pallet_sage,
//!      }
//! );
//!
//! pub struct SageEngine;
//! // Use `TestRandomness` for predictable output.
//! // Otherwise, we can instantiate the `pallet_insecure_randomness_collective_flip`
//! // in the runtime.
//! pub type Randomness = TestRandomness<TestRuntime>;
//!
//! impl_core_pallets!(TestRuntime, System);
//! impl_ajuna_seasons!(TestRuntime, GameAssetId, Sage);
//! impl_pallet_sage!(TestRuntime, GameTransition, GameAssetId, GameAsset, AjunaSeasons, Balances, Assets);
//! impl_test_runtime_sage_api!(
//!   SageEngine,
//!   TestRuntime,
//!   Sage,
//!   AjunaSeasons,
//!   GameAssetId,
//!   GameAsset,
//!   TransitionConfig,
//!   Randomness,
//!   H256
//! )
//! ```

use frame_support::{
	ord_parameter_types, parameter_types,
	traits::{
		fungible::{NativeFromLeft, NativeOrWithId, UnionOf},
		EitherOfDiverse,
	},
	PalletId,
};
use frame_system::{pallet_prelude::BlockNumberFor, EnsureRoot, EnsureSignedBy};
use sp_core::crypto::AccountId32;
use sp_runtime::{traits::IdentifyAccount, MultiSignature, Perbill};

// convenience reexport such that the tests do not need to put sp-keyring in the Cargo.toml.
pub use sp_keyring::AccountKeyring;

// reexports for macro resolution
pub use frame_system::{self, EnsureSigned};
pub use pallet_balances;
pub use pallet_timestamp;
pub use sp_runtime::{self, generic, traits::IdentityLookup};

pub use ajuna_primitives::{
	payment_handler::{
		TakeNoFeeHandler, TransferFungible, WithdrawCredit, WithdrawFungibles, WithdrawKind,
	},
	trade_manager::AllowAllTradesAndTransfers,
};
pub use sp_core::H256;
pub use sp_runtime::traits::{BlakeTwo256, Verify};

pub const NONE: u64 = 0;
pub const GENESIS_TIME: u64 = 1_585_058_843_000;
pub const ONE_DAY: u64 = 86_400_000;
pub const BLOCKTIME: u64 = 6_000; // 6s per block

// Unit = the base number of indivisible units for balances
pub const UNIT: Balance = 1_000_000_000_000;
pub const MILLI_UNIT: Balance = 1_000_000_000;
pub const MICRO_UNIT: Balance = 1_000_000;

/// The signature type used by accounts/transactions.
pub type Signature = MultiSignature;
/// An identifier for an account on this system.
pub type AccountId = <<MultiSignature as Verify>::Signer as IdentifyAccount>::AccountId;
pub type Address = sp_runtime::MultiAddress<AccountId, ()>;

pub type BlockNumber = u64;
pub type Balance = u128;

pub type Header = generic::Header<BlockNumber, BlakeTwo256>;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub const MaximumBlockWeight: u32 = 1024;
	pub const MaximumBlockLength: u32 = 2 * 1024;
	pub const AvailableBlockRatio: Perbill = Perbill::one();
}

#[macro_export]
macro_rules! impl_frame_system {
	($runtime:ident) => {
		impl frame_system::Config for $runtime {
			type BaseCallFilter = frame_support::traits::Everything;
			type BlockWeights = ();
			type BlockLength = ();
			type Block = generic::Block<Header, UncheckedExtrinsic>;
			type DbWeight = ();
			type RuntimeOrigin = RuntimeOrigin;
			type Nonce = u64;
			type RuntimeCall = RuntimeCall;
			type RuntimeTask = RuntimeTask;
			type Hash = H256;
			type Hashing = BlakeTwo256;
			type AccountId = AccountId;
			type Lookup = IdentityLookup<Self::AccountId>;
			type RuntimeEvent = RuntimeEvent;
			type BlockHashCount = BlockHashCount;
			type Version = ();
			type PalletInfo = PalletInfo;
			type AccountData = pallet_balances::AccountData<Balance>;
			type OnNewAccount = ();
			type OnKilledAccount = ();
			type SystemWeightInfo = ();
			type SS58Prefix = ();
			type OnSetCode = ();
			type MaxConsumers = frame_support::traits::ConstU32<16>;
			type SingleBlockMigrations = ();
			type MultiBlockMigrator = ();
			type PreInherents = ();
			type PostInherents = ();
			type PostTransactions = ();
		}
	};
}

pub type Moment = u64;
parameter_types! {
	pub const MinimumPeriod: Moment = BLOCKTIME / 2;
}

#[macro_export]
macro_rules! impl_timestamp {
	($runtime:ident, $scheduler:ident) => {
		impl pallet_timestamp::Config for $runtime {
			type Moment = Moment;
			type OnTimestampSet = $scheduler;
			type MinimumPeriod = MinimumPeriod;
			type WeightInfo = ();
		}
	};
	($runtime:ident) => {
		impl pallet_timestamp::Config for $runtime {
			type Moment = Moment;
			type OnTimestampSet = ();
			type MinimumPeriod = MinimumPeriod;
			type WeightInfo = ();
		}
	};
}

parameter_types! {
	pub const TransferFee: Balance = 0;
	pub const CreationFee: Balance = 0;
	pub const TransactionBaseFee: Balance = 0;
	pub const TransactionByteFee: Balance = 0;

	pub const ExistentialDeposit: Balance = 1;
}

#[macro_export]
macro_rules! impl_balances {
	($runtime:ident, $system:ident) => {
		impl pallet_balances::Config for $runtime {
			type Balance = Balance;
			type RuntimeEvent = RuntimeEvent;
			type DustRemoval = ();
			type ExistentialDeposit = ExistentialDeposit;
			type AccountStore = System;
			type WeightInfo = ();
			type MaxLocks = ();
			type MaxReserves = frame_support::traits::ConstU32<1000>;
			type ReserveIdentifier = [u8; 8];
			type RuntimeHoldReason = ();
			type RuntimeFreezeReason = RuntimeFreezeReason;
			type FreezeIdentifier = ();
			type MaxFreezes = frame_support::traits::ConstU32<0>;
		}
	};
}

parameter_types! {
	pub const AssetDeposit: Balance = Balance::MAX;
	pub const AssetAccountDeposit: Balance = 1_000 * UNIT;
	pub const ApprovalDeposit: Balance = 1_000 * UNIT;
	pub const MetadataDepositBase: Balance = 0;
	pub const MetadataDepositPerByte: Balance = 0;
}

pub type PalletAssetsAssetId = u32;

#[macro_export]
macro_rules! impl_assets {
	($runtime:ident) => {
		impl pallet_assets::Config for $runtime {
			type RuntimeEvent = RuntimeEvent;
			type Balance = Balance;
			type RemoveItemsLimit = ConstU32<1000>;
			type AssetId = PalletAssetsAssetId;
			type AssetIdParameter = parity_scale_codec::Compact<u32>;
			type Currency = Balances;
			type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<AccountId>>;
			type ForceOrigin = EnsureAlice;
			type AssetDeposit = AssetDeposit;
			type AssetAccountDeposit = AssetAccountDeposit;
			type MetadataDepositBase = MetadataDepositBase;
			type MetadataDepositPerByte = MetadataDepositPerByte;
			type ApprovalDeposit = ApprovalDeposit;
			type StringLimit = ConstU32<20>;
			type Freezer = ();
			type Extra = ();
			type CallbackHandle = ();
			type WeightInfo = ();
			#[cfg(feature = "runtime-benchmarks")]
			type BenchmarkHelper = ();
		}
	};
}

pub type FungiblesAssetId = WithdrawKind<NativeOrWithId<PalletAssetsAssetId>>;

// Assume that the test runtime has both, the pallet-balances and the pallet-assets.
pub type NativeAndAssetsG<Balances, Assets> =
	UnionOf<Balances, Assets, NativeFromLeft, NativeOrWithId<PalletAssetsAssetId>, AccountId>;
pub type TransferFungibles<Balances, Assets> =
	WithdrawFungibles<NativeAndAssetsG<Balances, Assets>, AccountId>;

pub type TestFeeFeeHandler<TransitionId> = TakeNoFeeHandler<
	AccountId,
	WithdrawKind<NativeOrWithId<PalletAssetsAssetId>>,
	Balance,
	pallet_sage::AffiliateMethods<TransitionId>,
	SeasonId,
>;

parameter_types! {
	pub const SagePalletId: PalletId = PalletId(*b"sage/tst");
}

#[macro_export]
macro_rules! impl_pallet_sage {
	(
		$runtime:ident,
		$game_transition:ident,
		$game_asset_id:ident,
		$game_asset:ident,
		$season_handler:ident,
		$pallet_balances:ident,
		$pallet_assets:ident
	) => {
		impl pallet_sage::Config for $runtime {
			type PalletId = SagePalletId;
			type SageGameTransition = $game_transition;
			type NextAssetIdProvider = IncrementingAssetIdProvider<$game_asset_id>;
			type SeasonHandler = $season_handler;
			type FeeHandler = TestFeeFeeHandler<$game_asset_id>;
			type TransferFunds = TransferFungibleAssets<TransferWithdraw, FungiblesAssetId>;
			type FungiblesAssetId = FungiblesAssetId;
			type FilterHandler = AllowAllTradesAndTransfers<(), $game_asset>;
			type Fungible = $pallet_balances;
			type RuntimeEvent = RuntimeEvent;
			type WeightInfo = ();
			#[cfg(feature = "runtime-benchmarks")]
			type BenchmarkHelper = ();
		}
	};
}

pub type SeasonId = u32;

#[macro_export]
macro_rules! impl_ajuna_seasons {
	($runtime:ident, $game_asset_id:ident, $pallet_sage:ident) => {
		impl pallet_ajuna_seasons::Config for $runtime {
			type RuntimeEvent = RuntimeEvent;
			type SeasonId = SeasonId;
			type AssetId = $game_asset_id;
			type AccountHandler = $pallet_sage;
			type Currency = Balances;
			type WeightInfo = ();
			#[cfg(feature = "runtime-benchmarks")]
			type BenchmarkHelper = ();
		}
	};
}

/// Test Runtime specific sage implementation.
#[macro_export]
macro_rules! impl_test_runtime_sage_api {
	(
		$impl_target:ident,
		$runtime:ident,
		$sage_instance:ident,
		$season_manager:ident,
		$game_asset_id:ident,
		$game_asset:ident,
		$transition_config:ident,
		$randomness_source:ident,
		$hash:ident
	) => {
		impl_sage_api!(
			$impl_target,
			$runtime,
			$sage_instance,
			$season_manager,
			$randomness_source,
			AccountId,
			$game_asset_id,
			$game_asset,
			FungiblesAssetId,
			Balance,
			BlockNumber,
			SeasonId,
			$transition_config,
			$hash,
		);
	};
}

#[macro_export]
macro_rules! impl_core_pallets {
	($t:ident, $system:ident) => {
		impl_frame_system!($t);
		impl_balances!($t, $system);
		impl_timestamp!($t);
		impl_assets!($t);
	};
}

parameter_types! {
	pub const MomentsPerDay: u64 = 86_400_000; // [ms/d]
}

ord_parameter_types! {
	pub const Alice: AccountId32 = AccountId32::new([212, 53, 147, 199, 21, 253, 211, 28, 97, 20, 26, 189, 4, 169, 159, 214, 130, 44, 133, 88, 133, 76, 205, 227, 154, 86, 132, 231, 165, 109, 162, 125]);
}

/// Test origin for the pallet's `EnsureOrigin` associated type.
pub type EnsureAlice = EitherOfDiverse<EnsureSignedBy<Alice, AccountId32>, EnsureRoot<AccountId32>>;

/// Provides an implementation of [`frame_support::traits::Randomness`] that should only be used in
/// tests!
/// 
/// This can be injected into the `impl_test_runtime_sage_api` macro in order to get predictable results.
pub struct TestRandomness<T>(sp_std::marker::PhantomData<T>);

impl<Output: parity_scale_codec::Decode + Default, T>
	frame_support::traits::Randomness<Output, BlockNumberFor<T>> for TestRandomness<T>
where
	T: frame_system::Config,
{
	fn random(subject: &[u8]) -> (Output, BlockNumberFor<T>) {
		use sp_runtime::traits::TrailingZeroInput;

		(
			Output::decode(&mut TrailingZeroInput::new(subject)).unwrap_or_default(),
			frame_system::Pallet::<T>::block_number(),
		)
	}
}
