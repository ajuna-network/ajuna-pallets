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
	payment_handler::{
		AffiliateFeeDistribution, AllowAllAssets, AssetGameFeeHandler, DistributeFee, PaymentFee,
		TransferFungibleAssets, VoucherHandler, WithdrawCreditOrVoucher, WithdrawFungibles,
		WithdrawKind, WithdrawWhitelistedCredit,
	},
	sage_api::SageApi,
	season_manager::{SeasonConfig, SeasonFeeConfig, SeasonManager},
};

use ajuna_primitives::next_asset_id_provider::IncrementingAssetIdProvider;
use frame_support::{
	parameter_types,
	traits::{
		fungible::{Mutate, NativeOrWithId},
		AsEnsureOriginWithArg,
	},
	PalletId,
};
use sp_core::H256;
use sp_runtime::{
	BuildStorage, DispatchError,
};
use sp_std::{cell::RefCell, collections::btree_map::BTreeMap};
use ajuna_primitives::runtime_types::AccountId;
use sage_testing::*;


pub fn alice() -> AccountId {
	AccountKeyring::Alice.to_account_id()
}
pub fn bob() -> AccountId {
	AccountKeyring::Bob.to_account_id()
}
pub fn charlie() -> AccountId {
	AccountKeyring::Charlie.to_account_id()
}
pub fn dave() -> AccountId {
	AccountKeyring::Dave.to_account_id()
}

pub const TOURNAMENT_TREASURY: AccountId = AccountId::new([9; 32]);

pub const SEASON_ID_0: SeasonId = 0;
pub const SEASON_ID_1: SeasonId = 1;

pub const MAIN_ASSET_ID: AssetId = 0;
pub const LOW_LIQUIDITY_ASSET_ID: AssetId = 99;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub struct Test {
		System: frame_system = 0,
		Balances: pallet_balances = 1,
		PalletAssets: pallet_assets = 2,
		Randomness: pallet_insecure_randomness_collective_flip = 3,
		Sage: pallet_sage = 4,
	}
);

impl_core_pallets!(Test, System);

impl pallet_insecure_randomness_collective_flip::Config for Test {}

pub type NativeAndAssets = NativeAndAssetsG<Balances, PalletAssets>;
pub const NATIVE_PAYMENT: WithdrawKind<NativeOrWithId<AssetId>> =
	WithdrawKind::Payment(NativeOrWithId::Native);
pub const SOME_NATIVE_PAYMENT: Option<WithdrawKind<NativeOrWithId<AssetId>>> = Some(NATIVE_PAYMENT);

use example_transition::{prelude::*, transition::CasinoJamTransitionConfig};

parameter_types! {
	pub const ExamplePalletId: PalletId = PalletId(*b"sage/exi");
}

thread_local! {
	pub static ASSET_SEEDS: RefCell<u64> = const { RefCell::new(0) };
	pub static ASSET_SEASONS: RefCell<BTreeMap<AssetId, SeasonId>> = const { RefCell::new(BTreeMap::new()) };
	pub static CURRENT_SEASON: RefCell<SeasonId> = const { RefCell::new(SEASON_ID_0) }
}

pub struct MockSeasonManager;

pub type MockAsset = Asset<BlockNumberFor<Test>>;
pub type MockTransitionId = CasinoAction;

