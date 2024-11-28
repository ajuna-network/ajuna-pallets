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

use crate::{self as pallet_sage, *};
use ajuna_primitives::{
	asset_manager::AssetInspector,
	fee_handler::{FeeProvider, GameFeeHandler},
	season_manager::{SeasonConfig, SeasonFeeConfig, SeasonManager},
	trade_manager::TradeManager,
	treasury_manager::TreasuryManager,
};

use frame_support::{
	parameter_types,
	traits::{ConstU16, ConstU64, ExistenceRequirement},
	PalletId,
};
use sp_runtime::{
	testing::{TestSignature, H256},
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	BuildStorage, DispatchError,
};
use sp_std::{cell::RefCell, collections::btree_map::BTreeMap};

pub type MockSignature = TestSignature;
pub type MockAccountPublic = <MockSignature as Verify>::Signer;
pub type MockAccountId = <MockAccountPublic as IdentifyAccount>::AccountId;
pub type MockBlock = frame_system::mocking::MockBlock<Test>;
pub type MockBalance = u64;
pub type MockCollectionId = u32;

pub const ALICE: MockAccountId = 1;
pub const BOB: MockAccountId = 2;
pub const CHARLIE: MockAccountId = 3;
pub const DAVE: MockAccountId = 4;

pub const TREASURER: MockAccountId = 431;

