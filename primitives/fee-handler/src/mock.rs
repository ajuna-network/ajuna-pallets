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
	fee_handler::{DistributeFee, GameFeeHandler, Payment},
	withdraw_credit::{EnsureWhitelistedAsset, WithdrawCredit, WithdrawWhitelistedCredit},
};
use frame_support::{
	derive_impl,
	traits::{
		fungibles::{Balanced, Credit},
		tokens::{Fortitude, Precision, Preservation},
		AsEnsureOriginWithArg, ConstU32,
	},
	BoundedVec,
};
use sp_runtime::{
	testing::TestSignature,
	traits::{IdentifyAccount, Verify},
	BuildStorage, DispatchError, TokenError,
};

pub type Signature = TestSignature;
pub type AccountSignature = <Signature as Verify>::Signer;
pub type AccountId = <AccountSignature as IdentifyAccount>::AccountId;
pub type Block = frame_system::mocking::MockBlock<Test>;
pub type Balance = u64;
pub type AssetId = u32;

pub const ALICE: AccountId = 1;
pub const BOB: AccountId = 2;
pub const CHARLIE: AccountId = 3;
pub const DAVE: AccountId = 4;

pub const TREASURER: AccountId = 431;

pub const WHITELISTED_ASSET_ID: AssetId = 888;
pub const NOT_WHITE_LISTED_ASSET_ID: AssetId = 999;

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
	type AccountId = AccountId;
	type AccountData = pallet_balances::AccountData<Balance>;
	type Block = Block;
}

#[derive_impl(pallet_balances::config_preludes::TestDefaultConfig)]
impl pallet_balances::Config for Test {
	type AccountStore = System;
}

#[derive_impl(pallet_assets::config_preludes::TestDefaultConfig)]
impl pallet_assets::Config for Test {
	type Currency = Balances;
	type CreateOrigin = AsEnsureOriginWithArg<frame_system::EnsureSigned<AccountId>>;
	type ForceOrigin = frame_system::EnsureRoot<u64>;
	type Freezer = ();
	type CallbackHandle = ();
}

pub struct TestAffiliatesFeeProvider;

pub enum AffiliateFeeId {
	Paying,
	Free,
}

impl DistributeFee for TestAffiliatesFeeProvider {
	type AccountId = AccountId;
	type Balance = Balance;
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

pub struct TestTournamentFeeProvider;

pub enum TournamentFeeId {
	Paying,
	Free,
}

impl DistributeFee for TestTournamentFeeProvider {
	type AccountId = AccountId;
	type Balance = Balance;
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

pub type TestFeeHandler =
	GameFeeHandler<WithdrawWhitelistedAssets, TestAffiliatesFeeProvider, TestTournamentFeeProvider>;

pub type WithdrawWhitelistedAssets = WithdrawWhitelistedCredit<WhitelistedAssets, WithdrawAsset>;

pub struct WhitelistedAssets;

impl EnsureWhitelistedAsset for WhitelistedAssets {
	type AssetId = AssetId;

	fn ensure_whitelisted(asset_id: &Self::AssetId) -> Result<(), DispatchError> {
		match asset_id {
			&WHITELISTED_ASSET_ID => Ok(()),
			&NOT_WHITE_LISTED_ASSET_ID => Err(DispatchError::Token(TokenError::Unsupported)),
			_ => Err(DispatchError::Token(TokenError::Unsupported)),
		}
	}
}

pub struct WithdrawAsset;

impl WithdrawCredit for WithdrawAsset {
	type AccountId = AccountId;
	type AssetId = AssetId;
	type Assets = Assets;
	type Balance = Balance;

	fn withdraw_credit(
		who: &Self::AccountId,
		asset_id: Self::AssetId,
		credit: Self::Balance,
	) -> Result<Credit<Self::AccountId, Self::Assets>, DispatchError> {
		let asset_fee_credit = Self::Assets::withdraw(
			asset_id.clone(),
			who,
			credit,
			Precision::Exact,
			Preservation::Preserve,
			Fortitude::Polite,
		)?;

		Ok(asset_fee_credit)
	}
}

#[derive(Default)]
pub struct ExtBuilder {
	balances: Vec<(AccountId, Balance)>,
}

impl ExtBuilder {
	pub fn balances(mut self, balances: &[(AccountId, Balance)]) -> Self {
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
					(WHITELISTED_ASSET_ID, ALICE, true, 1),
					(NOT_WHITE_LISTED_ASSET_ID, ALICE, true, 1),
				],
				metadata: vec![
					// id, name, symbol, decimals
					(WHITELISTED_ASSET_ID, "Token 888 Name".into(), "TO888".into(), 10),
					(NOT_WHITE_LISTED_ASSET_ID, "Token 999 Name".into(), "TO999".into(), 10),
				],
				accounts: vec![
					// id, account_id, balance
					(WHITELISTED_ASSET_ID, ALICE, 100),
					(NOT_WHITE_LISTED_ASSET_ID, ALICE, 100),
				],
				next_asset_id: None,
			},
		};

		let mut ext: sp_io::TestExternalities = config.build_storage().unwrap().into();
		ext.execute_with(|| System::set_block_number(1));
		ext
	}
}
