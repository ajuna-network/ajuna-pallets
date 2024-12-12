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
	fee_handler::{AssetGameFeeHandler, DistributeFee, PaymentFee},
	voucher_handler::VoucherHandler,
	withdraw_credit::{EnsureWhitelistedAsset, WithdrawWhitelistedCredit},
	AffiliateFeeDistribution, NativeGameFeeHandler, TournamentFeeDistribution, WithdrawAsset,
	WithdrawCreditOrVoucher, WithdrawKind, WithdrawNative,
};
use frame_support::{
	derive_impl,
	traits::{AsEnsureOriginWithArg, ConstU32},
};
use sp_runtime::{
	testing::TestSignature,
	traits::{IdentifyAccount, Verify},
	BuildStorage, DispatchError, TokenError,
};
use std::{cell::RefCell, collections::HashMap};

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
pub const FERDIE: AccountId = 5;

pub const TOURNAMENT_TREASURY: AccountId = 431;

pub const WHITELISTED_ASSET_ID: AssetId = 888;
pub const WHITELISTED_ASSET_ID_PAYMENT: WithdrawKind<AssetId> =
	WithdrawKind::Payment(WHITELISTED_ASSET_ID);
pub const NOT_WHITE_LISTED_ASSET_ID: AssetId = 999;
pub const NOT_WHITELISTED_ASSET_ID_PAYMENT: WithdrawKind<AssetId> =
	WithdrawKind::Payment(NOT_WHITE_LISTED_ASSET_ID);
pub const VOUCHER_ASSET_PAYMENT: WithdrawKind<AssetId> = WithdrawKind::Voucher;
pub const NATIVE_ASSET_PAYMENT: WithdrawKind<()> = WithdrawKind::Payment(());
pub const VOUCHER_NATIVE_PAYMENT: WithdrawKind<()> = WithdrawKind::Voucher;

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
	type ForceOrigin = frame_system::EnsureRoot<AccountId>;
	type Freezer = ();
	type CallbackHandle = ();
	#[cfg(feature = "runtime-benchmarks")]
	type BenchmarkHelper = ();
}

pub struct TestAffiliatesFeeProvider;

pub type TestAffiliatesMaxDistribution = ConstU32<3>;

pub enum AffiliateFeeId {
	Paying,
	Free,
}

impl DistributeFee for TestAffiliatesFeeProvider {
	type AccountId = AccountId;
	type Balance = Balance;
	type FeeIdentifier = AffiliateFeeId;
	type FeeDistribution =
		AffiliateFeeDistribution<Self::AccountId, Self::Balance, TestAffiliatesMaxDistribution>;

	fn distribute_fee(
		base_fee: Self::Balance,
		_account: &Self::AccountId,
		identifier: &Self::FeeIdentifier,
	) -> Option<Self::FeeDistribution> {
		match identifier {
			AffiliateFeeId::Paying => Some(
				vec![
					PaymentFee::new(BOB, base_fee * 4 / 20),
					PaymentFee::new(CHARLIE, base_fee * 3 / 20),
					PaymentFee::new(DAVE, base_fee * 2 / 20),
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
	type FeeDistribution = TournamentFeeDistribution<Self::AccountId, Self::Balance>;

	fn distribute_fee(
		base_fee: Self::Balance,
		_account: &Self::AccountId,
		identifier: &Self::FeeIdentifier,
	) -> Option<Self::FeeDistribution> {
		match identifier {
			TournamentFeeId::Paying =>
				Some(PaymentFee::new(TOURNAMENT_TREASURY, base_fee * 2 / 10)),
			TournamentFeeId::Free => None,
		}
	}
}

thread_local! {
	pub static VOUCHERS: RefCell<HashMap<AccountId, Balance>> = RefCell::new(HashMap::new());
}

pub struct MockVoucherHandler;

impl VoucherHandler for MockVoucherHandler {
	type AccountId = AccountId;
	type Balance = Balance;

	fn consume_vouchers_from(
		account: &Self::AccountId,
		amount: Self::Balance,
	) -> Result<(), DispatchError> {
		VOUCHERS.with_borrow_mut(|voucher_store| match voucher_store.get(account) {
			Some(voucher_amt) if *voucher_amt >= amount => {
				voucher_store.insert(*account, voucher_amt.saturating_sub(amount));
				Ok(())
			},
			_ => Err(DispatchError::Token(TokenError::FundsUnavailable)),
		})
	}
}

pub type TestAssetFeeHandler = AssetGameFeeHandler<
	AccountId,
	Assets,
	WithdrawCreditOrVoucher<WithdrawWhitelistedAssets, MockVoucherHandler>,
	TestAffiliatesFeeProvider,
	TestAffiliatesMaxDistribution,
	TestTournamentFeeProvider,
>;

pub type TestNativeFeeHandler = NativeGameFeeHandler<
	AccountId,
	Balances,
	WithdrawCreditOrVoucher<WithdrawNative<AccountId, Balances>, MockVoucherHandler>,
	TestAffiliatesFeeProvider,
	TestAffiliatesMaxDistribution,
	TestTournamentFeeProvider,
>;

pub type WithdrawWhitelistedAssets =
	WithdrawWhitelistedCredit<WhitelistedAssets, WithdrawAsset<Test>>;

pub struct WhitelistedAssets;

impl EnsureWhitelistedAsset for WhitelistedAssets {
	type AssetId = AssetId;

	fn ensure_whitelisted(asset_id: &Self::AssetId) -> Result<(), DispatchError> {
		match *asset_id {
			WHITELISTED_ASSET_ID => Ok(()),
			NOT_WHITE_LISTED_ASSET_ID => Err(DispatchError::Token(TokenError::Unsupported)),
			_ => Err(DispatchError::Token(TokenError::Unsupported)),
		}
	}
}

#[derive(Default)]
pub struct ExtBuilder {
	vouchers: Vec<(AccountId, Balance)>,
}

impl ExtBuilder {
	pub fn vouchers(mut self, vouchers: &[(AccountId, Balance)]) -> Self {
		self.vouchers = vouchers.to_vec();
		self
	}
}

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
		let config = RuntimeGenesisConfig {
			system: Default::default(),
			balances: pallet_balances::GenesisConfig::<Test, ()> { balances: vec![(ALICE, 100)] },
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
		ext.execute_with(|| {
			if !self.vouchers.is_empty() {
				for (account, voucher_amt) in self.vouchers.iter().copied() {
					VOUCHERS.with_borrow_mut(|voucher_store| {
						voucher_store.insert(account, voucher_amt);
					});
				}
			}
		});
		ext
	}
}
