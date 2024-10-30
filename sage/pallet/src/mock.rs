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

use crate::{self as pallet_sage};
use frame_support::{
	parameter_types,
	traits::{ConstU16, ConstU64},
};
use sp_runtime::{
	testing::{TestSignature, H256},
	traits::{BlakeTwo256, IdentifyAccount, IdentityLookup, Verify},
	BuildStorage,
};

pub type MockSignature = TestSignature;
pub type MockAccountPublic = <MockSignature as Verify>::Signer;
pub type MockAccountId = <MockAccountPublic as IdentifyAccount>::AccountId;
pub type MockBlock = frame_system::mocking::MockBlock<Test>;
pub type MockBalance = u64;
pub type MockCollectionId = u32;

pub const ALICE: MockAccountId = 1;
pub const BOB: MockAccountId = 2;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub struct Test {
		System: frame_system = 0,
		Balances: pallet_balances = 1,
		// pub type SageExampleTransitionInstance = pallet_sage::<Instance1>;
		SageExampleTransition: pallet_sage::<Instance1> = 2,
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
		todo!()
	}
}

pub type SageExampleTransitionInstance = pallet_sage::Instance1;
impl crate::Config<SageExampleTransitionInstance> for Test {
	type SageGameTransition = ExampleTransitionGeneric<MockAccountId, MockBalance, SageMock>;
	type SageApi = SageMock;
	type Currency = Balances;
	type RuntimeEvent = RuntimeEvent;
	type WeightInfo = ();
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
		let config = RuntimeGenesisConfig {
			system: Default::default(),
			balances: BalancesConfig { balances: self.balances },
		};

		let mut ext: sp_io::TestExternalities = config.build_storage().unwrap().into();
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}
