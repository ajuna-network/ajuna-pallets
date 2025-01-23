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
		use crate::mock::{VOUCHERS, VOUCHER_ASSET_PAYMENT};

		#[test]
		fn withdraw_and_pay_fee_deposits_all_into_the_treasury() {
			ExtBuilder::default().build().execute_with(|| {
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
			ExtBuilder::default().build().execute_with(|| {
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
			ExtBuilder::default().build().execute_with(|| {
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
			ExtBuilder::default().build().execute_with(|| {
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
		fn withdraw_and_pay_fee_deposits_to_tournament_affiliates_and_treasury_works_with_vouchers()
		{
			ExtBuilder::default().vouchers(&[(ALICE, 5)]).build().execute_with(|| {
				let fee = 2;
				let alice_balance_before = Assets::balance(WHITELISTED_ASSET_ID, ALICE);
				let fee_beneficiary = FERDIE;

				let alice_initial_vouchers = VOUCHERS
					.with_borrow(|voucher_store| voucher_store.get(&ALICE).copied())
					.expect("Should contain remaining vouchers");

				TestAssetFeeHandler::withdraw_and_pay_fees(
					&ALICE,
					VOUCHER_ASSET_PAYMENT,
					fee,
					&TournamentFeeId::Paying,
					&AffiliateFeeId::Paying,
					&fee_beneficiary,
				)
				.unwrap();

				// check that the fee has not been deducted from the payer
				assert_eq!(Assets::balance(WHITELISTED_ASSET_ID, ALICE), alice_balance_before);

				// check that no tournament allocation has occurred
				assert_eq!(Assets::balance(WHITELISTED_ASSET_ID, TOURNAMENT_TREASURY), 0);

				// check that no fee has also been allocated to the affiliates
				assert_eq!(Assets::balance(WHITELISTED_ASSET_ID, BOB), 0);
				assert_eq!(Assets::balance(WHITELISTED_ASSET_ID, CHARLIE), 0);
				assert_eq!(Assets::balance(WHITELISTED_ASSET_ID, DAVE), 0);

				// check that the requested vouchers have been deducted from storage
				let alice_current_vouchers = VOUCHERS
					.with_borrow(|voucher_store| voucher_store.get(&ALICE).copied())
					.expect("Should contain remaining vouchers");
				assert_eq!(alice_current_vouchers, alice_initial_vouchers - fee);
			});
		}

		#[test]
		fn withdraw_and_pay_fee_fails_if_missing_funds() {
			ExtBuilder::default().build().execute_with(|| {
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
						index: 2,
						error: [0, 0, 0, 0],
						message: Some("BalanceLow")
					})
				);

				assert_eq!(Assets::balance(WHITELISTED_ASSET_ID, ALICE), alice_balance_before);
			});
		}

		#[test]
		fn withdraw_and_pay_fee_fails_for_not_whitelisted_asset() {
			ExtBuilder::default().build().execute_with(|| {
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

		#[test]
		fn withdraw_and_pay_fee_fails_if_missing_vouchers() {
			ExtBuilder::default().vouchers(&[(ALICE, 10)]).build().execute_with(|| {
				let fee = 101;
				let alice_balance_before = Assets::balance(WHITELISTED_ASSET_ID, ALICE);
				let fee_beneficiary = FERDIE;

				let alice_initial_vouchers = VOUCHERS
					.with_borrow(|voucher_store| voucher_store.get(&ALICE).copied())
					.expect("Should contain remaining vouchers");

				assert_noop!(
					TestAssetFeeHandler::withdraw_and_pay_fees(
						&ALICE,
						VOUCHER_ASSET_PAYMENT,
						fee,
						&TournamentFeeId::Free,
						&AffiliateFeeId::Free,
						&fee_beneficiary,
					),
					DispatchError::Token(TokenError::FundsUnavailable)
				);

				assert_eq!(Assets::balance(WHITELISTED_ASSET_ID, ALICE), alice_balance_before);

				// check that the vouchers have been untouched
				let alice_current_vouchers = VOUCHERS
					.with_borrow(|voucher_store| voucher_store.get(&ALICE).copied())
					.expect("Should contain remaining vouchers");
				assert_eq!(alice_initial_vouchers, alice_current_vouchers);
			});
		}
	}

	mod withdraw_and_deposit_into_treasury {
		use super::*;
		use crate::mock::{VOUCHERS, VOUCHER_ASSET_PAYMENT};

		#[test]
		fn withdraw_and_deposit_into_treasury_works() {
			ExtBuilder::default().build().execute_with(|| {
				let fee = 20;
				let alice_balance_before = Assets::balance(WHITELISTED_ASSET_ID, ALICE);
				let fee_beneficiary = FERDIE;

				TestAssetFeeHandler::withdraw_and_deposit_into(
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
		fn withdraw_and_deposit_into_treasury_with_vouchers_does_not_err() {
			// This test is meant to show how using vouchers with the
			// 'withdraw_and_deposit_into_treasury' does not actually store anything in the
			// treasury, but that it also does not fail.
			ExtBuilder::default().vouchers(&[(ALICE, 15)]).build().execute_with(|| {
				let fee = 15;
				let alice_balance_before = Assets::balance(WHITELISTED_ASSET_ID, ALICE);
				let fee_beneficiary = FERDIE;

				let alice_initial_vouchers = VOUCHERS
					.with_borrow(|voucher_store| voucher_store.get(&ALICE).copied())
					.expect("Should contain remaining vouchers");

				TestAssetFeeHandler::withdraw_and_deposit_into(
					&ALICE,
					VOUCHER_ASSET_PAYMENT,
					&fee_beneficiary,
					fee,
				)
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
		fn withdraw_and_deposit_into_treasury_fails_if_missing_funds() {
			ExtBuilder::default().build().execute_with(|| {
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
						index: 2,
						error: [0, 0, 0, 0],
						message: Some("BalanceLow")
					})
				);

				assert_eq!(Assets::balance(WHITELISTED_ASSET_ID, ALICE), alice_balance_before);
			});
		}

		#[test]
		fn withdraw_and_deposit_into_treasury_fails_if_missing_vouchers() {
			ExtBuilder::default().vouchers(&[(ALICE, 10)]).build().execute_with(|| {
				let fee = 101;
				let alice_balance_before = Assets::balance(WHITELISTED_ASSET_ID, ALICE);
				let fee_beneficiary = FERDIE;

				let alice_initial_vouchers = VOUCHERS
					.with_borrow(|voucher_store| voucher_store.get(&ALICE).copied())
					.expect("Should contain remaining vouchers");

				assert_noop!(
					TestAssetFeeHandler::withdraw_and_pay_fees(
						&ALICE,
						VOUCHER_ASSET_PAYMENT,
						fee,
						&TournamentFeeId::Free,
						&AffiliateFeeId::Free,
						&fee_beneficiary,
					),
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
		fn withdraw_and_deposit_into_treasury_for_not_whitelisted_asset() {
			ExtBuilder::default().build().execute_with(|| {
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
		use crate::mock::{VOUCHERS, VOUCHER_NATIVE_PAYMENT};

		#[test]
		fn withdraw_and_pay_fee_deposits_all_into_the_treasury() {
			ExtBuilder::default().build().execute_with(|| {
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
			ExtBuilder::default().build().execute_with(|| {
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
			ExtBuilder::default().build().execute_with(|| {
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
			ExtBuilder::default().build().execute_with(|| {
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
		fn withdraw_and_pay_fee_deposits_to_tournament_affiliates_and_treasury_works_with_vouchers()
		{
			ExtBuilder::default().vouchers(&[(ALICE, 3)]).build().execute_with(|| {
				let fee = 2;
				let alice_balance_before = Balances::balance(&ALICE);
				let fee_beneficiary = FERDIE;

				let alice_initial_vouchers = VOUCHERS
					.with_borrow(|voucher_store| voucher_store.get(&ALICE).copied())
					.expect("Should contain remaining vouchers");

				TestNativeFeeHandler::withdraw_and_pay_fees(
					&ALICE,
					VOUCHER_NATIVE_PAYMENT,
					fee,
					&TournamentFeeId::Paying,
					&AffiliateFeeId::Paying,
					&fee_beneficiary,
				)
				.unwrap();

				// check that the fee has not been deducted from the payer's balance
				assert_eq!(Balances::balance(&ALICE), alice_balance_before);

				// check no tournament allocation happened
				assert_eq!(Balances::balance(&TOURNAMENT_TREASURY), 0);

				// check no affiliation allocation happened
				assert_eq!(Balances::balance(&BOB), 0);
				assert_eq!(Balances::balance(&CHARLIE), 0);
				assert_eq!(Balances::balance(&DAVE), 0);

				// check that the requested vouchers have been deducted from storage
				let alice_current_vouchers = VOUCHERS
					.with_borrow(|voucher_store| voucher_store.get(&ALICE).copied())
					.expect("Should contain remaining vouchers");
				assert_eq!(alice_current_vouchers, alice_initial_vouchers - fee);
			});
		}

		#[test]
		fn withdraw_and_pay_fee_fails_if_missing_funds() {
			ExtBuilder::default().build().execute_with(|| {
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

		#[test]
		fn withdraw_and_pay_fee_fails_if_missing_vouchers() {
			ExtBuilder::default().vouchers(&[(ALICE, 45)]).build().execute_with(|| {
				let fee = 101;
				let alice_balance_before = Balances::balance(&ALICE);
				let fee_beneficiary = FERDIE;

				let alice_initial_vouchers = VOUCHERS
					.with_borrow(|voucher_store| voucher_store.get(&ALICE).copied())
					.expect("Should contain remaining vouchers");

				assert_noop!(
					TestNativeFeeHandler::withdraw_and_pay_fees(
						&ALICE,
						VOUCHER_NATIVE_PAYMENT,
						fee,
						&TournamentFeeId::Free,
						&AffiliateFeeId::Free,
						&fee_beneficiary,
					),
					DispatchError::Token(TokenError::FundsUnavailable)
				);

				assert_eq!(Balances::balance(&ALICE), alice_balance_before);

				// check that the vouchers have been untouched
				let alice_current_vouchers = VOUCHERS
					.with_borrow(|voucher_store| voucher_store.get(&ALICE).copied())
					.expect("Should contain remaining vouchers");
				assert_eq!(alice_current_vouchers, alice_initial_vouchers);
			});
		}
	}

	mod withdraw_and_deposit_into_treasury {
		use super::*;
		use crate::mock::{VOUCHERS, VOUCHER_NATIVE_PAYMENT};

		#[test]
		fn withdraw_and_deposit_into_treasury_works() {
			ExtBuilder::default().build().execute_with(|| {
				let fee = 20;
				let alice_balance_before = Balances::balance(&ALICE);
				let fee_beneficiary = FERDIE;

				TestNativeFeeHandler::withdraw_and_deposit_into(
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
		fn withdraw_and_deposit_into_treasury_with_vouchers_does_not_err() {
			// This test is meant to show how using vouchers with the
			// 'withdraw_and_deposit_into_treasury' does not actually store anything in the
			// treasury, but that it also does not fail.
			ExtBuilder::default().vouchers(&[(ALICE, 8)]).build().execute_with(|| {
				let fee = 1;
				let alice_balance_before = Balances::balance(&ALICE);
				let fee_beneficiary = FERDIE;

				let alice_initial_vouchers = VOUCHERS
					.with_borrow(|voucher_store| voucher_store.get(&ALICE).copied())
					.expect("Should contain remaining vouchers");

				TestNativeFeeHandler::withdraw_and_deposit_into(
					&ALICE,
					VOUCHER_NATIVE_PAYMENT,
					&fee_beneficiary,
					fee,
				)
				.unwrap();

				assert_eq!(Balances::balance(&ALICE), alice_balance_before);
				assert_eq!(Balances::balance(&fee_beneficiary), 0);

				// check that the requested vouchers have been deducted from storage
				let alice_current_vouchers = VOUCHERS
					.with_borrow(|voucher_store| voucher_store.get(&ALICE).copied())
					.expect("Should contain remaining vouchers");
				assert_eq!(alice_current_vouchers, alice_initial_vouchers - fee);
			});
		}

		#[test]
		fn withdraw_and_deposit_into_treasury_fails_if_missing_funds() {
			ExtBuilder::default().build().execute_with(|| {
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

		#[test]
		fn withdraw_and_deposit_into_treasury_fails_if_missing_vouchers() {
			ExtBuilder::default().vouchers(&[(ALICE, 99)]).build().execute_with(|| {
				let fee = 101;
				let alice_balance_before = Balances::balance(&ALICE);
				let fee_beneficiary = FERDIE;

				let alice_initial_vouchers = VOUCHERS
					.with_borrow(|voucher_store| voucher_store.get(&ALICE).copied())
					.expect("Should contain remaining vouchers");

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

				// check that the vouchers have been untouched
				let alice_current_vouchers = VOUCHERS
					.with_borrow(|voucher_store| voucher_store.get(&ALICE).copied())
					.expect("Should contain remaining vouchers");
				assert_eq!(alice_current_vouchers, alice_initial_vouchers);
			});
		}
	}
}
