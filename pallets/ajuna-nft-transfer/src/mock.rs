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

use crate::{self as pallet_ajuna_nft_transfer};
use frame_support::{
	parameter_types,
	traits::{AsEnsureOriginWithArg, ConstU16, ConstU64},
	PalletId,
};
use frame_system::{EnsureRoot, EnsureSigned};
use pallet_nfts::{PalletFeature, PalletFeatures};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{
	testing::{TestSignature, H256},
	traits::{BlakeTwo256, Get, IdentifyAccount, IdentityLookup, Verify},
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
		System: frame_system,
		Nft: pallet_nfts,
		Balances: pallet_balances,
		NftTransfer: pallet_ajuna_nft_transfer,
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

#[derive(Debug, PartialEq, Eq, Clone, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct ParameterGet<const N: u32>;

impl<const N: u32> Get<u32> for ParameterGet<N> {
	fn get() -> u32 {
		N
	}
}

pub type KeyLimit = ParameterGet<32>;
pub type ValueLimit = ParameterGet<64>;

parameter_types! {
	pub const CollectionDeposit: MockBalance = 999;
	pub const ItemDeposit: MockBalance = 333;
	pub const StringLimit: u32 = 128;
	pub const MetadataDepositBase: MockBalance = 1;
	pub const AttributeDepositBase: MockBalance = 1;
	pub const DepositPerByte: MockBalance = 1;
	pub const ApprovalsLimit: u32 = 1;
	pub const ItemAttributesApprovalsLimit: u32 = 10;
	pub const MaxTips: u32 = 1;
	pub const MaxDeadlineDuration: u32 = 1;
	pub const MaxAttributesPerCall: u32 = 10;
	pub MockFeatures: PalletFeatures = PalletFeatures::from_disabled(
		PalletFeature::Attributes.into()
	);
}

#[cfg(feature = "runtime-benchmarks")]
pub struct Helper;
#[cfg(feature = "runtime-benchmarks")]
impl<CollectionId: From<u16>, ItemId: From<[u8; 32]>>
	pallet_nfts::BenchmarkHelper<CollectionId, ItemId> for Helper
{
	fn collection(i: u16) -> CollectionId {
		i.into()
	}
	fn item(i: u16) -> ItemId {
		let mut id = [0_u8; 32];
		let bytes = i.to_be_bytes();
		id[0] = bytes[0];
		id[1] = bytes[1];
		id.into()
	}
}

impl pallet_nfts::Config for Test {
	type RuntimeEvent = RuntimeEvent;
	type CollectionId = MockCollectionId;
	type ItemId = H256;
	type Currency = Balances;
	type ForceOrigin = EnsureRoot<MockAccountId>;
	type CreateOrigin = AsEnsureOriginWithArg<EnsureSigned<MockAccountId>>;
	type Locker = NftTransfer;
	type CollectionDeposit = CollectionDeposit;
	type ItemDeposit = ItemDeposit;
	type MetadataDepositBase = MetadataDepositBase;
	type AttributeDepositBase = AttributeDepositBase;
	type DepositPerByte = DepositPerByte;
	type StringLimit = StringLimit;
	type KeyLimit = KeyLimit;
	type ValueLimit = ValueLimit;
	type ApprovalsLimit = ApprovalsLimit;
	type ItemAttributesApprovalsLimit = ItemAttributesApprovalsLimit;
	type MaxTips = MaxTips;
	type MaxDeadlineDuration = MaxDeadlineDuration;
	type MaxAttributesPerCall = MaxAttributesPerCall;
	type Features = MockFeatures;
	type OffchainSignature = MockSignature;
	type OffchainPublic = MockAccountPublic;
	pallet_nfts::runtime_benchmarks_enabled! {
		type Helper = Helper;
	}
	type WeightInfo = ();
}

parameter_types! {
	pub const NftTransferPalletId: PalletId = PalletId(*b"aj/nfttr");
}

impl pallet_ajuna_nft_transfer::Config for Test {
	type PalletId = NftTransferPalletId;
	type RuntimeEvent = RuntimeEvent;
	type CollectionId = MockCollectionId;
	type ItemId = H256;
	type ItemConfig = pallet_nfts::ItemConfig;
	type KeyLimit = KeyLimit;
	type ValueLimit = ValueLimit;
	type NftHelper = Nft;
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
