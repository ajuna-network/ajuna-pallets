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

#![cfg(test)]

use frame_support::{
    parameter_types,
    traits::{AsEnsureOriginWithArg, ConstU16, ConstU64},
    PalletId,
};
use frame_system::{EnsureRoot, EnsureSigned};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::{
    testing::H256,
    traits::{BlakeTwo256, Get, IdentifyAccount, IdentityLookup, Verify},
    BuildStorage, MultiSignature,
};
use sp_runtime::traits::BlockNumberProvider;

pub type MockSignature = MultiSignature;
pub type MockAccountPublic = <MockSignature as Verify>::Signer;
pub type MockAccountId = <MockAccountPublic as IdentifyAccount>::AccountId;
pub type MockBlock = frame_system::mocking::MockBlock<Runtime>;
pub type MockBlockNumber = u64;
pub type MockBalance = u64;
pub type MockCollectionId = u32;

impl crate::Config for Runtime {}

frame_support::construct_runtime!(
	pub struct Runtime {
		System: frame_system = 0,
		Balances: pallet_balances = 1,
        Vesting: orml_vesting = 17,
	}
);

impl frame_system::Config for Runtime {
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
}

parameter_types! {
	pub static MockExistentialDeposit: MockBalance = 321;
}

impl pallet_balances::Config for Runtime {
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

parameter_types! {
	pub const MinVestedTransfer: Balance = MockExistentialDeposit;
}


parameter_types! {
	pub static MockBlockNumberProvider: u64 = 0;
}

impl BlockNumberProvider for MockBlockNumberProvider {
    type BlockNumber = u64;

    fn current_block_number() -> BlockNumberFor<Runtime> {
        Self::get()
    }
}


impl orml_vesting::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type Currency = Balances;
    type MinVestedTransfer = MinVestedTransfer;
    type VestedTransferOrigin = EnsureSigned<AccountId>;
    type WeightInfo = ();
    type MaxVestingSchedules = frame_support::traits::ConstU32<100>;
    type BlockNumberProvider = MockBlockNumberProvider;
}

pub fn new_test_ext() -> sp_io::TestExternalities {
    let t = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();
    sp_io::TestExternalities::new(t)
}
