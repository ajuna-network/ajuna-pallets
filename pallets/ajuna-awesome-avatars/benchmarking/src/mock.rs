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

use ajuna_primitives::{
	account_manager::WhitelistKey,
	fee_handler::{FeeProvider, GameFeeHandler},
};
use frame_support::{
	parameter_types,
	traits::{ConstU16, ConstU64},
	PalletId,
};
use frame_system::pallet_prelude::BlockNumberFor;
use pallet_ajuna_affiliates::{traits::AffiliateUnlockRules, BenchmarkHelper};
use pallet_ajuna_awesome_avatars::{
	benchmark_helper,
	types::{AffiliateMethods, Avatar, SeasonId},
	AvatarIdOf, AvatarOf, AvatarRankerFor, Avatars, CurrentSeasonStatus, Owners,
};
use pallet_ajuna_tournament::{GoldenDuckConfig, TournamentConfig};
use sp_runtime::{
	bounded_vec,
	testing::H256,
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	BuildStorage, DispatchError, MultiSignature,
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

pub struct MockTransitionFeeProvider;

impl FeeProvider for MockTransitionFeeProvider {
	type AccountId = MockAccountId;
	type FeeIdentifier = SeasonId;
	type FeeCurrency = MockBalance;
	type FeeOutput = MockBalance;

	fn get_fee_from(
		base_fee: Self::FeeCurrency,
		_account: &Self::AccountId,
		_identifier: &Self::FeeIdentifier,
	) -> Self::FeeOutput {
		base_fee
	}
}

impl pallet_ajuna_awesome_avatars::Config for Runtime {
	type PalletId = AwesomeAvatarsPalletId;
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type Randomness = Randomness;
	type FeeChainMaxLength = AffiliateMaxLevel;
	type AffiliateHandler = Affiliates;
	type TournamentHandler = Tournament;
	type FeeHandler = GameFeeHandler<
		MockAccountId,
		Balances,
		Affiliates,
		Tournament,
		MockTransitionFeeProvider,
		AAvatars,
	>;
	type WeightInfo = ();
}

parameter_types! {
	pub const AffiliateMaxLevel: u32 = 2;
	pub const AffiliateWhitelistKey: WhitelistKey = [1, 2, 1, 2, 3, 3, 4, 5];
}

#[cfg(feature = "runtime-benchmarks")]
pub struct AffiliateBenchmarkHelper;

#[cfg(feature = "runtime-benchmarks")]
impl BenchmarkHelper<AffiliateMethods, MockUnlockParameter> for AffiliateBenchmarkHelper {
	fn create_rule_id(_id: u32) -> AffiliateMethods {
		AffiliateMethods::Mint
	}

	fn create_params(id: u32) -> MockUnlockParameter {
		id as u8
	}
}

pub type MockUnlockParameter = u8;
pub struct MockAffiliateRules;

impl AffiliateUnlockRules for MockAffiliateRules {
	type AccountId = MockAccountId;
	type UnlockParameters = MockUnlockParameter;

	fn execute_unlock_rule_for(
		_account: &Self::AccountId,
		_params: Self::UnlockParameters,
	) -> Result<(), DispatchError> {
		Ok(())
	}
}

type AffiliatesInstance1 = pallet_ajuna_affiliates::Instance1;
impl pallet_ajuna_affiliates::Config<AffiliatesInstance1> for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type WhitelistKey = AffiliateWhitelistKey;
	type AccountManager = AAvatars;
	type RuleIdentifier = AffiliateMethods;
	type AffiliateMaxLevel = AffiliateMaxLevel;
	type UnlockParameters = MockUnlockParameter;
	type AffiliatesUnlockRules = MockAffiliateRules;
	type WeightInfo = ();
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = AffiliateBenchmarkHelper;
}

parameter_types! {
	pub const TournamentPalletId1: PalletId = PalletId(*b"aj/trmt1");
	pub const MinimumTournamentPhaseDuration: MockBlockNumber = 100;
}

#[cfg(feature = "runtime-benchmarks")]
pub struct TournamentBenchmarkHelper;

#[cfg(feature = "runtime-benchmarks")]
impl
	pallet_ajuna_tournament::BenchmarkHelper<
		SeasonId,
		MockBlockNumber,
		MockBalance,
		AvatarRankerFor<Runtime>,
		MockAccountId,
		AvatarIdOf<Runtime>,
		AvatarOf<Runtime>,
	> for TournamentBenchmarkHelper
{
	fn create_category_id(id: u32) -> SeasonId {
		id as SeasonId
	}

	fn create_default_tournament_config(
	) -> TournamentConfig<MockBlockNumber, MockBalance, AvatarRankerFor<Runtime>> {
		TournamentConfig {
			start: 20_u64,
			active_end: 50_u64,
			claim_end: 70_u64,
			initial_reward: Some(10),
			max_reward: None,
			take_fee_percentage: None,
			reward_distribution: bounded_vec![40, 30, 10],
			golden_duck_config: GoldenDuckConfig::Enabled(10),
			max_players: 4,
			ranker: AvatarRankerFor::<Runtime>::default(),
		}
	}

	fn create_entities(
		owner: MockAccountId,
		count: u32,
	) -> Vec<(AvatarIdOf<Runtime>, AvatarOf<Runtime>)> {
		benchmark_helper::create_avatars::<Runtime>(owner.clone(), count).unwrap();

		let season_id = CurrentSeasonStatus::<Runtime>::get().season_id;
		let avatar_ids = Owners::<Runtime>::get(owner, season_id);
		let mut avatars = Vec::with_capacity(avatar_ids.len());

		for avatar_id in avatar_ids.iter() {
			let (_, avatar) = Avatars::<Runtime>::get(avatar_id).unwrap();
			avatars.push(avatar);
		}

		avatar_ids.into_iter().zip(avatars).collect()
	}
}

type TournamentInstance1 = pallet_ajuna_tournament::Instance1;
impl pallet_ajuna_tournament::Config<TournamentInstance1> for Runtime {
	type PalletId = TournamentPalletId1;
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type TournamentCategoryId = SeasonId;
	type EntityId = crate::AvatarIdOf<Runtime>;
	type RankedEntity = Avatar<BlockNumberFor<Runtime>>;
	type EntityRanker = AvatarRankerFor<Runtime>;
	type AccountManager = AAvatars;
	type AssetManager = AAvatars;
	type MinimumTournamentPhaseDuration = MinimumTournamentPhaseDuration;
	type WeightInfo = ();
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = TournamentBenchmarkHelper;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();
	sp_io::TestExternalities::new(t)
}
