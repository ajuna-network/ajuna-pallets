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
	fee_handler::AssetGameFeeHandler,
	season_manager::{SeasonConfig, SeasonFeeConfig, SeasonManager},
	trade_manager::TradeManager,
};

use ajuna_primitives::fee_handler::{
	AllowAllAssets, DistributeFee, Payment, WithdrawFungibles, WithdrawWhitelistedCredit,
};
use frame_support::{
	derive_impl, parameter_types,
	traits::{
		fungible::{NativeFromLeft, NativeOrWithId, UnionOf},
		AsEnsureOriginWithArg,
	},
	PalletId,
};
use sp_runtime::{
	testing::TestSignature,
	traits::{IdentifyAccount, Verify},
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

pub const TOURNAMENT_TREASURY: MockAccountId = 431;

pub const SEASON_ID_0: MockSeasonId = 0;
pub const SEASON_ID_1: MockSeasonId = 1;

pub const MAIN_ASSET_ID: u32 = 0;
pub const LOW_LIQUIDITY_ASSET_ID: u32 = 99;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub struct Test {
		System: frame_system = 0,
		Balances: pallet_balances = 1,
		PalletAssets: pallet_assets = 2,
		Sage: pallet_sage = 3,
	}
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type AccountId = MockAccountId;
	type AccountData = pallet_balances::AccountData<MockBalance>;
	type Block = MockBlock;
}

parameter_types! {
	pub const MockExistentialDeposit: MockBalance = 3;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
	type AccountStore = System;
	type ExistentialDeposit = MockExistentialDeposit;
}

#[derive_impl(pallet_assets::config_preludes::TestDefaultConfig)]
impl pallet_assets::Config for Test {
	type Currency = Balances;
	type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<MockAccountId>>;
	type ForceOrigin = frame_system::EnsureRoot<u64>;
	type Freezer = ();
	type CallbackHandle = ();
}

pub type NativeAndAssets =
	UnionOf<Balances, PalletAssets, NativeFromLeft, NativeOrWithId<u32>, MockAccountId>;
pub const NATIVE: NativeOrWithId<u32> = NativeOrWithId::Native;

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

pub struct MockFilterHandler;

pub type MockFilter = u32;

impl TradeManager for MockFilterHandler {
	type TradeFilter = MockFilter;
	type Asset = Asset;

	fn can_be_traded_using(asset: &Self::Asset, filter: &Self::TradeFilter) -> bool {
		asset.asset_type == *filter
	}
}

impl TransferManager for MockFilterHandler {
	type TransferFilter = MockFilter;
	type Asset = Asset;

