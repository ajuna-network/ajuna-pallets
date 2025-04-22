// Ajuna Node
// Copyright (C) 2022 BlogaTech AG
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use crate::{IdentifyVoucherOrAssetId, WithdrawCredit};
use frame_support::{
	pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo},
	traits::{
		fungibles,
		tokens::{Fortitude, Preservation},
	},
};
use sp_runtime::{DispatchError, TokenError};
use sp_std::marker::PhantomData;

/// Abstraction around transferring funds without disclosing the
/// internals of how the funds are managed.
///
/// The main benefit of this trait is that the users doesn't have
/// to care if the fungible or the fungibles is used behind the
/// scenes.
pub trait TransferFungible {
	type AccountId;

	type AssetId;

	type Balance;

	fn transfer(
		asset_id: Self::AssetId,
		from: &Self::AccountId,
		to: &Self::AccountId,
		amount: Self::Balance,
		preservation: Preservation,
	) -> Result<TransferResult<Self::AssetId, Self::Balance>, DispatchError>;

	fn transfer_all(
		asset_id: Self::AssetId,
		from: &Self::AccountId,
		to: &Self::AccountId,
	) -> Result<TransferResult<Self::AssetId, Self::Balance>, DispatchError>;
}

pub struct TransferFungibleAssets<W, I>(PhantomData<(W, I)>);

/// Result of a successful transfer.
///
/// It is important to check this result because we do have implementations
/// that withdraw credit in one asset and allocate it as another asset.
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Debug, Default, Copy, Clone, PartialEq)]
pub struct TransferResult<AssetId, Amount> {
	pub input_asset_id: AssetId,
	pub input_amount: Amount,
	pub output_asset_id: AssetId,
	pub output_amount: Amount,
}

