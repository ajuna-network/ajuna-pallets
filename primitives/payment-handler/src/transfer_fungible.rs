use frame_support::pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo};
use crate::{IdentifyVoucherOrAssetId, WithdrawCredit};
use frame_support::traits::{
	fungibles,
	tokens::{Fortitude, Preservation},
};
use sp_runtime::{DispatchError, TokenError};
use sp_std::marker::PhantomData;

/// Abstraction around transferring funds without disclosing the
/// internals of how the funds are managed.
///
/// The main benefit of this trait is that the uses doesn't have
/// to care if the fungible or the fungibles is used behind the
/// scenes. The `AssetId` will be `()` for the fungible
/// implementation.
pub trait TransferFungible {
	type AccountId;

	type AssetId;

	type Balance;

	fn transfer(
		asset_id: Self::AssetId,
		from: &Self::AccountId,
		to: &Self::AccountId,
		amount: Self::Balance,
		preservation: Preservation
	) -> Result<TransferResult<Self::AssetId, Self::Balance>, DispatchError>;

	fn transfer_all(
		asset_id: Self::AssetId,
		from: &Self::AccountId,
		to: &Self::AccountId,
	) -> Result<TransferResult<Self::AssetId, Self::Balance>, DispatchError>;
}

pub struct TransferAll<W, I>(PhantomData<(W, I)>);

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Debug, Default, Copy, Clone, PartialEq)]
pub struct TransferResult<AssetId, Amount> {
	pub asset_id: AssetId,
	pub amount: Amount,
}

impl<AccountId, Assets, W, I> TransferFungible for TransferAll<W, I>
where
	Assets: fungibles::Balanced<AccountId, Balance = W::Balance>
		+ fungibles::Inspect<AccountId, Balance = W::Balance, AssetId = I::AssetId>,
	W: WithdrawCredit<
		AccountId = AccountId,
		Assets = Assets,
		Credit = fungibles::Credit<AccountId, Assets>,
		AssetId = I,
	>,
	I: IdentifyVoucherOrAssetId,
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
		preservation: Preservation
	) -> Result<TransferResult<Self::AssetId, Self::Balance>, DispatchError> {
		if let Some(credit) = W::withdraw_credit(from, asset_id, amount, preservation)? {
			let result = TransferResult {
				asset_id: credit.asset().into(),
				amount: credit.peek()
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
			// The asset id is a voucher or anything else not-relating to fungible assets.
			log::error!("No credit withdrawn, maybe the asset was a voucher ");
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
			log::error!("No credit withdrawn, maybe the asset was a voucher ");
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

	type TestAssetTransfer = TransferAll<
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

				TestAssetTransfer::transfer(WHITELISTED_ASSET_ID_PAYMENT, &ALICE, &fee_beneficiary, fee, Preservation::Preserve)
					.unwrap();

				assert_eq!(
					Assets::balance(WHITELISTED_ASSET_ID, ALICE),
					alice_balance_before - fee
				);
				assert_eq!(Assets::balance(WHITELISTED_ASSET_ID, fee_beneficiary), fee)
			});
		}

		#[test]
		fn transfer_with_vouchers_does_not_err() {
			// This test is meant to show how using vouchers with the
			// 'withdraw_and_deposit_into' does not actually store anything in the
			// beneficiary, but that it also does not fail.
			ExtBuilder::default().vouchers(&[(ALICE, 15)]).build().execute_with(|| {
				let fee = 15;
				let alice_balance_before = Assets::balance(WHITELISTED_ASSET_ID, ALICE);
				let fee_beneficiary = FERDIE;

				let alice_initial_vouchers = VOUCHERS
					.with_borrow(|voucher_store| voucher_store.get(&ALICE).copied())
					.expect("Should contain remaining vouchers");

				TestAssetTransfer::transfer(VOUCHER_ASSET_PAYMENT, &ALICE, &fee_beneficiary, fee, Preservation::Preserve)
					.unwrap();

				assert_eq!(Assets::balance(WHITELISTED_ASSET_ID, ALICE), alice_balance_before);
				assert_eq!(Assets::balance(WHITELISTED_ASSET_ID, fee_beneficiary), 0);

				// check that the requested vouchers have been deducted from storage
				let alice_current_vouchers = VOUCHERS
					.with_borrow(|voucher_store| voucher_store.get(&ALICE).copied())
					.expect("Should contain remaining vouchers");
				assert_eq!(alice_current_vouchers, alice_initial_vouchers - fee);
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
		fn transfer_fails_if_missing_vouchers() {
			ExtBuilder::default().vouchers(&[(ALICE, 10)]).build().execute_with(|| {
				let fee = 101;
				let alice_balance_before = Assets::balance(WHITELISTED_ASSET_ID, ALICE);
				let fee_beneficiary = FERDIE;

				let alice_initial_vouchers = VOUCHERS
					.with_borrow(|voucher_store| voucher_store.get(&ALICE).copied())
					.expect("Should contain remaining vouchers");

				assert_noop!(
					TestAssetTransfer::transfer(VOUCHER_ASSET_PAYMENT, &ALICE, &fee_beneficiary, fee, Preservation::Preserve),
					DispatchError::Token(TokenError::FundsUnavailable)
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

				TestAssetTransfer::transfer_all(WHITELISTED_ASSET_ID_PAYMENT, &ALICE, &fee_beneficiary)
					.unwrap();

				assert_eq!(
					Assets::balance(WHITELISTED_ASSET_ID, ALICE),
					0
				);
				assert_eq!(Assets::balance(WHITELISTED_ASSET_ID, fee_beneficiary), alice_balance_before)
			});
		}
	}
}
