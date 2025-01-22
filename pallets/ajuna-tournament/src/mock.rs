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

use crate::{self as pallet_ajuna_tournament, *};
use ajuna_primitives::{
	account_manager::{AccountManager, WhitelistKey},
	asset_manager::{AssetManager, Lock},
	tournament_manager::{EntityRanker, TournamentBenchmarkHelper},
};
use frame_support::{
	parameter_types,
	traits::{ConstU16, ConstU64, LockIdentifier},
	PalletId,
};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::{
	testing::H256,
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	BuildStorage, MultiSignature,
};
use sp_std::{cell::RefCell, cmp::Ordering, collections::btree_map::BTreeMap};

pub type MockSignature = MultiSignature;
pub type MockAccountPublic = <MockSignature as Verify>::Signer;
pub type MockAccountId = <MockAccountPublic as IdentifyAccount>::AccountId;
pub type MockBlock = frame_system::mocking::MockBlock<Test>;
pub type MockBalance = u64;
pub type MockBlockNumber = BlockNumberFor<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub struct Test {
		System: frame_system = 0,
		Balances: pallet_balances = 1,
		TournamentAlpha: pallet_ajuna_tournament::<Instance1> = 2,
		TournamentBeta: pallet_ajuna_tournament::<Instance2> = 3,
		#[cfg(feature = "runtime-benchmarks")]
		TournamentBench: pallet_ajuna_tournament = 4,
	}
);

impl frame_system::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type RuntimeOrigin = RuntimeOrigin;
	type RuntimeCall = RuntimeCall;
	type RuntimeTask = RuntimeTask;
	type Nonce = u32;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = MockAccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Block = MockBlock;
	type BlockHashCount = ConstU64<250>;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<MockBalance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
	type SingleBlockMigrations = ();
	type MultiBlockMigrator = ();
	type PreInherents = ();
	type PostInherents = ();
	type PostTransactions = ();
}

parameter_types! {
	pub const MockExistentialDeposit: MockBalance = 3;
}

impl pallet_balances::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type RuntimeHoldReason = ();
	type RuntimeFreezeReason = ();
	type WeightInfo = ();
	type Balance = MockBalance;
	type DustRemoval = ();
	type ExistentialDeposit = MockExistentialDeposit;
	type AccountStore = System;
	type ReserveIdentifier = [u8; 8];
	type FreezeIdentifier = ();
	type MaxLocks = ();
	type MaxReserves = ();
	type MaxFreezes = ();
}

pub type MockCategoryId = u32;
pub type MockEntityId = H256;
pub type MockEntity = u32;

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq, Eq)]
pub struct MockRanker;

impl EntityRanker for MockRanker {
	type EntityId = MockEntityId;
	type Entity = MockEntity;

	fn can_rank(&self, _entity: (&Self::EntityId, &Self::Entity)) -> bool {
		true
	}

	fn rank_against(
		&self,
		entity: (&Self::EntityId, &Self::Entity),
		other: (&Self::EntityId, &Self::Entity),
	) -> Ordering {
		if entity.0.cmp(other.0) == Ordering::Equal {
			Ordering::Equal
		} else {
			entity.1.cmp(other.1)
		}
	}
}

thread_local! {
	pub static ORGANIZER: RefCell<Option<MockAccountId>> = const { RefCell::new(None) };
	pub static ASSETS: RefCell<BTreeMap<MockEntityId, MockEntity>> = const { RefCell::new(BTreeMap::new()) };
	pub static OWNERS: RefCell<BTreeMap<MockAccountId, MockEntityId>> = const { RefCell::new(BTreeMap::new()) };
}

pub struct MockAccountManager;

pub const ACCOUNT_IS_NOT_ORGANIZER: &str = "ACCOUNT_IS_NOT_ORGANIZER";
pub const NO_ORGANIZER_SET: &str = "NO_ORGANIZER_SET";

impl MockAccountManager {
	pub fn set_organizer(account: MockAccountId) {
		ORGANIZER.with_borrow_mut(|maybe_organizer| *maybe_organizer = Some(account))
	}
}

impl AccountManager for MockAccountManager {
	type AccountId = MockAccountId;

