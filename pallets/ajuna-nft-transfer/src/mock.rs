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

use crate::{
	self as pallet_ajuna_nft_transfer,
	traits::{NFTAttribute, NftConvertible},
};
use ajuna_primitives::asset_manager::{AssetManager, Lock};
use frame_support::{
	ensure, parameter_types,
	traits::{
		AsEnsureOriginWithArg, ConstU16, ConstU64, Currency, ExistenceRequirement, LockIdentifier,
	},
	BoundedVec, PalletId,
};
use frame_system::{EnsureRoot, EnsureSigned};
use pallet_nfts::{PalletFeature, PalletFeatures};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::{
	bounded_vec,
	testing::{TestSignature, H256},
	traits::{BlakeTwo256, Get, IdentifyAccount, IdentityLookup, Verify},
	BuildStorage, DispatchError,
};
use std::{cell::RefCell, collections::BTreeMap};

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

pub type ItemId = H256;

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
	type ItemId = ItemId;
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
	type Item = MockItem;
	type ItemId = ItemId;
	type ItemConfig = pallet_nfts::ItemConfig;
	type AssetManager = MockAssetManager;
	type KeyLimit = KeyLimit;
	type ValueLimit = ValueLimit;
	type NftHelper = Nft;
	type WeightInfo = ();
}

#[derive(Encode, Decode, Clone, Eq, PartialEq, Debug)]
pub struct MockItem {
	pub field_1: Vec<u8>,
	pub field_2: u32,
	pub field_3: bool,
}

impl MockItem {
	pub fn new_with_field2(number: u32) -> Self {
		Self { field_2: number, ..Self::default() }
	}
}

impl Default for MockItem {
	fn default() -> Self {
		Self { field_1: vec![1, 2, 3], field_2: 333, field_3: true }
	}
}

impl NftConvertible<KeyLimit, ValueLimit> for MockItem {
	const ITEM_CODE: &'static [u8] = &[11];
	const IPFS_URL_CODE: &'static [u8] = &[21];

	fn get_attribute_codes() -> Vec<NFTAttribute<KeyLimit>> {
		vec![bounded_vec![111], bounded_vec![222], bounded_vec![240]]
	}

	fn get_encoded_attributes(&self) -> Vec<(NFTAttribute<KeyLimit>, NFTAttribute<ValueLimit>)> {
		vec![
			(bounded_vec![111], BoundedVec::try_from(self.field_1.clone()).unwrap()),
			(bounded_vec![222], BoundedVec::try_from(self.field_2.to_le_bytes().to_vec()).unwrap()),
			(bounded_vec![240], BoundedVec::try_from(vec![self.field_3 as u8]).unwrap()),
		]
	}
}

thread_local! {
	pub static OWNERS: RefCell<BTreeMap<MockAccountId, ItemId>> = RefCell::new(BTreeMap::new());
	pub static ASSETS: RefCell<BTreeMap<ItemId, MockItem>> = RefCell::new(BTreeMap::new());
	pub static LOCKED_ASSETS: RefCell<BTreeMap<ItemId, Lock<MockAccountId>>> = RefCell::new(BTreeMap::new());
	pub static ORGANIZER: RefCell<MockAccountId> = RefCell::new(ALICE);
	pub static NFT_TRANSFER_OPEN: RefCell<bool> = RefCell::new(true);
	pub static PREPARE_FEE: RefCell<MockBalance> = RefCell::new(999);
}

pub struct MockAssetManager;

impl MockAssetManager {
	pub fn create_assets(owner: MockAccountId, count: u32) -> Vec<ItemId> {
		let mut ids = Vec::with_capacity(count as usize);
		for i in 0..count {
			let id = ItemId::repeat_byte(i as u8);
			ids.push(id);
			Self::add_asset(owner, id, MockItem::new_with_field2(i))
		}

		ids
	}

