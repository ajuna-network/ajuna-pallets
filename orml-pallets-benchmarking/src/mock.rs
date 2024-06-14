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

use frame_support::{
	parameter_types,
	traits::{AsEnsureOriginWithArg, ConstU16, ConstU64},
};
use frame_system::{pallet_prelude::BlockNumberFor, EnsureRoot, EnsureSigned};
use sp_runtime::{
	testing::H256,
	traits::{BlakeTwo256, BlockNumberProvider, IdentifyAccount, IdentityLookup, Verify},
	BuildStorage, MultiSignature,
};

pub type MockSignature = MultiSignature;
pub type MockAccountPublic = <MockSignature as Verify>::Signer;
pub type MockAccountId = <MockAccountPublic as IdentifyAccount>::AccountId;
pub type MockBlock = frame_system::mocking::MockBlock<Runtime>;
pub type MockBalance = u64;
pub type MockAssetId = u32;

frame_support::construct_runtime!(
	pub struct Runtime {
		System: frame_system = 0,
		Balances: pallet_balances = 1,
		AssetsPallet: pallet_assets = 2,
		ParachainInfo: staging_parachain_info = 3,
		XcmPallet: pallet_xcm = 10,
		Vesting: orml_vesting = 17,
		OrmlXcm: orml_xcm = 18,
		Xtokens: orml_xtokens = 19,
	}
);

impl crate::vesting::Config for Runtime {}

impl frame_system::Config for Runtime {
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
	pub static MockExistentialDeposit: MockBalance = 321;
}

impl pallet_balances::Config for Runtime {
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

impl staging_parachain_info::Config for Runtime {}

parameter_types! {
	pub const AssetDeposit: MockBalance = MockBalance::MAX;
	pub const MetadataDepositBase: MockBalance = 0;
	pub const AttributeDepositBase: MockBalance = 0;
	pub const AssetAccountDeposit: MockBalance = 1_000;
	pub const ApprovalDeposit: MockBalance = 1_000;
	pub const MetadataDepositPerByte: MockBalance = 0;
}

#[cfg(feature = "runtime-benchmarks")]
pub struct AssetHelper;
#[cfg(feature = "runtime-benchmarks")]
impl<AssetId: From<u32>> pallet_assets::BenchmarkHelper<AssetId> for AssetHelper {
	fn create_asset_id_parameter(id: u32) -> AssetId {
		id.into()
	}
}

impl pallet_assets::Config for Runtime {
	type RuntimeEvent = RuntimeEvent;
	type Balance = MockBalance;
	type RemoveItemsLimit = frame_support::traits::ConstU32<1000>;
	type AssetId = MockAssetId;
	type AssetIdParameter = parity_scale_codec::Compact<MockAssetId>;
	type Currency = Balances;
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<MockAccountId>>;
	type ForceOrigin = EnsureRoot<MockAccountId>;
	type AssetDeposit = AssetDeposit;
	type AssetAccountDeposit = AssetAccountDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type MetadataDepositPerByte = MetadataDepositPerByte;
	type ApprovalDeposit = ApprovalDeposit;
	type StringLimit = frame_support::traits::ConstU32<20>;
	type Freezer = ();
	type Extra = ();
	type CallbackHandle = ();
	type WeightInfo = ();
	pallet_assets::runtime_benchmarks_enabled! {
		type BenchmarkHelper = AssetHelper;
	}
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
	type MinVestedTransfer = MockExistentialDeposit;
	type VestedTransferOrigin = EnsureSigned<MockAccountId>;
	type WeightInfo = ();
	type MaxVestingSchedules = frame_support::traits::ConstU32<100>;
	type BlockNumberProvider = MockBlockNumberProvider;
}

impl crate::benchmarks::xcm::Config for Runtime {}

pub fn new_test_ext() -> sp_io::TestExternalities {
	let t = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();
	sp_io::TestExternalities::new(t)
}