	fn is_organizer(account: &Self::AccountId) -> Result<(), DispatchError> {
		ORGANIZER.with(|maybe_account| {
			if let Some(organizer) = maybe_account.borrow().as_ref() {
				ensure!(organizer == account, DispatchError::Other(ACCOUNT_IS_NOT_ORGANIZER));
				Ok(())
			} else {
				Err(DispatchError::Other(NO_ORGANIZER_SET))
			}
		})
	}

	fn is_whitelisted_for(_identifier: &WhitelistKey, _account: &Self::AccountId) -> bool {
		unimplemented!()
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn set_organizer(account: Self::AccountId) {
		MockAccountManager::set_organizer(account);
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn try_add_to_whitelist(
		_identifier: &WhitelistKey,
		_account: Self::AccountId,
	) -> Result<(), DispatchError> {
		unimplemented!()
	}
}

/// In the future we might want to use the `pallet-awesome-ajuna-avatars`, but currently this
/// pallet is too loaded and requires many dependencies.
///
/// Hence, we implement our own little asset manager here.
pub struct MockAssetManager;

impl MockAssetManager {
	pub fn create_assets(owner: MockAccountId, count: u32) -> Vec<(MockEntityId, MockEntity)> {
		let mut ids = Vec::with_capacity(count as usize);
		let mut items = Vec::with_capacity(count as usize);
		for i in 0..count {
			let id = MockEntityId::repeat_byte(i as u8);
			ids.push(id);
			items.push(i);
			Self::add_asset(owner.clone(), id, i)
		}

		ids.into_iter().zip(items).collect()
	}

	pub fn add_asset(owner: MockAccountId, asset_id: MockEntityId, asset: MockEntity) {
		OWNERS.with(|owners| owners.borrow_mut().insert(owner, asset_id));
		ASSETS.with(|assets| assets.borrow_mut().insert(asset_id, asset));
	}
}

pub const NOT_OWNER_ERR: &str = "NOT_OWNER";

impl AssetManager for MockAssetManager {
	type AccountId = MockAccountId;
	type AssetId = MockEntityId;
	type Asset = MockEntity;

	fn ensure_ownership(
		owner: &Self::AccountId,
		_asset_id: &Self::AssetId,
	) -> Result<Self::Asset, DispatchError> {
		let id = OWNERS
			.with(|owners| owners.borrow().get(owner).cloned())
			.ok_or(DispatchError::Other(NOT_OWNER_ERR))?;
		ASSETS
			.with(|assets| assets.borrow().get(&id).cloned())
			.ok_or(DispatchError::Other(NOT_OWNER_ERR))
	}

	fn lock_asset(
		_lock_id: LockIdentifier,
		_owner: Self::AccountId,
		_asset_id: Self::AssetId,
	) -> Result<Self::Asset, DispatchError> {
		unimplemented!()
	}

	fn unlock_asset(
		_lock_id: LockIdentifier,
		_owner: Self::AccountId,
		_asset_id: Self::AssetId,
	) -> Result<Self::Asset, DispatchError> {
		unimplemented!()
	}

	fn is_locked(_asset: &Self::AssetId) -> Option<Lock<Self::AccountId>> {
		unimplemented!()
	}
}

parameter_types! {
	pub const TournamentPalletId1: PalletId = PalletId(*b"aj/trmt1");
	pub const TournamentPalletId2: PalletId = PalletId(*b"aj/trmt2");
	pub const MinimumTournamentPhaseDuration: MockBlockNumber = 2;
}

#[cfg(feature = "runtime-benchmarks")]
pub struct MockTournamentBenchmarkHelper;

#[cfg(feature = "runtime-benchmarks")]
impl
	TournamentBenchmarkHelper<
		MockCategoryId,
		TournamentConfigFor<Test, Instance1>,
		MockAccountId,
		(MockEntityId, MockEntity),
	> for MockTournamentBenchmarkHelper
{
	fn create_category_id() -> MockCategoryId {
		2
	}

	fn create_config() -> TournamentConfig<MockBlockNumber, MockBalance, MockRanker> {
		TournamentConfig {
			start: 20_u64,
			active_end: 50_u64,
			claim_end: 70_u64,
			initial_reward: Some(10),
			max_reward: None,
			take_fee_percentage: None,
			reward_distribution: vec![40, 30, 10].try_into().unwrap(),
			golden_duck_config: GoldenDuckConfig::Enabled(10),
			max_players: 4,
			ranker: MockRanker,
		}
	}

	fn create_entities(owner: &MockAccountId, count: usize) -> Vec<(MockEntityId, MockEntity)> {
		MockAssetManager::create_assets(owner.clone(), count as u32)
	}
}

type TournamentInstance1 = pallet_ajuna_tournament::Instance1;
impl pallet_ajuna_tournament::Config<TournamentInstance1> for Test {
	type PalletId = TournamentPalletId1;
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type TournamentCategoryId = MockCategoryId;
	type EntityId = MockEntityId;
	type RankedEntity = MockEntity;
	type EntityRanker = MockRanker;
	type AccountManager = MockAccountManager;
	type AssetManager = MockAssetManager;
	type MinimumTournamentPhaseDuration = MinimumTournamentPhaseDuration;
	type WeightInfo = ();
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = MockTournamentBenchmarkHelper;
}

type TournamentInstance2 = pallet_ajuna_tournament::Instance2;
impl pallet_ajuna_tournament::Config<TournamentInstance2> for Test {
	type PalletId = TournamentPalletId2;
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type TournamentCategoryId = MockCategoryId;
	type EntityId = MockEntityId;
	type RankedEntity = MockEntity;
	type EntityRanker = MockRanker;
	type AccountManager = MockAccountManager;
	type AssetManager = MockAssetManager;
	type MinimumTournamentPhaseDuration = MinimumTournamentPhaseDuration;
	type WeightInfo = ();
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = MockTournamentBenchmarkHelper;
}

#[cfg(feature = "runtime-benchmarks")]
impl Config for Test {
	type PalletId = TournamentPalletId1;
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type TournamentCategoryId = MockCategoryId;
	type EntityId = MockEntityId;
	type RankedEntity = MockEntity;
	type EntityRanker = MockRanker;
	type AccountManager = MockAccountManager;
	type AssetManager = MockAssetManager;
	type MinimumTournamentPhaseDuration = MinimumTournamentPhaseDuration;
	type WeightInfo = ();
	type BenchmarkHelper = MockTournamentBenchmarkHelper;
}

#[cfg(test)]
pub struct ExtBuilder {
	balances: Vec<(MockAccountId, MockBalance)>,
	organizer: Option<MockAccountId>,
}

#[cfg(test)]
impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			balances: vec![
				(crate::tests::ALICE, 1_000),
				(crate::tests::BOB, 1_000),
				(crate::tests::CHARLIE, 1_000),
				(crate::tests::EDWARD, 1_000),
				(crate::tests::DAVE, 1_000),
			],
			organizer: None,
		}
	}
}