	fn can_be_transferred_using(asset: &Self::Asset, filter: &Self::TransferFilter) -> bool {
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
impl BenchmarkHelper<MockAccountId, MockSeasonId, AssetId, Asset, ExampleTransitionId>
	for SageBenchmarkHelper
{
	fn create_asset_for(account: &MockAccountId, season_id: &MockSeasonId, seed: u32) -> AssetId {
		let asset_id = AssetId::from_low_u64_le(seed as u64);
		let asset = Asset::create(asset_id, 0, 0, 0, [seed as u8; 32], 1, Level::One);

		MockSeasonManager::register_asset_in(&asset_id, season_id)
			.expect("Asset should be registered");
		Assets::<Test, ()>::insert(asset_id, (account, asset));
		AssetOwners::<Test, ()>::insert((account, season_id, &asset_id), ());

		asset_id
	}

	fn create_bench_transition_for(
		account: &MockAccountId,
		season: &MockSeasonId,
		seed: u32,
	) -> (ExampleTransitionId, Vec<AssetId>) {
		let asset_id_1 = Self::create_asset_for(account, season, seed);
		let asset_id_2 = Self::create_asset_for(account, season, seed * 2);
		let asset_id_3 = Self::create_asset_for(account, season, seed * 3);
		let asset_id_4 = Self::create_asset_for(account, season, seed * 4);
		let asset_id_5 = Self::create_asset_for(account, season, seed * 5);

		let asset_vec = vec![asset_id_1, asset_id_2, asset_id_3, asset_id_4, asset_id_5];

		(ExampleTransitionId::BenchTransition, asset_vec)
	}
}

impl crate::Config for Test {
	type PalletId = ExamplePalletId;
	type SageGameTransition = ExampleTransitionGeneric<MockAccountId, MockAssetMediator>;
	type SeasonHandler = MockSeasonManager;
	type FeeHandler = AssetGameFeeHandler<
		MockAccountId,
		NativeAndAssets,
		WithdrawWhitelistedCredit<
			AllowAllAssets<NativeOrWithId<u32>>,
			WithdrawFungibles<NativeAndAssets, MockAccountId>,
		>,
		TestAffiliatesFeeProvider,
		TestTournamentFeeProvider,
	>;
	type PaymentAssetId = NativeOrWithId<u32>;
	type FilterHandler = MockFilterHandler;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = SageBenchmarkHelper;
}

pub struct TestAffiliatesFeeProvider;

impl DistributeFee for TestAffiliatesFeeProvider {
	type AccountId = MockAccountId;
	type Balance = MockBalance;
	type FeeIdentifier = AffiliateMethods<ExampleTransitionId>;
	type MaxDistributions = ConstU32<3>;

	fn distribute_fee(
		_base_fee: Self::Balance,
		_account: &Self::AccountId,
		identifier: &Self::FeeIdentifier,
	) -> Option<BoundedVec<Payment<Self::AccountId, Self::Balance>, Self::MaxDistributions>> {
		match identifier {
			AffiliateMethods::UpgradeAssetInventory => None,
			AffiliateMethods::TradeAsset => None,
			AffiliateMethods::StateTransition(_) => None,
		}
	}
}

pub struct TestTournamentFeeProvider;

pub enum TournamentFeeId {
	Paying,
	Free,
}

pub const PAYING: u8 = 0;
pub const FREE: u8 = 1;

impl DistributeFee for TestTournamentFeeProvider {
	type AccountId = MockAccountId;
	type Balance = MockBalance;
	type FeeIdentifier = MockSeasonId;
	type MaxDistributions = ConstU32<1>;

	fn distribute_fee(
		base_fee: Self::Balance,
		_account: &Self::AccountId,
		identifier: &Self::FeeIdentifier,
	) -> Option<BoundedVec<Payment<Self::AccountId, Self::Balance>, Self::MaxDistributions>> {
		match identifier {
			&PAYING => Some(
				vec![Payment::new(TOURNAMENT_TREASURY, base_fee * 5 / 10)]
					.try_into()
					.expect("max distribution = 1; qed"),
			),
			&FREE => None,
			_ => panic!("Did not identify free or paying"),
		}
	}
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
			pallet_assets: pallet_assets::GenesisConfig {
				assets: vec![
					// id, owner, is_sufficient, min_balance
					(MAIN_ASSET_ID, ALICE, true, 1),
					(LOW_LIQUIDITY_ASSET_ID, ALICE, true, 1),
				],
				metadata: vec![
					// id, name, symbol, decimals
					(MAIN_ASSET_ID, "Main Asset".into(), "MAIN".into(), 10),
					(LOW_LIQUIDITY_ASSET_ID, "Main Asset".into(), "MAIN".into(), 10),
				],
				accounts: vec![
					// id, account_id, balance
					(MAIN_ASSET_ID, ALICE, 100),
					(MAIN_ASSET_ID, BOB, 100),
					(MAIN_ASSET_ID, CHARLIE, 100),
					(MAIN_ASSET_ID, DAVE, 100),
					(LOW_LIQUIDITY_ASSET_ID, ALICE, 1),
				],
				next_asset_id: None,
			},
		};

		let mut ext: sp_io::TestExternalities = config.build_storage().unwrap().into();
		ext.execute_with(|| System::set_block_number(1));
		ext.execute_with(|| {
			let _ = Balances::deposit_creating(&TOURNAMENT_TREASURY, MockExistentialDeposit::get());

			if let Some(organizer) = self.organizer {
				Organizer::<Test, ()>::put(organizer);
			}

			if !self.locks.is_empty() {
				for (account, season_id, lock) in self.locks {
					let config = PlayerConfig { inventory_tier: InventoryTier::One, locks: lock };
					PlayerSeasonConfigs::<Test, ()>::insert(account, season_id, config);
				}
			}

			// Setting initial general config for tests
			let config = GeneralConfig {
				transition: (),
				transfer: TransferConfig { open: true },
				trade: TradeConfig { open: true },
			};
			GeneralConfigStore::<Test, ()>::put(config);
		});
		ext
	}
}

pub fn new_benchmark_ext() -> sp_io::TestExternalities {
	use crate::mock::{Balances, RuntimeOrigin, System, Test, SEASON_ID_0};

	let t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
	let mut ext = sp_io::TestExternalities::new(t);
	ext.execute_with(|| System::set_block_number(1));
	ext.execute_with(|| {
		GeneralConfigStore::<Test, ()>::mutate(|config| {
			config.trade.open = true;
			config.transfer.open = true;
		});
		SeasonUnlocks::<Test, ()>::mutate(SEASON_ID_0, LockableFeature::TradeAsset, |rule| {
			*rule = Some(UnlockRule::from([0, 0, 0, 0, 0]));
		});
		SeasonUnlocks::<Test, ()>::mutate(SEASON_ID_0, LockableFeature::TransferAsset, |rule| {
			*rule = Some(UnlockRule::from([0, 0, 0, 0, 0]));
		});

		let acc_1 = crate::benchmarking::account::<Test, ()>(crate::benchmarking::ACC_1);
		Organizer::<Test, ()>::put(acc_1);
		PlayerSeasonConfigs::<Test, ()>::mutate(acc_1, SEASON_ID_0, |config| {
			config.locks = Locks::all_unlocked();
		});
		Balances::force_set_balance(RuntimeOrigin::root(), acc_1, 100_000)
			.expect("Should set balance");

		let acc_2 = crate::benchmarking::account::<Test, ()>(crate::benchmarking::ACC_2);
		PlayerSeasonConfigs::<Test, ()>::mutate(acc_2, SEASON_ID_0, |config| {
			config.locks = Locks::all_unlocked();
		});
		Balances::force_set_balance(RuntimeOrigin::root(), acc_2, 100_000)
			.expect("Should set balance");
	});
	ext
}
