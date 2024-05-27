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

use crate::{self as pallet_ajuna_new_omega, types::*, *};
use frame_support::{
	parameter_types,
	traits::{ConstU16, ConstU64, Hooks},
};
pub(crate) use sp_runtime::testing::H256;
use sp_runtime::{
	testing::TestSignature,
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	BuildStorage,
};

pub use utils::*;

pub type MockBlock = frame_system::mocking::MockBlock<Test>;
pub type MockSignature = TestSignature;
pub type MockAccountPublic = <MockSignature as Verify>::Signer;
pub type MockAccountId = <MockAccountPublic as IdentifyAccount>::AccountId;
pub type MockBalance = u64;
pub type MockNonce = u64;

pub const ALICE: MockAccountId = 1;
pub const BOB: MockAccountId = 2;
pub const CHARLIE: MockAccountId = 3;
pub const DAVE: MockAccountId = 4;

frame_support::construct_runtime!(
	pub struct Test {
		System: frame_system = 0,
		Balances: pallet_balances = 1,
		Random: pallet_insecure_randomness_collective_flip = 2,
		Omega: pallet_ajuna_new_omega = 3,
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
	pub static MockExistentialDeposit: MockBalance = 10;
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

impl pallet_ajuna_new_omega::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type Randomness = Random;
}
#[derive(Default)]
pub struct ExtBuilder {
	balances: Vec<(MockAccountId, MockBalance)>,
}

impl ExtBuilder {
	pub fn balances(mut self, balances: &[(MockAccountId, MockBalance)]) -> Self {
		self.balances = balances.to_vec();
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
		pallet_balances::GenesisConfig::<Test> { balances: self.balances }
			.assimilate_storage(&mut t)
			.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| run_to_block(1));
		ext
	}
}

pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		if System::block_number() > 1 {
			//Omega::on_finalize(System::block_number());
			System::on_finalize(System::block_number());
		}
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		//Omega::on_initialize(System::block_number());
	}
}

pub(crate) mod utils {
	use super::*;
	use frame_support::traits::OriginTrait;

	pub fn prepare_hangar_for<T: Config>(account: &AccountIdFor<T>) {
		let origin = OriginFor::<T>::signed(account.clone());

		// Initialize default ships
		Pallet::<T>::add_ship(
			origin.clone(),
			Ship {
				command_power: 1,
				hit_points: 120,
				attack_base: 80,
				attack_variable: 20,
				defence: 20,
				speed: 4,
				range: 4,
			},
		)
		.expect("Should add ship");
		Pallet::<T>::add_ship(
			origin.clone(),
			Ship {
				command_power: 3,
				hit_points: 150,
				attack_base: 65,
				attack_variable: 20,
				defence: 30,
				speed: 3,
				range: 8,
			},
		)
		.expect("Should add ship");
		Pallet::<T>::add_ship(
			origin.clone(),
			Ship {
				command_power: 4,
				hit_points: 220,
				attack_base: 65,
				attack_variable: 20,
				defence: 35,
				speed: 2,
				range: 15,
			},
		)
		.expect("Should add ship");
		Pallet::<T>::add_ship(
			origin,
			Ship {
				command_power: 10,
				hit_points: 450,
				attack_base: 80,
				attack_variable: 20,
				defence: 40,
				speed: 1,
				range: 30,
			},
		)
		.expect("Should add ship");
	}

	pub fn add_commander_to<T: Config>(account: &AccountIdFor<T>, commander_id: CommanderId) {
		Pallet::<T>::try_add_commander_to(account, commander_id).expect("Should add commander")
	}
}
