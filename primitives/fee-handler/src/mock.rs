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

use crate::fee_handler::{DistributeFee, Payment};
use frame_support::{
	derive_impl,
	traits::{AsEnsureOriginWithArg, ConstU32},
	BoundedVec,
};
use sp_runtime::{
	testing::TestSignature,
	traits::{IdentifyAccount, Verify},
	BuildStorage,
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

pub const TREASURER: MockAccountId = 431;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub struct Test {
		System: frame_system = 0,
		Balances: pallet_balances = 1,
		Assets: pallet_assets = 2,
	}
);

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Test {
	type AccountId = MockAccountId;
	type AccountData = pallet_balances::AccountData<MockBalance>;
	type Block = MockBlock;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
	type AccountStore = System;
}

#[derive_impl(pallet_assets::config_preludes::TestDefaultConfig)]
impl pallet_assets::Config for Test {
	type Currency = Balances;
	type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<MockAccountId>>;
	type ForceOrigin = frame_system::EnsureRoot<u64>;
	type Freezer = ();
	type CallbackHandle = ();
}

pub struct MockAffiliatesFeeProvider;

pub enum AffiliateFeeId {
	Paying,
	Free,
}

impl DistributeFee for MockAffiliatesFeeProvider {
	type AccountId = MockAccountId;
	type Balance = MockBalance;
	type FeeIdentifier = AffiliateFeeId;
	type MaxDistributions = ConstU32<3>;

	fn distribute_fee(
		base_fee: Self::Balance,
		_account: &Self::AccountId,
		identifier: &Self::FeeIdentifier,
	) -> Option<BoundedVec<Payment<Self::AccountId, Self::Balance>, Self::MaxDistributions>> {
		match identifier {
			AffiliateFeeId::Paying => Some(
				vec![
					Payment::new(ALICE, base_fee * 5 / 10),
					Payment::new(BOB, base_fee * 3 / 10),
					Payment::new(CHARLIE, base_fee * 2 / 10),
				]
				.try_into()
				.expect("max distributions = 3; qed"),
			),
			AffiliateFeeId::Free => None,
		}
	}
}

pub struct MockTournamentFeeProvider;

pub enum TournamentFeeId {
	Paying,
	Free,
}

impl DistributeFee for MockTournamentFeeProvider {
	type AccountId = MockAccountId;
	type Balance = MockBalance;
	type FeeIdentifier = TournamentFeeId;
	type MaxDistributions = ConstU32<1>;

	fn distribute_fee(
		base_fee: Self::Balance,
		_account: &Self::AccountId,
		identifier: &Self::FeeIdentifier,
	) -> Option<BoundedVec<Payment<Self::AccountId, Self::Balance>, Self::MaxDistributions>> {
		match identifier {
			TournamentFeeId::Paying => Some(
				vec![Payment::new(TREASURER, base_fee * 2 / 10)]
					.try_into()
					.expect("max distribution = 1; qed"),
			),
			TournamentFeeId::Free => None,
		}
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
			assets: pallet_assets::GenesisConfig {
				assets: vec![
					// id, owner, is_sufficient, min_balance
					(888, ALICE, true, 1),
					(999, ALICE, true, 1),
				],
				metadata: vec![
					// id, name, symbol, decimals
					(888, "Token 888 Name".into(), "TO888".into(), 10),
					(999, "Token 999 Name".into(), "TO999".into(), 10),
				],
				accounts: vec![
					// id, account_id, balance
					(888, ALICE, 100),
					(999, ALICE, 100),
				],
				next_asset_id: None,
			},
		};

		let mut ext: sp_io::TestExternalities = config.build_storage().unwrap().into();
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}
