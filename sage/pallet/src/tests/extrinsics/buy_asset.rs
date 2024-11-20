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

use super::*;

#[test]
fn buy_should_work() {
	let initial_balance = 1_000_000;

	ExtBuilder::default()
		.balances(&[
			(ALICE, initial_balance),
			(BOB, initial_balance),
			(CHARLIE, initial_balance),
			(DAVE, initial_balance),
		])
		.locks(&[
			(ALICE, SEASON_ID_0, Locks::all_unlocked()),
			(BOB, SEASON_ID_0, Locks::all_unlocked()),
			(ALICE, SEASON_ID_1, Locks::all_unlocked()),
			(BOB, SEASON_ID_1, Locks::all_unlocked()),
		])
		.build()
		.execute_with(|| {
			let season_config_0 =
				<Test as Config<Instance1>>::SeasonHandler::get_season_config_for(&SEASON_ID_0)
					.expect("Should get season config");
			let season_fees_0 = season_config_0.fee;

			let asset_ids = create_assets::<Instance1>(SEASON_ID_0, BOB, 3);

			let owned_by_alice = AssetOwners::<Test, Instance1>::iter_prefix((ALICE, SEASON_ID_0))
				.map(|(asset_id, _)| asset_id)
				.collect::<Vec<_>>();
			let owned_by_bob = AssetOwners::<Test, Instance1>::iter_prefix((BOB, SEASON_ID_0))
				.map(|(asset_id, _)| asset_id)
				.collect::<Vec<_>>();

			let asset_for_sale = asset_ids[0];
			let asset_price = 4_417;
			assert_ok!(Sage::set_asset_price(
				RuntimeOrigin::signed(BOB),
				asset_for_sale,
				asset_price
			));
			assert_ok!(Sage::buy_asset(RuntimeOrigin::signed(ALICE), asset_for_sale));

			// check for balance transfer
			let price_fee = asset_price
				.saturating_mul(season_fees_0.buy_percent as u64)
				.saturating_div(MAX_PERCENTAGE as u64);
			assert_eq!(Balances::free_balance(ALICE), initial_balance - asset_price - price_fee);
			assert_eq!(Balances::free_balance(BOB), initial_balance + asset_price);

			// check for ownership transfer
			assert_eq!(
				AssetOwners::<Test, Instance1>::iter_prefix((ALICE, SEASON_ID_0)).count(),
				owned_by_alice.len() + 1
			);
			assert_eq!(
				AssetOwners::<Test, Instance1>::iter_prefix((BOB, SEASON_ID_0)).count(),
				owned_by_bob.len() - 1
			);
			assert!(AssetOwners::<Test, Instance1>::contains_key((
				ALICE,
				SEASON_ID_0,
				asset_for_sale
			)));
			assert!(!AssetOwners::<Test, Instance1>::contains_key((
				BOB,
				SEASON_ID_0,
				asset_for_sale
			)));
			assert_eq!(Assets::<Test, Instance1>::get(asset_for_sale).unwrap().0, ALICE);

			// check for removal from trade storage
			assert_eq!(AssetTradePrices::<Test, Instance1>::get(SEASON_ID_0, asset_for_sale), None);

			// check for account stats
			assert_eq!(
				PlayerSeasonStats::<Test, Instance1>::get(ALICE, SEASON_ID_0).bought_amount,
				1
			);
			assert_eq!(PlayerSeasonStats::<Test, Instance1>::get(BOB, SEASON_ID_0).sold_amount, 1);

			// check events
			System::assert_last_event(RuntimeEvent::Sage(Event::AssetTraded {
				asset_id: asset_for_sale,
				from: BOB,
				to: ALICE,
				price: asset_price,
			}));

			// charlie buys from bob
			let asset_for_sale = asset_ids[1];
			let asset_price = 1_357;
			assert_ok!(Sage::set_asset_price(
				RuntimeOrigin::signed(BOB),
				asset_for_sale,
				asset_price
			));
			assert_ok!(Sage::buy_asset(RuntimeOrigin::signed(CHARLIE), asset_for_sale));
			assert_eq!(
				PlayerSeasonStats::<Test, Instance1>::get(CHARLIE, SEASON_ID_0).bought_amount,
				1
			);
			assert_eq!(PlayerSeasonStats::<Test, Instance1>::get(BOB, SEASON_ID_0).sold_amount, 2);

			// check season id
			let asset_on_sale = create_assets::<Instance1>(SEASON_ID_1, ALICE, 1)[0];
			let asset_price = 369;
			assert_ok!(Sage::set_asset_price(
				RuntimeOrigin::signed(ALICE),
				asset_on_sale,
				asset_price
			));
			assert_ok!(Sage::buy_asset(RuntimeOrigin::signed(DAVE), asset_on_sale));
			// Since the current season is SEASON_ID_0 the stat changes are applied to that season
			// not SEASON_ID_1
			let current_season_id =
				<Test as Config<Instance1>>::SeasonHandler::get_current_season_id();
			assert_eq!(current_season_id, SEASON_ID_0);
			// changes in SEASON_ID_0
			assert_eq!(
				PlayerSeasonStats::<Test, Instance1>::get(ALICE, SEASON_ID_0).sold_amount,
				1
			);
			assert_eq!(
				PlayerSeasonStats::<Test, Instance1>::get(DAVE, SEASON_ID_0).bought_amount,
				1
			);
			// no changes were applied to SEASON_ID_1 stats
			assert_eq!(
				PlayerSeasonStats::<Test, Instance1>::get(ALICE, SEASON_ID_1).sold_amount,
				0
			);
			assert_eq!(
				PlayerSeasonStats::<Test, Instance1>::get(DAVE, SEASON_ID_1).bought_amount,
				0
			);
		});
}

