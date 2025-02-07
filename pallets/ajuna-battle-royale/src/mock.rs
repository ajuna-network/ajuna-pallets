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

use crate::{self as pallet_ajuna_battle_royale};

use core::num::NonZeroU32;
use frame_support::{
	pallet_prelude::Hooks,
	parameter_types,
	traits::{ConstU16, ConstU64},
};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::{
	testing::H256,
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	BuildStorage, MultiSignature,
};

pub type MockSignature = MultiSignature;
pub type MockAccountPublic = <MockSignature as Verify>::Signer;
pub type MockAccountId = <MockAccountPublic as IdentifyAccount>::AccountId;
pub type MockBlock = frame_system::mocking::MockBlock<Test>;
pub type MockBalance = u64;
pub type MockBlockNumber = BlockNumberFor<Test>;

pub const ALICE: MockAccountId = MockAccountId::new([1; 32]);
pub const BOB: MockAccountId = MockAccountId::new([2; 32]);
pub const CHARLIE: MockAccountId = MockAccountId::new([3; 32]);
pub const DAVE: MockAccountId = MockAccountId::new([4; 32]);
pub const EDWARD: MockAccountId = MockAccountId::new([5; 32]);

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub struct Test {
		System: frame_system = 0,
		Balances: pallet_balances = 1,
		BattleRoyale: pallet_ajuna_battle_royale::<Instance1> = 2,
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

parameter_types! {
	// Input lasts 3 blocks
	pub const InputPhaseDuration: NonZeroU32 = NonZeroU32::MIN.saturating_add(2);
	// Reveal lasts 3 blocks
	pub const RevealPhaseDuration: NonZeroU32 = NonZeroU32::MIN.saturating_add(2);
	// Execution lasts 1 block
	pub const ExecutionPhaseDuration: NonZeroU32 = NonZeroU32::MIN;
	// Shrink lasts 1 block
	pub const ShrinkPhaseDuration: NonZeroU32 = NonZeroU32::MIN;
	// Verification lasts 1 block
	pub const VerificationPhaseDuration: NonZeroU32 = NonZeroU32::MIN;
	// Idle lasts 2 blocks
	pub const IdlePhaseDuration: NonZeroU32 = NonZeroU32::MIN.saturating_add(1);
}

type BattleInstance1 = pallet_ajuna_battle_royale::Instance1;
impl pallet_ajuna_battle_royale::Config<BattleInstance1> for Test {
	type RuntimeEvent = RuntimeEvent;
	type InputPhaseDuration = InputPhaseDuration;
	type RevealPhaseDuration = RevealPhaseDuration;
	type ExecutionPhaseDuration = ExecutionPhaseDuration;
	type ShrinkPhaseDuration = ShrinkPhaseDuration;
	type VerificationPhaseDuration = VerificationPhaseDuration;
	type IdlePhaseDuration = IdlePhaseDuration;
}

pub struct ExtBuilder {
	balances: Vec<(MockAccountId, MockBalance)>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			balances: vec![
				(ALICE, 1_000),
				(BOB, 1_000),
				(CHARLIE, 1_000),
				(EDWARD, 1_000),
				(DAVE, 1_000),
			],
		}
	}
}

impl ExtBuilder {
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
		ext
	}
}

pub fn run_to_block(n: u64) {
	while System::block_number() < n {
		if System::block_number() > 1 {
			System::on_finalize(System::block_number());
			BattleRoyale::on_finalize(System::block_number());
		}
		System::set_block_number(System::block_number() + 1);
		System::on_initialize(System::block_number());
		BattleRoyale::on_initialize(System::block_number());
	}
}
