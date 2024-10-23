// Ajuna Node
// Copyright (C) 2022 BlogaTech AG

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

#![cfg(test)]

use ajuna_primitives::account_manager::WhitelistKey;
use frame_support::{
	parameter_types,
	traits::{ConstU16, ConstU64},
	PalletId,
};
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_ajuna_awesome_avatars::{
	types::{AffiliateMethods, Avatar, SeasonId},
	FeePropagationOf,
};
use sp_runtime::{
	testing::H256,
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	BuildStorage, MultiSignature,
};

pub type MockSignature = MultiSignature;
pub type MockAccountPublic = <MockSignature as Verify>::Signer;
pub type MockAccountId = <MockAccountPublic as IdentifyAccount>::AccountId;
pub type MockBlock = frame_system::mocking::MockBlock<Runtime>;
pub type MockBlockNumber = u64;
pub type MockBalance = u64;

impl crate::Config for Runtime {}

frame_support::construct_runtime!(
	pub struct Runtime {
		System: frame_system = 0,
		Balances: pallet_balances = 1,
		Randomness: pallet_insecure_randomness_collective_flip = 2,
		AAvatars: pallet_ajuna_awesome_avatars = 4,
		Affiliates: pallet_ajuna_affiliates::<Instance1> = 6,
		Tournament: pallet_ajuna_tournament::<Instance1> = 7,
	}
);

impl frame_system::Config for Runtime {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = MockAccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type RuntimeEvent = RuntimeEvent;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<MockBalance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
	type Nonce = u32;
	type Block = MockBlock;
	type RuntimeTask = RuntimeTask;
	type SingleBlockMigrations = ();
	type MultiBlockMigrator = ();
	type PreInherents = ();
	type PostInherents = ();
	type PostTransactions = ();
}

parameter_types! {
	pub static MockExistentialDeposit: MockBalance = 321;
}

impl pallet_balances::Config for Runtime {
	type Balance = MockBalance;
	type DustRemoval = ();
	type RuntimeEvent = RuntimeEvent;
	type ExistentialDeposit = MockExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type FreezeIdentifier = ();
	type MaxFreezes = ();
	type RuntimeHoldReason = ();
	type RuntimeFreezeReason = ();
}

impl pallet_insecure_randomness_collective_flip::Config for Runtime {}

parameter_types! {
	pub const CollectionDeposit: MockBalance = 1;
	pub const ItemDeposit: MockBalance = 1;
	pub const StringLimit: u32 = 128;
	pub const MetadataDepositBase: MockBalance = 1;
	pub const AttributeDepositBase: MockBalance = 1;
	pub const DepositPerByte: MockBalance = 1;
	pub const ApprovalsLimit: u32 = 1;
	pub const ItemAttributesApprovalsLimit: u32 = 10;
	pub const MaxTips: u32 = 1;
	pub const MaxDeadlineDuration: u32 = 1;
	pub const MaxAttributesPerCall: u32 = 10;
	pub ConfigFeatures: pallet_nfts::PalletFeatures = pallet_nfts::PalletFeatures::all_enabled();
}

parameter_types! {
	pub const AwesomeAvatarsPalletId: PalletId = PalletId(*b"aj/aaatr");
}

impl pallet_ajuna_awesome_avatars::Config for Runtime {
	type PalletId = AwesomeAvatarsPalletId;
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type Randomness = Randomness;
	type FeeChainMaxLength = AffiliateMaxLevel;
	type AffiliateHandler = Affiliates;
	type TournamentHandler = Tournament;
	type WeightInfo = ();
}

parameter_types! {
	pub const AffiliateMaxLevel: u32 = 2;
	pub const AffiliateWhitelistKey: WhitelistKey = [1, 2, 1, 2, 3, 3, 4, 5];
}

type AffiliatesInstance1 = pallet_ajuna_affiliates::Instance1;
impl pallet_ajuna_affiliates::Config<AffiliatesInstance1> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type WhitelistKey = AffiliateWhitelistKey;
	type AccountManager = AAvatars;
	type RuleIdentifier = AffiliateMethods;
	type RuntimeRule = FeePropagationOf<Runtime>;
	type AffiliateMaxLevel = AffiliateMaxLevel;
}

parameter_types! {
	pub const TournamentPalletId1: PalletId = PalletId(*b"aj/trmt1");
	pub const MinimumTournamentPhaseDuration: MockBlockNumber = 100;
}

type TournamentInstance1 = pallet_ajuna_tournament::Instance1;
impl pallet_ajuna_tournament::Config<TournamentInstance1> for Runtime {
	type PalletId = TournamentPalletId1;
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type SeasonId = SeasonId;
	type EntityId = crate::AvatarIdOf<Runtime>;
	type RankedEntity = Avatar<BlockNumberFor<Runtime>>;
	type MinimumTournamentPhaseDuration = MinimumTournamentPhaseDuration;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();
	sp_io::TestExternalities::new(t)
}