	pub fn add_asset(owner: MockAccountId, asset_id: ItemId, asset: MockItem) {
		OWNERS.with(|owners| owners.borrow_mut().insert(owner, asset_id.clone()));
		ASSETS.with(|assets| assets.borrow_mut().insert(asset_id, asset));
	}

	pub fn set_nft_transfer_open(open: bool) {
		NFT_TRANSFER_OPEN.with(|is_open| *is_open.borrow_mut() = open)
	}

	pub fn set_prepare_fee(fee: MockBalance) {
		PREPARE_FEE.with(|current| *current.borrow_mut() = fee)
	}
}

pub const NOT_OWNER_ERR: &str = "NOT_OWNER";
pub const ALREADY_LOCKED_ERR: &str = "ALREADY_LOCKED";
pub const NOT_LOCKED_ERR: &str = "NOT_LOCKED";
pub const LOCKED_BY_OTHER_ERR: &str = "LOCKED_BY_OTHER";

impl AssetManager for MockAssetManager {
	type AccountId = MockAccountId;
	type AssetId = ItemId;
	type Asset = MockItem;

	fn ensure_organizer(account: &Self::AccountId) -> Result<(), DispatchError> {
		ORGANIZER.with(|locked| {
			if &*locked.borrow() == account {
				Ok(())
			} else {
				DispatchError::BadOrigin.into()
			}
		})
	}

	fn ensure_ownership(
		owner: &Self::AccountId,
		_asset_id: &Self::AssetId,
	) -> Result<Self::Asset, DispatchError> {
		let id = OWNERS
			.with(|owners| owners.borrow().get(&owner).cloned())
			.ok_or_else(|| DispatchError::Other(NOT_OWNER_ERR))?;
		ASSETS
			.with(|assets| assets.borrow().get(&id).cloned())
			.ok_or_else(|| DispatchError::Other(NOT_OWNER_ERR))
	}

	fn lock_asset(
		lock_id: LockIdentifier,
		owner: Self::AccountId,
		asset_id: Self::AssetId,
	) -> Result<Self::Asset, DispatchError> {
		let asset = Self::ensure_ownership(&owner, &asset_id)?;

		LOCKED_ASSETS.with(|locked| {
			let mut borrowed = locked.borrow_mut();

			if borrowed.contains_key(&asset_id) {
				return DispatchError::Other(ALREADY_LOCKED_ERR).into();
			} else {
				borrowed.insert(asset_id, Lock::new(lock_id, owner));
				Ok(())
			}
		})?;

		Ok(asset)
	}

	fn unlock_asset(
		lock_id: LockIdentifier,
		owner: Self::AccountId,
		asset_id: Self::AssetId,
	) -> Result<Self::Asset, DispatchError> {
		let asset = Self::ensure_ownership(&owner, &asset_id)?;

		let lock = LOCKED_ASSETS.with(|locked| {
			locked
				.borrow_mut()
				.remove(&asset_id)
				.ok_or_else(|| DispatchError::Other(NOT_LOCKED_ERR))
		})?;

		ensure!(lock.id == lock_id, DispatchError::Other(LOCKED_BY_OTHER_ERR));
		ensure!(lock.locker == owner, DispatchError::Other(NOT_OWNER_ERR));

		Ok(asset)
	}

	fn is_locked(asset: &Self::AssetId) -> Option<Lock<Self::AccountId>> {
		LOCKED_ASSETS.with(|locked| locked.borrow().get(asset).cloned())
	}

	fn nft_transfer_open() -> bool {
		NFT_TRANSFER_OPEN.with(|locked| locked.borrow().clone())
	}

	fn handle_asset_fees(
		_asset: &Self::Asset,
		from: &Self::AccountId,
		fees_recipient: &Self::AccountId,
	) -> Result<(), DispatchError> {
		PREPARE_FEE.with(|fee| {
			let f = *fee.borrow();
			<Balances as Currency<MockAccountId>>::transfer(
				from,
				fees_recipient,
				f,
				ExistenceRequirement::AllowDeath,
			)
		})?;

		Ok(())
	}
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