impl<AccountId, Assets, W, I> TransferFungible for TransferFungibleAssets<W, I>
where
	Assets: fungibles::Balanced<AccountId, Balance = W::Balance>
		+ fungibles::Inspect<AccountId, Balance = W::Balance, AssetId = I::AssetId>,
	W: WithdrawCredit<
		AccountId = AccountId,
		Assets = Assets,
		Credit = fungibles::Credit<AccountId, Assets>,
		AssetId = I,
	>,
	I: IdentifyVoucherOrAssetId + Clone,
	W::AssetId: From<I::AssetId>,
{
	type AccountId = AccountId;
	type AssetId = W::AssetId;
	type Balance = W::Balance;

	fn transfer(
		asset_id: Self::AssetId,
		from: &Self::AccountId,
		to: &Self::AccountId,
		amount: Self::Balance,
		preservation: Preservation,
	) -> Result<TransferResult<Self::AssetId, Self::Balance>, DispatchError> {
		if asset_id.is_voucher() {
			log::debug!("Transferring vouchers is unsupported on this level. The pallet should handle that.");
			return Err(DispatchError::Token(TokenError::Unsupported));
		}

		if let Some(credit) = W::withdraw_credit(from, asset_id.clone(), amount, preservation)? {
			let result = TransferResult {
				input_asset_id: asset_id,
				input_amount: amount,
				output_asset_id: credit.asset().into(),
				output_amount: credit.peek(),
			};

			if let Err(_credit) = W::Assets::resolve(to, credit) {
				// We decide to continue here, because the error has nothing to do with the
				// account sending the transaction. It would be a bad user experience if
				// the transaction fails because we can't allocate the fees to the recipient.
				log::error!(
				"Could not deposit to beneficiary, it probably doesn't exist, burning the credit..."
			);
			}
			Ok(result)
		} else {
			// We should never get here as we checked above if the asset was a voucher, and all
			// other known implementations should return `Some`.
			log::error!("No credit withdrawn, this is unexpected.");
			Err(DispatchError::Token(TokenError::Unsupported))
		}
	}

	fn transfer_all(
		asset_id: Self::AssetId,
		from: &Self::AccountId,
		to: &Self::AccountId,
	) -> Result<TransferResult<Self::AssetId, Self::Balance>, DispatchError> {
		if let Some(id) = asset_id.as_asset_id() {
			let balance = W::Assets::reducible_balance(
				id.clone(),
				from,
				Preservation::Expendable,
				Fortitude::Polite,
			);

			Self::transfer(asset_id, from, to, balance, Preservation::Expendable)
		} else {
			// The asset id is a voucher or anything else not-relating to fungible assets.
			log::debug!("Transferring vouchers is unsupported on this level. The pallet should handle that.");
			Err(DispatchError::Token(TokenError::Unsupported))
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		mock::{
			AssetId, Assets, ExtBuilder, MockVoucherHandler, WithdrawWhitelistedAssets, ALICE,
			FERDIE, NOT_WHITELISTED_ASSET_ID_PAYMENT, NOT_WHITE_LISTED_ASSET_ID, VOUCHERS,
			VOUCHER_ASSET_PAYMENT, WHITELISTED_ASSET_ID, WHITELISTED_ASSET_ID_PAYMENT,
		},
		WithdrawCreditOrVoucher, WithdrawKind,
	};
	use frame_support::assert_noop;
	use sp_runtime::{ModuleError, TokenError};

	type TestAssetTransfer = TransferFungibleAssets<
		WithdrawCreditOrVoucher<WithdrawWhitelistedAssets, MockVoucherHandler>,
		WithdrawKind<AssetId>,
	>;

	mod transfer {
		use super::*;

		#[test]
		fn transfer_works() {
			ExtBuilder::default().build().execute_with(|| {
				let fee = 20;
				let alice_balance_before = Assets::balance(WHITELISTED_ASSET_ID, ALICE);
				let fee_beneficiary = FERDIE;

				TestAssetTransfer::transfer(
					WHITELISTED_ASSET_ID_PAYMENT,
					&ALICE,
					&fee_beneficiary,
					fee,
					Preservation::Preserve,
				)
				.unwrap();

				assert_eq!(
					Assets::balance(WHITELISTED_ASSET_ID, ALICE),
					alice_balance_before - fee
				);
				assert_eq!(Assets::balance(WHITELISTED_ASSET_ID, fee_beneficiary), fee)
			});
		}

		#[test]
		fn transfer_fails_if_missing_funds() {
			ExtBuilder::default().build().execute_with(|| {
				let fee = 101;
				let alice_balance_before = Assets::balance(WHITELISTED_ASSET_ID, ALICE);
				let fee_beneficiary = FERDIE;

				assert_noop!(
					TestAssetTransfer::transfer(
						WHITELISTED_ASSET_ID_PAYMENT,
						&ALICE,
						&fee_beneficiary,
						fee,
						Preservation::Preserve
					),
					DispatchError::Module(ModuleError {
						index: 2,
						error: [0, 0, 0, 0],
						message: Some("BalanceLow")
					})
				);

				assert_eq!(Assets::balance(WHITELISTED_ASSET_ID, ALICE), alice_balance_before);
			});
		}

		#[test]
		fn transfer_with_vouchers_fails() {
			ExtBuilder::default().vouchers(&[(ALICE, 10)]).build().execute_with(|| {
				let fee = 101;
				let alice_balance_before = Assets::balance(WHITELISTED_ASSET_ID, ALICE);
				let fee_beneficiary = FERDIE;

				let alice_initial_vouchers = VOUCHERS
					.with_borrow(|voucher_store| voucher_store.get(&ALICE).copied())
					.expect("Should contain remaining vouchers");

				assert_noop!(
					TestAssetTransfer::transfer(
						VOUCHER_ASSET_PAYMENT,
						&ALICE,
						&fee_beneficiary,
						fee,
						Preservation::Preserve
					),
					DispatchError::Token(TokenError::Unsupported)
				);

				assert_eq!(Assets::balance(WHITELISTED_ASSET_ID, ALICE), alice_balance_before);

				// check that the vouchers have been untouched
				let alice_current_vouchers = VOUCHERS
					.with_borrow(|voucher_store| voucher_store.get(&ALICE).copied())
					.expect("Should contain remaining vouchers");
				assert_eq!(alice_current_vouchers, alice_initial_vouchers);
			});
		}

		#[test]
		fn transfer_fails_for_not_whitelisted_asset() {
			ExtBuilder::default().build().execute_with(|| {
				let fee = 101;
				let alice_balance_before = Assets::balance(NOT_WHITE_LISTED_ASSET_ID, ALICE);
				let fee_beneficiary = FERDIE;

				assert_noop!(
					TestAssetTransfer::transfer(
						NOT_WHITELISTED_ASSET_ID_PAYMENT,
						&ALICE,
						&fee_beneficiary,
						fee,
						Preservation::Preserve
					),
					DispatchError::Token(TokenError::Unsupported)
				);

				assert_eq!(Assets::balance(NOT_WHITE_LISTED_ASSET_ID, ALICE), alice_balance_before);
			});
		}
	}

	mod transfer_all {
		use super::*;

		#[test]
		fn transfer_all_works() {
			ExtBuilder::default().build().execute_with(|| {
				let alice_balance_before = Assets::balance(WHITELISTED_ASSET_ID, ALICE);
				let fee_beneficiary = FERDIE;

				TestAssetTransfer::transfer_all(
					WHITELISTED_ASSET_ID_PAYMENT,
					&ALICE,
					&fee_beneficiary,
				)
				.unwrap();

				assert_eq!(Assets::balance(WHITELISTED_ASSET_ID, ALICE), 0);
				assert_eq!(
					Assets::balance(WHITELISTED_ASSET_ID, fee_beneficiary),
					alice_balance_before
				)
			});
		}
	}

	#[test]
	fn transfer_all_with_vouchers_fails() {
		ExtBuilder::default().vouchers(&[(ALICE, 10)]).build().execute_with(|| {
			let alice_balance_before = Assets::balance(WHITELISTED_ASSET_ID, ALICE);
			let fee_beneficiary = FERDIE;

			let alice_initial_vouchers = VOUCHERS
				.with_borrow(|voucher_store| voucher_store.get(&ALICE).copied())
				.expect("Should contain remaining vouchers");

			assert_noop!(
				TestAssetTransfer::transfer_all(VOUCHER_ASSET_PAYMENT, &ALICE, &fee_beneficiary,),
				DispatchError::Token(TokenError::Unsupported)
			);

			assert_eq!(Assets::balance(WHITELISTED_ASSET_ID, ALICE), alice_balance_before);

			// check that the vouchers have been untouched
			let alice_current_vouchers = VOUCHERS
				.with_borrow(|voucher_store| voucher_store.get(&ALICE).copied())
				.expect("Should contain remaining vouchers");
			assert_eq!(alice_current_vouchers, alice_initial_vouchers);
		});
	}
}
