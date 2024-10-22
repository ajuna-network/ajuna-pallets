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

use crate::{self as pallet_ajuna_affiliates, *};
use ajuna_primitives::account_manager::WhitelistKey;
use frame_support::{
	ensure, parameter_types,
	traits::{ConstU16, ConstU64},
};
use sp_runtime::{
	testing::{TestSignature, H256},
	traits::{BlakeTwo256, ConstU32, IdentifyAccount, IdentityLookup, Verify},
	BoundedVec, BuildStorage, DispatchError,
};
use sp_std::{
	cell::RefCell,
	collections::{btree_map::BTreeMap, btree_set::BTreeSet},
};

pub type MockSignature = TestSignature;
pub type MockAccountPublic = <MockSignature as Verify>::Signer;
pub type MockAccountId = <MockAccountPublic as IdentifyAccount>::AccountId;
pub type MockBlock = frame_system::mocking::MockBlock<Test>;
pub type MockBalance = u64;

pub const ALICE: MockAccountId = 1;
pub const BOB: MockAccountId = 2;
pub const CHARLIE: MockAccountId = 3;
pub const DAVE: MockAccountId = 4;
pub const EDWARD: MockAccountId = 5;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub struct Test {
		System: frame_system = 0,
		Balances: pallet_balances = 1,
		AffiliatesAlpha: pallet_ajuna_affiliates::<Instance1> = 2,
		AffiliatesBeta: pallet_ajuna_affiliates::<Instance2> = 3,
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

parameter_types! {
	pub const AffiliateMaxLevel: u32 = 2;
}

thread_local! {
	pub static WHITELISTED_ACCOUNTS: RefCell<BTreeMap<WhitelistKey ,BTreeSet<MockAccountId>>> = RefCell::new({
		let mut account_map = BTreeMap::new();
		account_map.insert(AffiliateWhitelistKey::get(), BTreeSet::new());
		account_map
	});
	pub static ORGANIZER: RefCell<Option<MockAccountId>> = RefCell::new(None);
}

pub struct MockAccountManager;

pub const ACCOUNT_IS_NOT_ORGANIZER: &str = "ACCOUNT_IS_NOT_ORGANIZER";
pub const NO_ORGANIZER_SET: &str = "NO_ORGANIZER_SET";

impl ajuna_primitives::account_manager::AccountManager for MockAccountManager {
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

	fn is_whitelisted_for(identifier: &WhitelistKey, account: &Self::AccountId) -> bool {
		WHITELISTED_ACCOUNTS.with(|accounts| {
			if let Some(entry) = accounts.borrow_mut().get_mut(identifier) {
				entry.contains(account)
			} else {
				false
			}
		})
	}

	fn try_add_to_whitelist(
		identifier: &WhitelistKey,
		account: &Self::AccountId,
	) -> Result<(), DispatchError> {
		WHITELISTED_ACCOUNTS.with(|accounts| {
			if let Some(entry) = accounts.borrow_mut().get_mut(identifier) {
				entry.insert(*account);
				Ok(())
			} else {
				Err(DispatchError::Other("No account set for identifier"))
			}
		})
	}

	fn remove_from_whitelist(identifier: &WhitelistKey, account: &Self::AccountId) {
		WHITELISTED_ACCOUNTS.with(|accounts| {
			if let Some(entry) = accounts.borrow_mut().get_mut(identifier) {
				entry.remove(account);
			}
		})
	}
}

pub type MockRuleId = u8;
pub type MockRuntimeRule = BoundedVec<u8, ConstU32<2>>;

parameter_types! {
	pub const AffiliateWhitelistKey: WhitelistKey = [1, 2, 1, 2, 3, 3, 4, 5];
}

pub(crate) type AffiliatesInstance1 = pallet_ajuna_affiliates::Instance1;
impl pallet_ajuna_affiliates::Config<AffiliatesInstance1> for Test {
	type RuntimeEvent = RuntimeEvent;
	type WhitelistKey = AffiliateWhitelistKey;
	type AccountManager = MockAccountManager;
	type RuleIdentifier = MockRuleId;
	type RuntimeRule = MockRuntimeRule;
	type AffiliateMaxLevel = AffiliateMaxLevel;
}

pub(crate) type AffiliatesInstance2 = pallet_ajuna_affiliates::Instance2;
impl pallet_ajuna_affiliates::Config<AffiliatesInstance2> for Test {
	type RuntimeEvent = RuntimeEvent;
	type WhitelistKey = AffiliateWhitelistKey;
	type AccountManager = MockAccountManager;
	type RuleIdentifier = MockRuleId;
	type RuntimeRule = MockRuntimeRule;
	type AffiliateMaxLevel = AffiliateMaxLevel;
}

#[derive(Default)]
pub struct ExtBuilder {
	balances: Vec<(MockAccountId, MockBalance)>,
	organizer: Option<MockAccountId>,
	affiliators: Vec<MockAccountId>,
}

impl ExtBuilder {
	pub fn balances(mut self, balances: &[(MockAccountId, MockBalance)]) -> Self {
		self.balances = balances.to_vec();
		self
	}

	pub fn organizer(mut self, organizer: MockAccountId) -> Self {
		self.organizer = Some(organizer);
		self
	}

	pub fn affiliators(mut self, affiliators: &[MockAccountId]) -> Self {
		self.affiliators = affiliators.to_vec();
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let config = RuntimeGenesisConfig {
			system: Default::default(),
			balances: BalancesConfig { balances: self.balances },
		};

		let mut ext: sp_io::TestExternalities = config.build_storage().unwrap().into();
		ext.execute_with(|| System::set_block_number(1));
		ext.execute_with(|| {
			if let Some(account) = self.organizer {
				ORGANIZER.with(|organizer| {
					*organizer.borrow_mut() = Some(account);
				})
			}
			if !self.affiliators.is_empty() {
				for account in self.affiliators.into_iter() {
					AffiliatesAlpha::try_mark_account_as_affiliatable(&account)
						.expect("Should mark as affiliatable");
					AffiliatesBeta::try_mark_account_as_affiliatable(&account)
						.expect("Should mark as affiliatable");
				}
			}
		});
		ext
	}
}
