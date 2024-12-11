use crate::{
	fee_handler::FeeHandler,
	mock::{
		AffiliateFeeId, Assets, Balances, ExtBuilder, TestAssetFeeHandler, TestNativeFeeHandler,
		TournamentFeeId, ALICE, BOB, CHARLIE, DAVE, FERDIE, NATIVE_ASSET_PAYMENT,
		NOT_WHITELISTED_ASSET_ID_PAYMENT, NOT_WHITE_LISTED_ASSET_ID, TOURNAMENT_TREASURY,
		WHITELISTED_ASSET_ID, WHITELISTED_ASSET_ID_PAYMENT,
	},
};
use frame_support::assert_noop;
use sp_runtime::{DispatchError, ModuleError, TokenError};

mod asset_fee_handler {
	use super::*;

	mod withdraw_and_pay_fees {
		use super::*;

		#[test]
		fn withdraw_and_pay_fee_deposits_all_into_the_treasury() {
			ExtBuilder.build().execute_with(|| {
				let fee = 20;
				let alice_balance_before = Assets::balance(WHITELISTED_ASSET_ID, ALICE);
				let fee_beneficiary = FERDIE;

				TestAssetFeeHandler::withdraw_and_pay_fees(
					&ALICE,
					WHITELISTED_ASSET_ID_PAYMENT,
					fee,
					&TournamentFeeId::Free,
					&AffiliateFeeId::Free,
					&fee_beneficiary,
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
		fn withdraw_and_pay_fee_deposits_into_tournament_and_treasury() {
			ExtBuilder.build().execute_with(|| {
				let fee = 2;
				let alice_balance_before = Assets::balance(WHITELISTED_ASSET_ID, ALICE);
				let fee_beneficiary = FERDIE;

				TestAssetFeeHandler::withdraw_and_pay_fees(
					&ALICE,
					WHITELISTED_ASSET_ID_PAYMENT,
					fee,
					&TournamentFeeId::Paying,
					&AffiliateFeeId::Free,
					&fee_beneficiary,
				)
				.unwrap();

				// check that the fee has been deducted from the payer
				assert_eq!(
					Assets::balance(WHITELISTED_ASSET_ID, ALICE),
					alice_balance_before - fee
				);

				// check allocation
				assert_eq!(
					Assets::balance(WHITELISTED_ASSET_ID, TOURNAMENT_TREASURY),
					fee * 2 / 10
				);

				// Remaining fee after the tournament fee goes to the fee_beneficiary.
				// The math should fix truncation discrepancies due to integer operations.
				assert_eq!(
					Assets::balance(WHITELISTED_ASSET_ID, fee_beneficiary),
					fee - fee * 2 / 10
				);
				assert_eq!(
					Assets::balance(WHITELISTED_ASSET_ID, TOURNAMENT_TREASURY) +
						Assets::balance(WHITELISTED_ASSET_ID, fee_beneficiary),
					fee
				);

				// No money went to the affiliates, as there was no affiliate chain.
				assert_eq!(Assets::balance(WHITELISTED_ASSET_ID, BOB), 0);
				assert_eq!(Assets::balance(WHITELISTED_ASSET_ID, CHARLIE), 0);
				assert_eq!(Assets::balance(WHITELISTED_ASSET_ID, DAVE), 0);
			});
		}

		#[test]
		fn withdraw_and_pay_fee_deposits_to_affiliates_and_treasury() {
			ExtBuilder.build().execute_with(|| {
				let fee = 10;
				let alice_balance_before = Assets::balance(WHITELISTED_ASSET_ID, ALICE);
				let fee_beneficiary = FERDIE;

				TestAssetFeeHandler::withdraw_and_pay_fees(
					&ALICE,
					WHITELISTED_ASSET_ID_PAYMENT,
					fee,
					&TournamentFeeId::Free,
					&AffiliateFeeId::Paying,
					&fee_beneficiary,
				)
				.unwrap();

				// check that the fee has been deducted from the payer
				assert_eq!(
					Assets::balance(WHITELISTED_ASSET_ID, ALICE),
					alice_balance_before - fee
				);

				// check affiliation allocation
				let bob_share = fee * 4 / 20;
				let charlie_share = fee * 3 / 20;
				let dave_share = fee * 2 / 20;
				assert_eq!(Assets::balance(WHITELISTED_ASSET_ID, BOB), bob_share);
				assert_eq!(Assets::balance(WHITELISTED_ASSET_ID, CHARLIE), charlie_share);
				assert_eq!(Assets::balance(WHITELISTED_ASSET_ID, DAVE), dave_share);

				// remaining fee after the tournament fee goes to the fee_beneficiary.
				// The math should fix truncation discrepancies due to integer operations.
				assert_eq!(
					Assets::balance(WHITELISTED_ASSET_ID, fee_beneficiary),
					fee - bob_share - charlie_share - dave_share
				);

				// no money went to the tournament, as it was free
				assert_eq!(Assets::balance(WHITELISTED_ASSET_ID, TOURNAMENT_TREASURY), 0)
			});
		}

		#[test]
		fn withdraw_and_pay_fee_deposits_to_tournament_affiliates_and_treasury() {
			ExtBuilder.build().execute_with(|| {
				let fee = 10;
				let alice_balance_before = Assets::balance(WHITELISTED_ASSET_ID, ALICE);
				let fee_beneficiary = FERDIE;

				TestAssetFeeHandler::withdraw_and_pay_fees(
					&ALICE,
					WHITELISTED_ASSET_ID_PAYMENT,
					fee,
					&TournamentFeeId::Paying,
					&AffiliateFeeId::Paying,
					&fee_beneficiary,
				)
				.unwrap();

				// check that the fee has been deducted from the payer
				assert_eq!(
					Assets::balance(WHITELISTED_ASSET_ID, ALICE),
					alice_balance_before - fee
				);

				// check tournament allocation
				let tournament_share = fee * 2 / 10;
				assert_eq!(
					Assets::balance(WHITELISTED_ASSET_ID, TOURNAMENT_TREASURY),
					tournament_share
				);
				let fee_after_tournament = fee - tournament_share;

				// check affiliation allocation
				let bob_share = fee_after_tournament * 4 / 20;
				let charlie_share = fee_after_tournament * 3 / 20;
				let dave_share = fee_after_tournament * 2 / 20;
				assert_eq!(Assets::balance(WHITELISTED_ASSET_ID, BOB), bob_share);
				assert_eq!(Assets::balance(WHITELISTED_ASSET_ID, CHARLIE), charlie_share);
				assert_eq!(Assets::balance(WHITELISTED_ASSET_ID, DAVE), dave_share);

				// remaining fee after the tournament fee goes to the fee_beneficiary.
				// The math should fix truncation discrepancies due to integer operations.
				assert_eq!(
					Assets::balance(WHITELISTED_ASSET_ID, fee_beneficiary),
					fee - tournament_share - bob_share - charlie_share - dave_share
				);
			});
		}

		#[test]
		fn withdraw_and_pay_fee_fails_if_missing_funds() {
			ExtBuilder.build().execute_with(|| {
				let fee = 101;
				let alice_balance_before = Assets::balance(WHITELISTED_ASSET_ID, ALICE);
				let fee_beneficiary = FERDIE;

				assert_noop!(
					TestAssetFeeHandler::withdraw_and_pay_fees(
						&ALICE,
						WHITELISTED_ASSET_ID_PAYMENT,
						fee,
						&TournamentFeeId::Free,
						&AffiliateFeeId::Free,
						&fee_beneficiary,
					),
					DispatchError::Module(ModuleError {
						index: 3,
						error: [0, 0, 0, 0],
						message: Some("BalanceLow")
					})
				);

				assert_eq!(Assets::balance(WHITELISTED_ASSET_ID, ALICE), alice_balance_before);
			});
		}

		#[test]
		fn withdraw_and_pay_fee_fails_for_not_whitelisted_asset() {
			ExtBuilder.build().execute_with(|| {
				let fee = 2;
				let alice_balance_before = Assets::balance(NOT_WHITE_LISTED_ASSET_ID, ALICE);
				let fee_beneficiary = FERDIE;

				assert_noop!(
					TestAssetFeeHandler::withdraw_and_pay_fees(
						&ALICE,
						NOT_WHITELISTED_ASSET_ID_PAYMENT,
						fee,
						&TournamentFeeId::Free,
						&AffiliateFeeId::Free,
						&fee_beneficiary,
					),
					DispatchError::Token(TokenError::Unsupported)
				);

				assert_eq!(Assets::balance(NOT_WHITE_LISTED_ASSET_ID, ALICE), alice_balance_before);
			});
		}
	}

	mod withdraw_and_deposit_into_treasury {
		use super::*;

		#[test]
		fn withdraw_and_deposit_into_treasury_works() {
			ExtBuilder.build().execute_with(|| {
				let fee = 20;
				let alice_balance_before = Assets::balance(WHITELISTED_ASSET_ID, ALICE);
				let fee_beneficiary = FERDIE;

				TestAssetFeeHandler::withdraw_and_deposit_into_treasury(
					&ALICE,
					WHITELISTED_ASSET_ID_PAYMENT,
					&fee_beneficiary,
					fee,
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
		fn withdraw_and_deposit_into_treasury_fails_if_missing_funds() {
			ExtBuilder.build().execute_with(|| {
				let fee = 101;
				let alice_balance_before = Assets::balance(WHITELISTED_ASSET_ID, ALICE);
				let fee_beneficiary = FERDIE;

				assert_noop!(
					TestAssetFeeHandler::withdraw_and_pay_fees(
						&ALICE,
						WHITELISTED_ASSET_ID_PAYMENT,
						fee,
						&TournamentFeeId::Free,
						&AffiliateFeeId::Free,
						&fee_beneficiary,
					),
					DispatchError::Module(ModuleError {
						index: 3,
						error: [0, 0, 0, 0],
						message: Some("BalanceLow")
					})
				);

				assert_eq!(Assets::balance(WHITELISTED_ASSET_ID, ALICE), alice_balance_before);
			});
		}

		#[test]
		fn withdraw_and_deposit_into_treasury_for_not_whitelisted_asset() {
			ExtBuilder.build().execute_with(|| {
				let fee = 101;
				let alice_balance_before = Assets::balance(NOT_WHITE_LISTED_ASSET_ID, ALICE);
				let fee_beneficiary = FERDIE;

				assert_noop!(
					TestAssetFeeHandler::withdraw_and_pay_fees(
						&ALICE,
						NOT_WHITELISTED_ASSET_ID_PAYMENT,
						fee,
						&TournamentFeeId::Free,
						&AffiliateFeeId::Free,
						&fee_beneficiary,
					),
					DispatchError::Token(TokenError::Unsupported)
				);

				assert_eq!(Assets::balance(NOT_WHITE_LISTED_ASSET_ID, ALICE), alice_balance_before);
			});
		}
	}
}

mod native_fee_handler {
	use super::*;
	use frame_support::traits::fungible::Inspect;

	mod withdraw_and_pay_fees {
		use super::*;

		#[test]
		fn withdraw_and_pay_fee_deposits_all_into_the_treasury() {
			ExtBuilder.build().execute_with(|| {
				let fee = 20;
				let alice_balance_before = Balances::balance(&ALICE);
				let fee_beneficiary = FERDIE;

				TestNativeFeeHandler::withdraw_and_pay_fees(
					&ALICE,
					NATIVE_ASSET_PAYMENT,
					fee,
					&TournamentFeeId::Free,
					&AffiliateFeeId::Free,
					&fee_beneficiary,
				)
				.unwrap();

				assert_eq!(Balances::balance(&ALICE), alice_balance_before - fee);
				assert_eq!(Balances::balance(&fee_beneficiary), fee)
			});
		}

		#[test]
		fn withdraw_and_pay_fee_deposits_into_tournament_and_treasury() {
			ExtBuilder.build().execute_with(|| {
				let fee = 2;
				let alice_balance_before = Balances::balance(&ALICE);
				let fee_beneficiary = FERDIE;

				TestNativeFeeHandler::withdraw_and_pay_fees(
					&ALICE,
					NATIVE_ASSET_PAYMENT,
					fee,
					&TournamentFeeId::Paying,
					&AffiliateFeeId::Free,
					&fee_beneficiary,
				)
				.unwrap();

				// check that the fee has been deducted from the payer
				assert_eq!(Balances::balance(&ALICE), alice_balance_before - fee);

				// check allocation
				assert_eq!(Balances::balance(&TOURNAMENT_TREASURY), fee * 2 / 10);

				// Remaining fee after the tournament fee goes to the fee_beneficiary.
				// The math should fix truncation discrepancies due to integer operations.
				assert_eq!(Balances::balance(&fee_beneficiary), fee - fee * 2 / 10);
				assert_eq!(
					Balances::balance(&TOURNAMENT_TREASURY) + Balances::balance(&fee_beneficiary),
					fee
				);

				// No money went to the affiliates, as there was no affiliate chain.
				assert_eq!(Balances::balance(&BOB), 0);
				assert_eq!(Balances::balance(&CHARLIE), 0);
				assert_eq!(Balances::balance(&DAVE), 0);
			});
		}

		#[test]
		fn withdraw_and_pay_fee_deposits_to_affiliates_and_treasury() {
			ExtBuilder.build().execute_with(|| {
				let fee = 10;
				let alice_balance_before = Balances::balance(&ALICE);
				let fee_beneficiary = FERDIE;

				TestNativeFeeHandler::withdraw_and_pay_fees(
					&ALICE,
					NATIVE_ASSET_PAYMENT,
					fee,
					&TournamentFeeId::Free,
					&AffiliateFeeId::Paying,
					&fee_beneficiary,
				)
				.unwrap();

				// check that the fee has been deducted from the payer
				assert_eq!(Balances::balance(&ALICE), alice_balance_before - fee);

				// check affiliation allocation
				let bob_share = fee * 4 / 20;
				let charlie_share = fee * 3 / 20;
				let dave_share = fee * 2 / 20;
				assert_eq!(Balances::balance(&BOB), bob_share);
				assert_eq!(Balances::balance(&CHARLIE), charlie_share);
				assert_eq!(Balances::balance(&DAVE), dave_share);

				// remaining fee after the tournament fee goes to the fee_beneficiary.
				// The math should fix truncation discrepancies due to integer operations.
				assert_eq!(
					Balances::balance(&fee_beneficiary),
					fee - bob_share - charlie_share - dave_share
				);

				// no money went to the tournament, as it was free
				assert_eq!(Balances::balance(&TOURNAMENT_TREASURY), 0)
			});
		}

		#[test]
		fn withdraw_and_pay_fee_deposits_to_tournament_affiliates_and_treasury() {
			ExtBuilder.build().execute_with(|| {
				let fee = 10;
				let alice_balance_before = Balances::balance(&ALICE);
				let fee_beneficiary = FERDIE;

				TestNativeFeeHandler::withdraw_and_pay_fees(
					&ALICE,
					NATIVE_ASSET_PAYMENT,
					fee,
					&TournamentFeeId::Paying,
					&AffiliateFeeId::Paying,
					&fee_beneficiary,
				)
				.unwrap();

				// check that the fee has been deducted from the payer
				assert_eq!(Balances::balance(&ALICE), alice_balance_before - fee);

				// check tournament allocation
				let tournament_share = fee * 2 / 10;
				assert_eq!(Balances::balance(&TOURNAMENT_TREASURY), tournament_share);
				let fee_after_tournament = fee - tournament_share;

				// check affiliation allocation
				let bob_share = fee_after_tournament * 4 / 20;
				let charlie_share = fee_after_tournament * 3 / 20;
				let dave_share = fee_after_tournament * 2 / 20;
				assert_eq!(Balances::balance(&BOB), bob_share);
				assert_eq!(Balances::balance(&CHARLIE), charlie_share);
				assert_eq!(Balances::balance(&DAVE), dave_share);

				// remaining fee after the tournament fee goes to the fee_beneficiary.
				// The math should fix truncation discrepancies due to integer operations.
				assert_eq!(
					Balances::balance(&fee_beneficiary),
					fee - tournament_share - bob_share - charlie_share - dave_share
				);
			});
		}

		#[test]
		fn withdraw_and_pay_fee_fails_if_missing_funds() {
			ExtBuilder.build().execute_with(|| {
				let fee = 101;
				let alice_balance_before = Balances::balance(&ALICE);
				let fee_beneficiary = FERDIE;

				assert_noop!(
					TestNativeFeeHandler::withdraw_and_pay_fees(
						&ALICE,
						NATIVE_ASSET_PAYMENT,
						fee,
						&TournamentFeeId::Free,
						&AffiliateFeeId::Free,
						&fee_beneficiary,
					),
					DispatchError::Token(TokenError::FundsUnavailable)
				);

				assert_eq!(Balances::balance(&ALICE), alice_balance_before);
			});
		}
	}

	mod withdraw_and_deposit_into_treasury {
		use super::*;
		use crate::mock::VoucherBalances;

		#[test]
		fn withdraw_and_deposit_into_treasury_works() {
			ExtBuilder.build().execute_with(|| {
				let fee = 20;
				let alice_balance_before = Balances::balance(&ALICE);
				let fee_beneficiary = FERDIE;

				TestNativeFeeHandler::withdraw_and_deposit_into_treasury(
					&ALICE,
					NATIVE_ASSET_PAYMENT,
					&fee_beneficiary,
					fee,
				)
				.unwrap();

				assert_eq!(Balances::balance(&ALICE), alice_balance_before - fee);
				assert_eq!(Balances::balance(&fee_beneficiary), fee)
			});
		}

		#[test]
		fn withdraw_and_deposit_into_treasury_fails_if_missing_funds() {
			ExtBuilder.build().execute_with(|| {
				let fee = 101;
				let alice_balance_before = Balances::balance(&ALICE);
				let fee_beneficiary = FERDIE;

				assert_noop!(
					TestNativeFeeHandler::withdraw_and_pay_fees(
						&ALICE,
						NATIVE_ASSET_PAYMENT,
						fee,
						&TournamentFeeId::Free,
						&AffiliateFeeId::Free,
						&fee_beneficiary,
					),
					DispatchError::Token(TokenError::FundsUnavailable)
				);

				assert_eq!(Balances::balance(&ALICE), alice_balance_before);
			});
		}
	}
}
