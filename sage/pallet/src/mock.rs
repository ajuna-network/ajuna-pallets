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
	}
);

impl frame_system::Config for Test {
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
	pub const MockExistentialDeposit: MockBalance = 3;
}

impl pallet_balances::Config for Test {
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

use example_transition::{
	generic::ExampleTransitionGeneric,
	types::{Asset, AssetId},
};
use sage_api::SageApi;

pub struct SageMock;

/// For now we implement this manually for every game so that we can delegate the
/// call to the instance of the account manager etc. corresponding to the game this is
/// implemented for.
///
/// Later we can hopefully do a blanket implementation for a struct that will automatically
/// implement the sage api that looks like this:
///
/// ```rust
/// pub type ExampleGameSage = SageCore<AssetManager, FeeManager>;
/// ```
/// `ExampleGameSage` will then automatically implement `SageApi` if the `AssetManager` and
/// `FeeManager` implement their corresponding traits.
impl SageApi for SageMock {
	type AssetId = AssetId;
	type Asset = Asset;
	type Balance = MockBalance;
	type AccountId = MockAccountId;

	fn ensure_ownership(
		_account: &Self::AccountId,
		_asset: &Self::AssetId,
	) -> Result<(), sage_api::Error> {
		// this would be a call to our asset manager implementation
		todo!()
	}

	fn try_mutate_asset<R, F: FnOnce(&mut Self::Asset) -> Result<R, sage_api::Error>>(
		_asset: &Self::AssetId,
		_f: F,
	) -> Result<R, sage_api::Error> {
		// this would be a call to our asset manager implementation
		todo!()
	}

	fn transfer_ownership(
		_asset: Self::AssetId,
		_to: Self::AccountId,
	) -> Result<(), sage_api::Error> {
		// this would be a call to our asset manager implementation
		todo!()
	}

	fn handle_fees(_balance: Self::Balance) -> Result<(), sage_api::Error> {
		// this would be a call to our fee handler implementation
		// This call will probably have more arguments, or there are multiple
		// handle fee variants depending on what we will come up with in the
		// fee manager.
		todo!()
	}
}

parameter_types! {
	pub const ExamplePalletId: PalletId = PalletId(*b"sage/exi");
}

pub struct MockSeasonManager;

pub type MockSeasonId = u8;

impl SeasonManager for MockSeasonManager {
	type SeasonId = MockSeasonId;
	type AssetId = AssetId;
	type Balance = MockBalance;

	fn get_season_id_for(_asset: &Self::AssetId) -> Self::SeasonId {
		MockSeasonId::default()
	}

	fn get_current_season_id() -> Self::SeasonId {
		MockSeasonId::default()
	}

	fn is_valid_season(_season_id: &Self::SeasonId) -> Result<(), DispatchError> {
		Ok(())
	}

	fn get_season_config_for(
		_season_id: &Self::SeasonId,
	) -> Result<SeasonConfig<Self::Balance>, DispatchError> {
		Ok(SeasonConfig::<Self::Balance> {
			fee: SeasonFeeConfig::<Self::Balance> {
				transfer_asset: MockExistentialDeposit::get(),
				buy_asset_min: MockExistentialDeposit::get(),
				buy_percent: 1,
				upgrade_asset_inventory: MockExistentialDeposit::get(),
				unlock_trade_asset: MockExistentialDeposit::get(),
				unlock_transfer_asset: MockExistentialDeposit::get(),
				state_transition: MockExistentialDeposit::get(),
			},
		})
	}
}

pub struct MockAffiliatesFeeProvider;

impl FeeProvider for MockAffiliatesFeeProvider {
	type AccountId = MockAccountId;
	type FeeIdentifier = AffiliateMethods;
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
		base_fee: Self::FeeCurrency,
		account: &Self::AccountId,
		_identifier: &Self::FeeIdentifier,
	) -> Self::FeeOutput {
		(base_fee, *account)
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

pub struct MockTradeHandler;

pub type MockTradeFilter = u8;

impl TradeManager for MockTradeHandler {
	type TradeFilter = MockTradeFilter;
	type Asset = Asset;

	fn is_tradeable_using(_asset: &Self::Asset, _filter: &Self::TradeFilter) -> bool {
		todo!()
	}
}

pub type SageInstance1 = pallet_sage::Instance1;
impl crate::Config<SageInstance1> for Test {
	type PalletId = ExamplePalletId;
	type SageGameTransition = ExampleTransitionGeneric<MockAccountId, MockBalance, SageMock>;
	type SageApi = SageMock;
	type SeasonHandler = MockSeasonManager;
	type FeeHandler = GameFeeHandler<
		MockAccountId,
		Balances,
		MockAffiliatesFeeProvider,
		MockTournamentFeeProvider,
		MockTreasuryManager,
	>;
	type TradeHandler = MockTradeHandler;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
}

#[derive(Default)]
pub struct ExtBuilder {
	balances: Vec<(MockAccountId, MockBalance)>,
	organizer: Option<MockAccountId>,
}

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
			let _ = Balances::deposit_creating(&TREASURER, MockExistentialDeposit::get());

			if let Some(organizer) = self.organizer {
				Organizer::<Test, Instance1>::put(organizer);
			}
		});
		ext
	}
}
