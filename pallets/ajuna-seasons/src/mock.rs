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

use crate::{self as pallet_ajuna_seasons, *};
use frame_support::{
	parameter_types,
	traits::{ConstU16, ConstU64},
};
use sp_runtime::BuildStorage;

use ajuna_primitives::account_manager::WhitelistKey;
use sp_runtime::{
	testing::H256,
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	MultiSignature,
};
use sp_std::cell::RefCell;

pub type MockSignature = MultiSignature;
pub type MockAccountPublic = <MockSignature as Verify>::Signer;
pub type MockAccountId = <MockAccountPublic as IdentifyAccount>::AccountId;
pub type MockBlock = frame_system::mocking::MockBlock<Test>;
pub type MockBalance = u64;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub struct Test {
		System: frame_system = 0,
		Balances: pallet_balances = 1,
		SeasonsAlpha: pallet_ajuna_seasons = 2,
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

pub type MockAssetId = u32;

thread_local! {
	pub static ORGANIZER: RefCell<Option<MockAccountId>> = const { RefCell::new(None) };
}

pub struct MockAccountManager;

impl MockAccountManager {
	pub fn set_organizer(owner: MockAccountId) {
		ORGANIZER.with(|maybe_account| {
			*maybe_account.borrow_mut() = Some(owner);
		});
	}
}

pub const ACCOUNT_IS_NOT_ORGANIZER: &str = "ACCOUNT_IS_NOT_ORGANIZER";
pub const NO_ORGANIZER_SET: &str = "NO_ORGANIZER_SET";

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

pub type MockSeasonId = u32;

#[cfg(feature = "runtime-benchmarks")]
pub struct SeasonsBenchmarkHelper;

#[cfg(feature = "runtime-benchmarks")]
impl BenchmarkHelper<MockSeasonId> for SeasonsBenchmarkHelper {
	fn create_season_id(id: u32) -> MockSeasonId {
		MockSeasonId::from(id)
	}
}

impl Config for Test {
	type SeasonId = MockSeasonId;
	type AssetId = MockAssetId;
	type AccountHandler = MockAccountManager;
	type Currency = Balances;
	type WeightInfo = ();
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = SeasonsBenchmarkHelper;
}

#[cfg(test)]
#[derive(Default)]
pub struct ExtBuilder {
	organizer: Option<MockAccountId>,
}

#[cfg(test)]
impl ExtBuilder {
	pub fn organizer(mut self, organizer: MockAccountId) -> Self {
		self.organizer = Some(organizer);
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let config = RuntimeGenesisConfig {
			system: Default::default(),
			balances: Default::default(),
			seasons_alpha: Default::default(),
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
			SeasonsAlpha::on_finalize(System::block_number());
		}
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		SeasonsAlpha::on_initialize(System::block_number());
	}
}
