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
fn remove_price_should_work() {
	ExtBuilder::default()
		.organizer(ALICE)
		.locks(&[(BOB, SEASON_ID_0, Locks::all_unlocked())])
		.build()
		.execute_with(|| {
			let filter = AssetFilter::Trade(VariantType::Player(PlayerType::Human));
			assert_ok!(Sage::update_asset_filter(
				RuntimeOrigin::signed(ALICE),
				SEASON_ID_0,
				filter
			));

			let asset_ids = create_assets::<()>(SEASON_ID_0, BOB, 2);
			let asset_for_sale = asset_ids[0];
			let price = 101;

			assert_ok!(Sage::set_asset_price(RuntimeOrigin::signed(BOB), asset_for_sale, price));

			assert_eq!(AssetTradePrices::<Test, ()>::get(SEASON_ID_0, asset_for_sale), Some(101));
			assert_ok!(Sage::remove_asset_price(RuntimeOrigin::signed(BOB), asset_for_sale));
			assert_eq!(AssetTradePrices::<Test, ()>::get(SEASON_ID_0, asset_for_sale), None);
			System::assert_last_event(RuntimeEvent::Sage(Event::AssetPriceUnset {
				asset_id: asset_for_sale,
			}));
		});
}

#[test]
fn remove_price_should_reject_when_trading_is_closed() {
	ExtBuilder::default().build().execute_with(|| {
		GeneralConfigStore::<Test, ()>::mutate(|config| config.trade.open = false);
		assert_noop!(
			Sage::remove_asset_price(RuntimeOrigin::signed(ALICE), 13),
			Error::<Test, ()>::TradeClosed,
		);
	});
}

#[test]
fn remove_price_should_reject_unsigned_calls() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(Sage::remove_asset_price(RuntimeOrigin::none(), 13), DispatchError::BadOrigin,);
	});
}

#[test]
fn remove_price_should_reject_incorrect_ownership() {
	ExtBuilder::default()
		.organizer(ALICE)
		.locks(&[(BOB, SEASON_ID_0, Locks::all_unlocked())])
		.build()
		.execute_with(|| {
			let filter = AssetFilter::Trade(VariantType::Player(PlayerType::Human));
			assert_ok!(Sage::update_asset_filter(
				RuntimeOrigin::signed(ALICE),
				SEASON_ID_0,
				filter
			));

			let asset_ids = create_assets::<()>(SEASON_ID_0, BOB, 3);
			let asset_for_sale = asset_ids[0];

			assert_ok!(Sage::set_asset_price(RuntimeOrigin::signed(BOB), asset_for_sale, 123));
			assert_noop!(
				Sage::remove_asset_price(RuntimeOrigin::signed(CHARLIE), asset_for_sale),
				Error::<Test, ()>::AssetNotOwned
			);
		});
}

#[test]
fn remove_price_should_reject_unlisted_asset() {
	ExtBuilder::default().build().execute_with(|| {
		let asset_ids = create_assets::<()>(SEASON_ID_0, BOB, 1);
		assert_noop!(
			Sage::remove_asset_price(RuntimeOrigin::signed(CHARLIE), asset_ids[0]),
			Error::<Test, ()>::AssetNotInTrade,
		);
	});
}