pub const SEASON_ID_0: MockSeasonId = 0;
pub const SEASON_ID_1: MockSeasonId = 1;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub struct Test {
		System: frame_system = 0,
		Balances: pallet_balances = 1,
		Sage: pallet_sage::<Instance1> = 2,
		#[cfg(feature = "runtime-benchmarks")]
		SageBench: pallet_sage = 3,
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

use example_transition::{
	generic::ExampleTransitionGeneric,
	types::{Asset, AssetId, ExampleTransitionId, Level},
};

parameter_types! {
	pub const ExamplePalletId: PalletId = PalletId(*b"sage/exi");
}

thread_local! {
	pub static ASSET_SEEDS: RefCell<u64> = RefCell::new(0);
	pub static ASSET_SEASONS: RefCell<BTreeMap<AssetId, MockSeasonId>> = RefCell::new(BTreeMap::new());
	pub static CURRENT_SEASON: RefCell<MockSeasonId> = RefCell::new(SEASON_ID_0)
}

pub struct MockSeasonManager;

pub type MockSeasonId = u8;

impl SeasonManager for MockSeasonManager {
	type SeasonId = MockSeasonId;
	type SeasonData = ();
	type AssetId = AssetId;
	type Balance = MockBalance;

	fn get_season_id_for(asset_id: &Self::AssetId) -> Result<Self::SeasonId, DispatchError> {
		ASSET_SEASONS.with(|store| {
			store
				.borrow()
				.get(asset_id)
				.cloned()
				.ok_or(DispatchError::Other("Unknown asset"))
		})
	}

	fn get_current_season_id() -> Result<Self::SeasonId, DispatchError> {
		Ok(CURRENT_SEASON.with(|season_id| *season_id.borrow()))
	}

	fn is_valid_season(season_id: &Self::SeasonId) -> Result<(), DispatchError> {
		match season_id {
			&SEASON_ID_0 | &SEASON_ID_1 => Ok(()),
			_ => Err(DispatchError::Other("Invalid season")),
		}
	}

	fn get_season_config_for(
		_season_id: &Self::SeasonId,
	) -> Result<SeasonConfig<Self::Balance, ()>, DispatchError> {
		Ok(SeasonConfig::<Self::Balance, ()> {
			fee: SeasonFeeConfig::<Self::Balance> {
				transfer_asset: MockExistentialDeposit::get(),
				buy_asset_min: MockExistentialDeposit::get(),
				buy_percent: 1,
				upgrade_asset_inventory: MockExistentialDeposit::get(),
				unlock_trade_asset: MockExistentialDeposit::get(),
				unlock_transfer_asset: MockExistentialDeposit::get(),
				state_transition_base_fee: MockExistentialDeposit::get(),
			},
			data: (),
		})
	}

	fn register_asset_in(
		asset_id: &Self::AssetId,
		season_id: &Self::SeasonId,
	) -> Result<(), DispatchError> {
		ASSET_SEASONS.with(|store| {
			store.borrow_mut().insert(*asset_id, *season_id);
		});

		Ok(())
	}
}

pub struct MockAffiliatesFeeProvider;

impl FeeProvider for MockAffiliatesFeeProvider {
	type AccountId = MockAccountId;
	type FeeIdentifier = AffiliateMethodsOf<Test, Instance1>;
	type FeeCurrency = MockBalance;
	type FeeOutput = Vec<(MockBalance, MockAccountId)>;

	fn get_fee_from(
		_base_fee: Self::FeeCurrency,
		_account: &Self::AccountId,
		_identifier: &Self::FeeIdentifier,
	) -> Self::FeeOutput {
		Vec::with_capacity(0)
	}
}

pub struct MockTournamentFeeProvider;

impl FeeProvider for MockTournamentFeeProvider {
	type AccountId = MockAccountId;
	type FeeIdentifier = MockSeasonId;
	type FeeCurrency = MockBalance;
	type FeeOutput = (MockBalance, MockAccountId);

	fn get_fee_from(
		_base_fee: Self::FeeCurrency,
		account: &Self::AccountId,
		_identifier: &Self::FeeIdentifier,
	) -> Self::FeeOutput {
		(0, *account)
	}
}

pub struct MockTransitionFeeProvider;

impl FeeProvider for MockTransitionFeeProvider {
	type AccountId = MockAccountId;
	type FeeIdentifier = ExampleTransitionId;
	type FeeCurrency = MockBalance;
	type FeeOutput = MockBalance;

	fn get_fee_from(
		base_fee: Self::FeeCurrency,
		_account: &Self::AccountId,
		identifier: &Self::FeeIdentifier,
	) -> Self::FeeOutput {
		match identifier {
			ExampleTransitionId::UpgradeAsset => base_fee.saturating_mul(2),
			ExampleTransitionId::ConsumeAsset => base_fee.saturating_mul(3),
			ExampleTransitionId::BenchTransition => base_fee.saturating_mul(10),
		}
	}
}

pub struct MockTreasuryManager;

impl TreasuryManager for MockTreasuryManager {
	type AccountId = MockAccountId;
	type Currency = MockBalance;
	type TreasuryPotKey = MockSeasonId;

	fn is_treasurer_for(
		_key: Self::TreasuryPotKey,
		account: &Self::AccountId,
	) -> Result<(), DispatchError> {
		ensure!(account == &TREASURER, DispatchError::BadOrigin);
		Ok(())
	}

	fn get_treasurer_for(_key: Self::TreasuryPotKey) -> Result<Self::AccountId, DispatchError> {
		Ok(TREASURER)
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn set_treasurer_for(_key: Self::TreasuryPotKey, _owner: Self::AccountId) {
		todo!()
	}

	fn deposit_into(
		depository: &Self::AccountId,
		_key: &Self::TreasuryPotKey,
		fee: Self::Currency,
	) -> Result<(), DispatchError> {
		Balances::transfer(depository, &TREASURER, fee, ExistenceRequirement::KeepAlive)
	}
}

pub struct MockFilterHandler;

pub type MockFilter = u32;

impl TradeManager for MockFilterHandler {
	type TradeFilter = MockFilter;
	type Asset = Asset;

	fn is_tradeable_using(asset: &Self::Asset, filter: &Self::TradeFilter) -> bool {
		asset.asset_type == *filter
	}
}

impl TransferManager for MockFilterHandler {
	type TransferFilter = MockFilter;
	type Asset = Asset;

	fn is_transferable_using(asset: &Self::Asset, filter: &Self::TransferFilter) -> bool {
		asset.asset_type == *filter
	}
}

pub struct MockAssetMediator;

impl AssetManager for MockAssetMediator {
	type AccountId = MockAccountId;
	type AssetId = AssetId;
	type Asset = Asset;

	fn ensure_ownership(
		owner: &Self::AccountId,
		asset_id: &Self::AssetId,
	) -> Result<Self::Asset, DispatchError> {
		<Sage as AssetManager>::ensure_ownership(owner, asset_id)
	}

	fn lock_asset(
		lock_id: LockIdentifier,
		owner: Self::AccountId,
		asset_id: Self::AssetId,
	) -> Result<Self::Asset, DispatchError> {
		<Sage as AssetManager>::lock_asset(lock_id, owner, asset_id)
	}

	fn unlock_asset(
		lock_id: LockIdentifier,
		owner: Self::AccountId,
		asset_id: Self::AssetId,
	) -> Result<Self::Asset, DispatchError> {
		<Sage as AssetManager>::unlock_asset(lock_id, owner, asset_id)
	}

	fn is_locked(asset: &Self::AssetId) -> Option<Lock<Self::AccountId>> {
		<Sage as AssetManager>::is_locked(asset)
	}

	fn nft_transfer_open() -> bool {
		<Sage as AssetManager>::nft_transfer_open()
	}

	fn handle_asset_prepare_fee(
		asset: &Self::Asset,
		from: &Self::AccountId,
		fees_recipient: &Self::AccountId,
	) -> Result<(), DispatchError> {
		<Sage as AssetManager>::handle_asset_prepare_fee(asset, from, fees_recipient)
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn create_assets(owner: Self::AccountId, count: u32) -> Vec<(Self::AssetId, Self::Asset)> {
		<Sage as AssetManager>::create_assets(owner, count)
	}
}

impl AssetInspector for MockAssetMediator {
	type AssetId = AssetId;
	type Asset = Asset;

	fn get_asset(asset_id: &Self::AssetId) -> Result<Self::Asset, DispatchError> {
		<Sage as AssetInspector>::get_asset(asset_id)
	}
}

#[cfg(feature = "runtime-benchmarks")]
pub struct SageBenchmarkHelper;

#[cfg(feature = "runtime-benchmarks")]
impl
	BenchmarkHelper<
		AssetId,
		Asset,
		MockFilter,
		MockFilter,
		MockSeasonId,
		ExampleTransitionId,
		(),
		(),
	> for SageBenchmarkHelper
{
	fn create_asset(seed: u32) -> (AssetId, Asset) {
		let asset_id = AssetId::from_low_u64_le(seed as u64);
		let asset = Asset::create(asset_id, 0, 0, 0, [seed as u8; 32], 1, Level::One);

		(asset_id, asset)
	}

	fn create_asset_trade_filter(id: u32) -> MockFilter {
		MockFilter::from(id)
	}

	fn create_asset_transfer_filter(id: u32) -> MockFilter {
		MockFilter::from(id)
	}

	fn create_transition_id(id: u32) -> ExampleTransitionId {
		if id == 99 {
			ExampleTransitionId::BenchTransition
		} else {
			ExampleTransitionId::UpgradeAsset
		}
	}

	fn create_season_id(id: u32) -> MockSeasonId {
		MockSeasonId::from(id as u8)
	}

	fn create_transition_config(_id: u32) {}

	fn create_extra(_id: u32) {}
}

pub type SageInstance1 = pallet_sage::Instance1;
impl crate::Config<SageInstance1> for Test {
	type PalletId = ExamplePalletId;
	type SageGameTransition = ExampleTransitionGeneric<MockAccountId, MockAssetMediator>;
	type SeasonHandler = MockSeasonManager;
	type FeeHandler = GameFeeHandler<
		MockAccountId,
		Balances,
		MockAffiliatesFeeProvider,
		MockTournamentFeeProvider,
		MockTransitionFeeProvider,
		MockTreasuryManager,
	>;
	type FilterHandler = MockFilterHandler;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = SageBenchmarkHelper;
}

#[derive(Default)]
pub struct ExtBuilder {
	organizer: Option<MockAccountId>,
	locks: Vec<(MockAccountId, MockSeasonId, Locks)>,
	balances: Vec<(MockAccountId, MockBalance)>,
}

impl ExtBuilder {
	pub fn organizer(mut self, organizer: MockAccountId) -> Self {
		self.organizer = Some(organizer);
		self
	}

	pub fn locks(mut self, locks: &[(MockAccountId, MockSeasonId, Locks)]) -> Self {
		self.locks = locks.to_vec();
		self
	}

	pub fn balances(mut self, balances: &[(MockAccountId, MockBalance)]) -> Self {
		self.balances = balances.to_vec();
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
			let _ = Balances::deposit_creating(&TREASURER, MockExistentialDeposit::get());

			if let Some(organizer) = self.organizer {
				Organizer::<Test, Instance1>::put(organizer);
			}

			if !self.locks.is_empty() {
				for (account, season_id, lock) in self.locks {
					let config = PlayerConfig { inventory_tier: InventoryTier::One, locks: lock };
					PlayerSeasonConfigs::<Test, Instance1>::insert(account, season_id, config);
				}
			}

			// Setting initial general config for tests
			let config = GeneralConfig {
				transition: (),
				transfer: TransferConfig { open: true },
				trade: TradeConfig { open: true },
			};
			GeneralConfigStore::<Test, Instance1>::put(config);
		});
		ext
	}
}
