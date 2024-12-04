use crate::{
	fee_handler::FeeHandler,
	mock::{
		AffiliateFeeId, Assets, ExtBuilder, TestFeeHandler, TournamentFeeId, ALICE, TREASURER,
		WHITELISTED_ASSET_ID,
	},
};

mod withdraw_and_pay_fees {
	use super::*;

	#[test]
	fn withdraw_and_pay_fee_deposits_all_into_the_treasury() {
		ExtBuilder::default().build().execute_with(|| {
			let fee = 2;
			let alice_balance_before = Assets::balance(WHITELISTED_ASSET_ID, ALICE);

			TestFeeHandler::withdraw_and_pay_fees(
				&ALICE,
				WHITELISTED_ASSET_ID,
				fee,
				&TournamentFeeId::Free,
				&AffiliateFeeId::Free,
				&TREASURER,
			)
			.unwrap();

			assert_eq!(Assets::balance(WHITELISTED_ASSET_ID, ALICE), alice_balance_before - fee);
			assert_eq!(Assets::balance(WHITELISTED_ASSET_ID, TREASURER), fee)
		});
	}
}
