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

use crate::{self as pallet_ajuna_awesome_avatars, types::*, *};
use frame_support::{
	parameter_types,
	traits::{ConstU16, ConstU64, Hooks},
	PalletId,
};
pub(crate) use sp_runtime::testing::H256;
use sp_runtime::{
	testing::TestSignature,
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	BuildStorage,
};

pub type MockBlock = frame_system::mocking::MockBlock<Test>;
pub type MockSignature = TestSignature;
pub type MockAccountPublic = <MockSignature as Verify>::Signer;
pub type MockAccountId = <MockAccountPublic as IdentifyAccount>::AccountId;
pub type MockBlockNumber = u64;
pub type MockBalance = u64;
pub type MockNonce = u64;

pub const ALICE: MockAccountId = 1;
pub const BOB: MockAccountId = 2;
pub const CHARLIE: MockAccountId = 3;
pub const DAVE: MockAccountId = 4;

pub const SEASON_ID: SeasonId = 1;

frame_support::construct_runtime!(
	pub struct Test {
		System: frame_system = 0,
		Balances: pallet_balances = 1,
		Randomness: pallet_insecure_randomness_collective_flip = 2,
		AAvatars: pallet_ajuna_awesome_avatars = 4,
		Affiliates: pallet_ajuna_affiliates::<Instance1> = 6,
		Tournament: pallet_ajuna_tournament::<Instance1> = 7,
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
	type Nonce = MockNonce;
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

impl pallet_insecure_randomness_collective_flip::Config for Test {}

parameter_types! {
	pub const AwesomeAvatarsPalletId: PalletId = PalletId(*b"aj/aaatr");
}

impl pallet_ajuna_awesome_avatars::Config for Test {
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

pub type AffiliatesInstance1 = pallet_ajuna_affiliates::Instance1;
impl pallet_ajuna_affiliates::Config<AffiliatesInstance1> for Test {
	type RuntimeEvent = RuntimeEvent;
	type WhitelistKey = AffiliateWhitelistKey;
	type AccountManager = AAvatars;
	type RuleIdentifier = AffiliateMethods;
	type RuntimeRule = FeePropagationOf<Test>;
	type AffiliateMaxLevel = AffiliateMaxLevel;
}

parameter_types! {
	pub const TournamentPalletId1: PalletId = PalletId(*b"aj/trmt1");
	pub const MinimumTournamentPhaseDuration: MockBlockNumber = 100;
}

pub(crate) type TournamentInstance1 = pallet_ajuna_tournament::Instance1;
impl pallet_ajuna_tournament::Config<TournamentInstance1> for Test {
	type PalletId = TournamentPalletId1;
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type SeasonId = SeasonId;
	type EntityId = AvatarIdOf<Test>;
	type RankedEntity = AvatarOf<Test>;
	type MinimumTournamentPhaseDuration = MinimumTournamentPhaseDuration;
}

pub struct ExtBuilder {
	existential_deposit: MockBalance,
	organizer: Option<MockAccountId>,
	seasons: Vec<(SeasonId, Season<MockBlockNumber, MockBalance>)>,
	schedules: Vec<(SeasonId, SeasonSchedule<MockBlockNumber>)>,
	trade_filters: Vec<(SeasonId, TradeFilters)>,
	mint_cooldown: MockBlockNumber,
	balances: Vec<(MockAccountId, MockBalance)>,
	free_mints: Vec<(MockAccountId, MintCount)>,
	locks: Vec<(MockAccountId, SeasonId, Locks)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			existential_deposit: MockExistentialDeposit::get(),
			organizer: Default::default(),
			seasons: Default::default(),
			schedules: Default::default(),
			trade_filters: Default::default(),
			mint_cooldown: Default::default(),
			balances: Default::default(),
			free_mints: Default::default(),
			locks: Default::default(),
		}
	}
}

impl ExtBuilder {
	pub fn existential_deposit(mut self, existential_deposit: MockBalance) -> Self {
		self.existential_deposit = existential_deposit;
		self
	}
	pub fn organizer(mut self, organizer: MockAccountId) -> Self {
		self.organizer = Some(organizer);
		self
	}
	pub fn seasons(mut self, seasons: &[(SeasonId, Season<MockBlockNumber, MockBalance>)]) -> Self {
		self.seasons = seasons.to_vec();
		self
	}

	pub fn schedules(mut self, schedules: &[(SeasonId, SeasonSchedule<MockBlockNumber>)]) -> Self {
		self.schedules = schedules.to_vec();
		self
	}

	pub fn trade_filters(mut self, trade_filters: &[(SeasonId, TradeFilters)]) -> Self {
		self.trade_filters = trade_filters.to_vec();
		self
	}

	pub fn mint_cooldown(mut self, mint_cooldown: MockBlockNumber) -> Self {
		self.mint_cooldown = mint_cooldown;
		self
	}
	pub fn balances(mut self, balances: &[(MockAccountId, MockBalance)]) -> Self {
		self.balances = balances.to_vec();
		self
	}
	pub fn free_mints(mut self, free_mints: &[(MockAccountId, MintCount)]) -> Self {
		self.free_mints = free_mints.to_vec();
		self
	}

	pub fn locks(mut self, locks: &[(MockAccountId, SeasonId, Locks)]) -> Self {
		self.locks = locks.to_vec();
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		MOCK_EXISTENTIAL_DEPOSIT.with(|v| *v.borrow_mut() = self.existential_deposit);

		let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
		pallet_balances::GenesisConfig::<Test> { balances: self.balances }
			.assimilate_storage(&mut t)
			.unwrap();
		pallet_ajuna_awesome_avatars::GenesisConfig::<Test>::default()
			.assimilate_storage(&mut t)
			.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext.execute_with(|| {
			if let Some(organizer) = self.organizer {
				Organizer::<Test>::put(organizer);
			}

			for (season_id, season) in self.seasons {
				Seasons::<Test>::insert(season_id, season);
			}

			for (season_id, season_schedule) in self.schedules {
				SeasonSchedules::<Test>::insert(season_id, season_schedule);
			}

			for (season_id, trade_filters) in self.trade_filters {
				SeasonTradeFilters::<Test>::insert(season_id, trade_filters);
			}

			GlobalConfigs::<Test>::mutate(|config| {
				config.mint.open = true;
				config.forge.open = true;
				config.trade.open = true;
				config.mint.cooldown = self.mint_cooldown;
			});

			for (account_id, mint_amount) in self.free_mints {
				PlayerConfigs::<Test>::mutate(account_id, |account| {
					account.free_mints = mint_amount
				});
			}

			if !self.locks.is_empty() {
				for (account, season_id, lock) in self.locks {
					let config = PlayerSeasonConfig::<BlockNumberFor<Test>> {
						storage_tier: Default::default(),
						stats: Default::default(),
						locks: lock,
					};
					PlayerSeasonConfigs::<Test>::insert(account, season_id, config);
				}
			}
		});
		ext
	}
}

pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		if System::block_number() > 1 {
			AAvatars::on_finalize(System::block_number());
			Tournament::on_finalize(System::block_number());
			System::on_finalize(System::block_number());
		}
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		AAvatars::on_initialize(System::block_number());
		Tournament::on_initialize(System::block_number());
	}
}