#[test]
fn buy_fee_should_be_calculated_correctly() {
	let initial_balance = 100_000;
	ExtBuilder::default()
		.balances(&[(ALICE, initial_balance), (BOB, initial_balance)])
		.locks(&[(ALICE, SEASON_ID_0, Locks::all_unlocked())])
		.build()
		.execute_with(|| {
			let season_config_0 =
				<Test as Config<Instance1>>::SeasonHandler::get_season_config_for(&SEASON_ID_0)
					.expect("Should get season config");
			let season_fees_0 = season_config_0.fee;

			let asset_ids = create_assets::<Instance1>(SEASON_ID_0, ALICE, 2);

			let asset_price = 9_999;
			assert_ok!(Sage::set_asset_price(
				RuntimeOrigin::signed(ALICE),
				asset_ids[0],
				asset_price
			));
			// If the 'buy_percent' makes the fee greater than 'min_buy_fee' then fee will be
			// calculated from the asset_price
			let price_fee_1 = asset_price
				.saturating_mul(season_fees_0.buy_percent as u64)
				.saturating_div(MAX_PERCENTAGE as u64);
			assert!(price_fee_1 > season_fees_0.buy_asset_min);
			assert_ok!(Sage::buy_asset(RuntimeOrigin::signed(BOB), asset_ids[0]));
			// We check that the fees have been paid
			assert_eq!(Balances::free_balance(BOB), initial_balance - asset_price - price_fee_1);
			assert_eq!(Balances::free_balance(ALICE), initial_balance + asset_price);

			let asset_price_2 = 10;
			assert_ok!(Sage::set_asset_price(
				RuntimeOrigin::signed(ALICE),
				asset_ids[1],
				asset_price_2
			));
			// If the 'buy_percent' makes the fee lower than 'min_buy_fee' then fee will be
			// calculated using that value
			let price_fee_2 = asset_price_2
				.saturating_mul(season_fees_0.buy_percent as u64)
				.saturating_div(MAX_PERCENTAGE as u64);
			assert!(price_fee_2 < season_fees_0.buy_asset_min);
			assert_ok!(Sage::buy_asset(RuntimeOrigin::signed(BOB), asset_ids[1]));
			assert_eq!(
				Balances::free_balance(BOB),
				initial_balance -
					asset_price - price_fee_1 -
					asset_price_2 - season_fees_0.buy_asset_min
			);
			assert_eq!(
				Balances::free_balance(ALICE),
				initial_balance + asset_price + asset_price_2
			);
		});
}

#[test]
fn buy_should_reject_when_trading_is_closed() {
	ExtBuilder::default().build().execute_with(|| {
		GeneralConfigStore::<Test, Instance1>::mutate(|config| config.trade.open = false);
		assert_noop!(
			Sage::buy_asset(RuntimeOrigin::signed(ALICE), AssetId::random()),
			Error::<Test, Instance1>::TradeClosed,
		);
	});
}

#[test]
fn buy_should_reject_unsigned_calls() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Sage::buy_asset(RuntimeOrigin::none(), AssetId::random()),
			DispatchError::BadOrigin,
		);
	});
}

#[test]
fn buy_should_reject_unlisted_asset() {
	ExtBuilder::default().build().execute_with(|| {
		let asset_ids = create_assets::<Instance1>(SEASON_ID_0, ALICE, 1);
		assert_noop!(
			Sage::buy_asset(RuntimeOrigin::signed(BOB), asset_ids[0]),
			Error::<Test, Instance1>::AssetNotInTrade,
		);
	});
}

#[test]
fn buy_should_reject_insufficient_balance() {
	let alice_initial_balance = 10_000;
	ExtBuilder::default()
		.balances(&[(ALICE, alice_initial_balance), (BOB, alice_initial_balance * 2)])
		.locks(&[(BOB, SEASON_ID_0, Locks::all_unlocked())])
		.build()
		.execute_with(|| {
			let asset_ids = create_assets::<Instance1>(SEASON_ID_0, BOB, 3);
			let asset_for_sale = asset_ids[0];
			let asset_price = alice_initial_balance + 1;

			assert_ok!(Sage::set_asset_price(
				RuntimeOrigin::signed(BOB),
				asset_for_sale,
				asset_price
			));
			assert_noop!(
				Sage::buy_asset(RuntimeOrigin::signed(ALICE), asset_for_sale),
				sp_runtime::TokenError::FundsUnavailable
			);
		});
}

#[test]
fn buy_should_reject_when_buyer_tries_to_buy_own_asset() {
	ExtBuilder::default()
		.locks(&[(BOB, SEASON_ID_0, Locks::all_unlocked())])
		.build()
		.execute_with(|| {
			let asset_ids = create_assets::<Instance1>(SEASON_ID_0, BOB, 3);
			let asset_for_sale = asset_ids[0];
			let asset_price = 749;

			assert_ok!(Sage::set_asset_price(
				RuntimeOrigin::signed(BOB),
				asset_for_sale,
				asset_price
			));
			assert_noop!(
				Sage::buy_asset(RuntimeOrigin::signed(BOB), asset_for_sale),
				Error::<Test, Instance1>::AlreadyOwned
			);
		});
}