#[cfg(test)]
impl ExtBuilder {
	pub fn balances(mut self, balances: &[(MockAccountId, MockBalance)]) -> Self {
		self.balances = balances.to_vec();
		self
	}

	pub fn organizer(mut self, organizer: MockAccountId) -> Self {
		self.organizer = Some(organizer);
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let config = RuntimeGenesisConfig {
			system: Default::default(),
			balances: BalancesConfig { balances: self.balances },
		};

		let mut ext: sp_io::TestExternalities = config.build_storage().unwrap().into();
		ext.execute_with(|| System::set_block_number(1));
		ext.execute_with(|| {
			if let Some(account) = self.organizer {
				MockAccountManager::set_organizer(account);
			}
		});
		ext
	}
}

pub fn new_test_ext() -> sp_io::TestExternalities {
	ExtBuilder::default().build()
}

pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		if System::block_number() > 1 {
			System::on_finalize(System::block_number());
			TournamentAlpha::on_finalize(System::block_number());
			TournamentBeta::on_finalize(System::block_number());
			#[cfg(feature = "runtime-benchmarks")]
			TournamentBench::on_finalize(System::block_number());
		}
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		TournamentAlpha::on_initialize(System::block_number());
		TournamentBeta::on_initialize(System::block_number());
		#[cfg(feature = "runtime-benchmarks")]
		TournamentBench::on_initialize(System::block_number());
	}
}