impl SeasonManager for MockSeasonManager {
	type SeasonId = SeasonId;
	type AssetId = AssetId;
	type Balance = Balance;

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
	) -> Result<SeasonConfig<Self::Balance>, DispatchError> {
		Ok(SeasonConfig::<Self::Balance> {
			fee: SeasonFeeConfig::<Self::Balance> {
				transfer_asset: ExistentialDeposit::get(),
				buy_asset_min: ExistentialDeposit::get(),
				buy_percent: 1,
				upgrade_asset_inventory: ExistentialDeposit::get(),
				unlock_trade_asset: ExistentialDeposit::get(),
				unlock_transfer_asset: ExistentialDeposit::get(),
				state_transition_base_fee: ExistentialDeposit::get(),
			},
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

pub struct TestSageEngine;
// The macro can't handle the brackets.

// Every new game we add can simply call that macro for another sage instance to
// implement the sage api given that the other types are identical.
impl_test_runtime_sage_api!(
	TestSageEngine,
	Test,
	DefaultSageInstance,
	MockSeasonManager,
	AssetId,
	MockAsset,
	CasinoJamTransitionConfig,
	Randomness,
	H256
);

pub struct MockVoucherHandler;

impl VoucherHandler for MockVoucherHandler {
	type AccountId = AccountId;
	type Balance = Balance;

	fn consume_vouchers_from(
		_account: &Self::AccountId,
		_amount: Self::Balance,
	) -> Result<(), DispatchError> {
		Ok(())
	}
}

pub type GameTransitionOf =
	CasinoJamTransition<AccountId, BlockNumberFor<Test>, TestSageEngine>;

pub type WithdrawAllCreditOrVoucher = WithdrawCreditOrVoucher<
	WithdrawWhitelistedCredit<
		AllowAllAssets<NativeOrWithId<AssetId>>,
		WithdrawFungibles<AccountId, NativeAndAssets>,
	>,
	MockVoucherHandler,
>;

type FungiblesAssetId = WithdrawKind<NativeOrWithId<AssetId>>;

// Can't use that currently, as our example game transition uses u32.
// type TestAssetIdProvider = RandomAssetIdProvider<AssetId, BlockNumberFor<Test>, Randomness>;
type TestAssetIdProvider = IncrementingAssetIdProvider<AssetId>;

type DefaultSageInstance = ();
impl crate::Config for Test {
	type PalletId = ExamplePalletId;
	type SageGameTransition = GameTransitionOf;
	type NextAssetIdProvider = TestAssetIdProvider;
	type SeasonHandler = MockSeasonManager;
	type FeeHandler = AssetGameFeeHandler<
		AccountId,
		NativeAndAssets,
		WithdrawAllCreditOrVoucher,
		TestAffiliatesFeeProvider,
		TestAffiliatesMaxDistribution,
		TestTournamentFeeProvider,
	>;
	type TransferFunds = TransferFungibleAssets<WithdrawAllCreditOrVoucher, FungiblesAssetId>;
	type FungiblesAssetId = FungiblesAssetId;
	type FilterHandler = GameFilter<BlockNumberFor<Test>>;
	type Fungible = Balances;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = GameBenchmarkHelper<BlockNumberFor<Test>>;
}

pub struct TestAffiliatesFeeProvider;

pub type TestAffiliatesMaxDistribution = ConstU32<3>;

impl DistributeFee for TestAffiliatesFeeProvider {
	type AccountId = AccountId;
	type Balance = Balance;
	type FeeIdentifier = AffiliateMethods<MockTransitionId>;
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

pub const PAYING: u32 = 0;
pub const FREE: u32 = 1;

impl DistributeFee for TestTournamentFeeProvider {
	type AccountId = AccountId;
	type Balance = Balance;
	type FeeIdentifier = SeasonId;
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
	organizer: Option<AccountId>,
	locks: Vec<(AccountId, SeasonId, Locks)>,
	balances: Vec<(AccountId, Balance)>,
	vouchers: Vec<(AccountId, Balance)>,
}

impl ExtBuilder {
	pub fn organizer(mut self, organizer: AccountId) -> Self {
		self.organizer = Some(organizer);
		self
	}

	pub fn locks(mut self, locks: &[(AccountId, SeasonId, Locks)]) -> Self {
		self.locks = locks.to_vec();
		self
	}

	pub fn balances(mut self, balances: &[(AccountId, Balance)]) -> Self {
		self.balances = balances.to_vec();
		self
	}

	pub fn vouchers(mut self, vouchers: &[(AccountId, Balance)]) -> Self {
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
					(MAIN_ASSET_ID, alice(), true, 1),
					(LOW_LIQUIDITY_ASSET_ID, alice(), true, 1),
				],
				metadata: vec![
					// id, name, symbol, decimals
					(MAIN_ASSET_ID, "Main Asset".into(), "MAIN".into(), 10),
					(LOW_LIQUIDITY_ASSET_ID, "Main Asset".into(), "MAIN".into(), 10),
				],
				accounts: vec![
					// id, account_id, balance
					(MAIN_ASSET_ID, alice(), 100),
					(MAIN_ASSET_ID, bob(), 100),
					(MAIN_ASSET_ID, charlie(), 100),
					(MAIN_ASSET_ID, dave(), 100),
					(LOW_LIQUIDITY_ASSET_ID, alice(), 1),
				],
				next_asset_id: None,
			},
			sage: GenesisConfig { organizer: None, season: None },
		};

		let mut ext: sp_io::TestExternalities = config.build_storage().unwrap().into();
		ext.execute_with(|| System::set_block_number(1));
		ext.execute_with(|| {
			let _ = Balances::set_balance(&TOURNAMENT_TREASURY, ExistentialDeposit::get());

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
