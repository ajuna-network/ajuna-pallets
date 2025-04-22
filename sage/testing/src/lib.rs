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

//extern crate externalities;
//extern crate test_client;
//extern crate node_primitives;

use frame_support::{ord_parameter_types, parameter_types, traits::EitherOfDiverse, PalletId};
use frame_system::{pallet_prelude::BlockNumberFor, EnsureRoot, EnsureSignedBy};
use sp_core::crypto::AccountId32;
use sp_runtime::{generic, traits::IdentifyAccount, MultiSignature, Perbill};

// convenience reexport such that the tests do not need to put sp-keyring in the Cargo.toml.
pub use sp_keyring::AccountKeyring;

// reexports for macro resolution
pub use frame_system;
pub use pallet_balances;
pub use pallet_timestamp;
pub use sp_runtime;

pub use sp_core::H256;
pub use sp_runtime::traits::{BlakeTwo256, Verify};

pub const NONE: u64 = 0;
pub const GENESIS_TIME: u64 = 1_585_058_843_000;
pub const ONE_DAY: u64 = 86_400_000;
pub const BLOCKTIME: u64 = 6_000; // 6s per block

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
	($t:ident) => {
		use sp_runtime::{generic, traits::IdentityLookup};
		impl frame_system::Config for $t {
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
			type ExtensionsWeightInfo = ();
		}
	};
}

pub type Moment = u64;
parameter_types! {
	pub const MinimumPeriod: Moment = BLOCKTIME / 2;
}

#[macro_export]
macro_rules! impl_timestamp {
	($t:ident, $scheduler:ident) => {
		impl pallet_timestamp::Config for $t {
			type Moment = Moment;
			type OnTimestampSet = $scheduler;
			type MinimumPeriod = MinimumPeriod;
			type WeightInfo = ();
		}
	};
	($t:ident) => {
		impl pallet_timestamp::Config for $t {
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
}

#[macro_export]
macro_rules! impl_balances {
	($t:ident, $system:ident) => {
		impl pallet_balances::Config for $t {
			type Balance = Balance;
			type RuntimeEvent = RuntimeEvent;
			type DustRemoval = ();
			type ExistentialDeposit = frame_support::traits::ConstU128<1>;
			type AccountStore = System;
			type WeightInfo = ();
			type MaxLocks = ();
			type MaxReserves = frame_support::traits::ConstU32<1000>;
			type ReserveIdentifier = [u8; 8];
			type RuntimeHoldReason = ();
			type RuntimeFreezeReason = RuntimeFreezeReason;
			type FreezeIdentifier = ();
			type MaxFreezes = frame_support::traits::ConstU32<0>;
			type DoneSlashHandler = ();
		}
	};
}

#[macro_export]
macro_rules! test_runtime {
	($t:ident, $system:ident, $scheduler:ident) => {
		impl_frame_system!($t);
		impl_balances!($t, $system);
		impl_timestamp!($t, $scheduler);
		impl_outer_origin_for_runtime!($t);
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
