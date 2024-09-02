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

mod mutator;

use crate::{self as pallet_sage, *};
use frame_support::{
	pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo},
	parameter_types,
	traits::{ConstU16, ConstU64, Hooks},
};
pub(crate) use sp_runtime::testing::H256;
use sp_runtime::{
	testing::TestSignature,
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	BuildStorage, DispatchError,
};

pub(crate) use mutator::*;

pub type MockBlock = frame_system::mocking::MockBlock<Test>;
pub type MockSignature = TestSignature;
pub type MockAccountPublic = <MockSignature as Verify>::Signer;
pub type MockAccountId = <MockAccountPublic as IdentifyAccount>::AccountId;
pub type MockBlockNumber = u64;
pub type MockBalance = u64;
pub type MockNonce = u64;

pub const ALICE: MockAccountId = 1;
pub const BOB: MockAccountId = 2;

frame_support::construct_runtime!(
	pub struct Test {
		System: frame_system = 0,
		Balances: pallet_balances = 1,
		Randomness: pallet_insecure_randomness_collective_flip = 2,
		SageAlpha: pallet_sage::<Instance1> = 3,
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
	type Nonce = MockNonce;
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
	pub static MockExistentialDeposit: MockBalance = 321;
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

impl pallet_insecure_randomness_collective_flip::Config for Test {}

type SageInstance1 = pallet_sage::Instance1;
impl pallet_sage::Config<SageInstance1> for Test {
	type RuntimeEvent = RuntimeEvent;
	type Currency = Balances;
	type Randomness = Randomness;
	type StateMutationHandler = mutator::MockStateMutator;
	type WeightInfo = ();
}

pub struct ExtBuilder {
	existential_deposit: MockBalance,
	balances: Vec<(MockAccountId, MockBalance)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self { existential_deposit: MockExistentialDeposit::get(), balances: Default::default() }
	}
}

impl ExtBuilder {
	pub fn existential_deposit(mut self, existential_deposit: MockBalance) -> Self {
		self.existential_deposit = existential_deposit;
		self
	}
	pub fn balances(mut self, balances: &[(MockAccountId, MockBalance)]) -> Self {
		self.balances = balances.to_vec();
		self
	}
	pub fn build(self) -> sp_io::TestExternalities {
		MOCK_EXISTENTIAL_DEPOSIT.with(|v| *v.borrow_mut() = self.existential_deposit);

		let mut t = frame_system::GenesisConfig::<Test>::default().build_storage().unwrap();
		pallet_balances::GenesisConfig::<Test> { balances: self.balances }
			.assimilate_storage(&mut t)
			.unwrap();
		pallet_sage::GenesisConfig::<Test, _>::default()
			.assimilate_storage(&mut t)
			.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}

pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		if System::block_number() > 1 {
			SageAlpha::on_finalize(System::block_number());
			System::on_finalize(System::block_number());
		}
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		SageAlpha::on_initialize(System::block_number());
	}
}
