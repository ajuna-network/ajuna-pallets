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
	chain_inspector::ChainInspector,
	payment_handler::{
		AffiliateFeeDistribution, AllowAllAssets, AssetGameFeeHandler, DistributeFee, PaymentFee,
		VoucherHandler, WithdrawCreditOrVoucher, WithdrawFungibles, WithdrawKind,
		WithdrawWhitelistedCredit,
	},
	season_manager::{SeasonConfig, SeasonFeeConfig, SeasonManager},
	trade_manager::TradeManager,
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
	BuildStorage, DispatchError, SaturatedConversion,
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
	type ForceOrigin = frame_system::EnsureRoot<MockAccountId>;
	type Freezer = ();
	type CallbackHandle = ();
}

pub type NativeAndAssets =
	UnionOf<Balances, PalletAssets, NativeFromLeft, NativeOrWithId<u32>, MockAccountId>;
pub const NATIVE_PAYMENT: WithdrawKind<NativeOrWithId<u32>> =
	WithdrawKind::Payment(NativeOrWithId::Native);
pub const SOME_NATIVE_PAYMENT: Option<WithdrawKind<NativeOrWithId<u32>>> = Some(NATIVE_PAYMENT);

use example_transition::{
	asset::{
		hero_jam::{AssetSubType, AssetType, HeroJamAsset, StateType},
		Asset, AssetId, AssetVariant,
		AssetVariant::HeroJam,
	},
	transition::{hero_jam::HeroAction, GameTransition, TransitionIdentifier},
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

pub type MockAsset = Asset<BlockNumberFor<Test>>;

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

pub type MockFilter = AssetType;

impl TradeManager for MockFilterHandler {
	type TradeFilter = MockFilter;
	type Asset = MockAsset;

	fn can_be_traded_using(asset: &Self::Asset, filter: &Self::TradeFilter) -> bool {
		match asset.asset_variant {
			AssetVariant::HeroJam(hero_jam_asset) => hero_jam_asset.asset_type == *filter,
		}
	}
}

impl TransferManager for MockFilterHandler {
	type TransferFilter = MockFilter;
	type Asset = MockAsset;

	fn can_be_transferred_using(asset: &Self::Asset, filter: &Self::TransferFilter) -> bool {
		match asset.asset_variant {
			AssetVariant::HeroJam(hero_jam_asset) => hero_jam_asset.asset_type == *filter,
		}
	}
}

pub struct MockAssetMediator;

impl AssetManager for MockAssetMediator {
	type AccountId = MockAccountId;
	type AssetId = AssetId;
	type Asset = MockAsset;

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
}

impl AssetInspector for MockAssetMediator {
	type AccountId = MockAccountId;
	type AssetId = AssetId;
	type Asset = MockAsset;

	fn get_asset(asset_id: &Self::AssetId) -> Result<Self::Asset, DispatchError> {
		<Sage as AssetInspector>::get_asset(asset_id)
	}

	fn iter_assets_from(account_id: &Self::AccountId) -> impl Iterator<Item = Self::AssetId> {
		<Sage as AssetInspector>::iter_assets_from(account_id)
	}
}

impl ChainInspector for MockAssetMediator {
	type BlockNumber = BlockNumberFor<Test>;

	fn get_current_block_number() -> Self::BlockNumber {
		System::block_number()
	}
}

#[cfg(feature = "runtime-benchmarks")]
pub struct SageBenchmarkHelper;

#[cfg(feature = "runtime-benchmarks")]
impl
	BenchmarkHelper<
		MockAccountId,
		MockSeasonId,
		AssetId,
		MockAsset,
		TransitionIdentifier,
		WithdrawKind<NativeOrWithId<u32>>,
	> for SageBenchmarkHelper
{
	fn create_asset_for(account: &MockAccountId, season_id: &MockSeasonId, seed: u32) -> AssetId {
		let asset_id = AssetId::from(seed);
		let asset = Asset {
			asset_variant: HeroJam(HeroJamAsset {
				id: asset_id,
				asset_type: AssetType::None,
				asset_subtype: AssetSubType::None,
				energy: 0,
				fatigue: 0,
				state_type: StateType::None,
				state_sub_type: 0,
				state_sub_value: 0,
				state_change_block_number: 0_u32.saturated_into(),
				balance: 10,
			}),
		};

		MockSeasonManager::register_asset_in(&asset_id, season_id)
			.expect("Asset should be registered");
		Assets::<Test, ()>::insert(asset_id, (account, asset));
		AssetOwners::<Test, ()>::insert((account, season_id, &asset_id), ());

		asset_id
	}

	fn create_bench_transition_for(
		_account: &MockAccountId,
		_season: &MockSeasonId,
		_seed: u32,
	) -> (TransitionIdentifier, Vec<AssetId>) {
		(TransitionIdentifier::HeroJam(HeroAction::Create), vec![])
	}

	fn create_payment_kind() -> WithdrawKind<NativeOrWithId<u32>> {
		WithdrawKind::Payment(NativeOrWithId::Native)
	}
}

pub struct MockVoucherHandler;

impl VoucherHandler for MockVoucherHandler {
	type AccountId = MockAccountId;
	type Balance = MockBalance;

	fn consume_vouchers_from(
		_account: &Self::AccountId,
		_amount: Self::Balance,
	) -> Result<(), DispatchError> {
		Ok(())
	}
}

pub type GameTransitionOf =
	GameTransition<MockAccountId, BlockNumberFor<Test>, MockAssetMediator, MockAssetMediator>;

impl crate::Config for Test {
	type PalletId = ExamplePalletId;
	type SageGameTransition = GameTransitionOf;
	type SeasonHandler = MockSeasonManager;
	type FeeHandler = AssetGameFeeHandler<
		MockAccountId,
		NativeAndAssets,
		WithdrawCreditOrVoucher<
			WithdrawWhitelistedCredit<
				AllowAllAssets<NativeOrWithId<u32>>,
				WithdrawFungibles<MockAccountId, NativeAndAssets>,
			>,
			MockVoucherHandler,
		>,
		TestAffiliatesFeeProvider,
		TestAffiliatesMaxDistribution,
		TestTournamentFeeProvider,
	>;
	type PaymentKind = WithdrawKind<NativeOrWithId<u32>>;
	type FilterHandler = MockFilterHandler;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = SageBenchmarkHelper;
}

pub struct TestAffiliatesFeeProvider;

pub type TestAffiliatesMaxDistribution = ConstU32<3>;

impl DistributeFee for TestAffiliatesFeeProvider {
	type AccountId = MockAccountId;
	type Balance = MockBalance;
	type FeeIdentifier = AffiliateMethods<TransitionIdentifier>;
	type FeeDistribution =
		AffiliateFeeDistribution<Self::AccountId, Self::Balance, TestAffiliatesMaxDistribution>;

	fn distribute_fee(
		_base_fee: Self::Balance,
		_account: &Self::AccountId,
		identifier: &Self::FeeIdentifier,
	) -> Option<Self::FeeDistribution> {
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
	type FeeDistribution = PaymentFee<Self::AccountId, Self::Balance>;

	fn distribute_fee(
		base_fee: Self::Balance,
		_account: &Self::AccountId,
		identifier: &Self::FeeIdentifier,
	) -> Option<Self::FeeDistribution> {
		match *identifier {
			PAYING => Some(PaymentFee::new(TOURNAMENT_TREASURY, base_fee * 5 / 10)),
			FREE => None,
			_ => panic!("Did not identify free or paying"),
		}
	}
}

#[derive(Default)]
pub struct ExtBuilder {
	organizer: Option<MockAccountId>,
	locks: Vec<(MockAccountId, MockSeasonId, Locks)>,
	balances: Vec<(MockAccountId, MockBalance)>,
	vouchers: Vec<(MockAccountId, MockBalance)>,
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

	pub fn vouchers(mut self, vouchers: &[(MockAccountId, MockBalance)]) -> Self {
		self.vouchers = vouchers.to_vec();
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

pub fn new_test_ext() -> sp_io::TestExternalities {
	ExtBuilder::default().build()
}
