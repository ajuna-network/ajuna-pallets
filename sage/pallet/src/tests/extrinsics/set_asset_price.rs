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
fn set_price_should_work() {
	ExtBuilder::default()
		.organizer(ALICE)
		.locks(&[(BOB, SEASON_ID_0, Locks::all_unlocked())])
		.build()
		.execute_with(|| {
			let filter = AssetFilter::Trade(AssetType::Hero);
			assert_ok!(Sage::update_asset_filter(
				RuntimeOrigin::signed(ALICE),
				SEASON_ID_0,
				filter
			));

			let asset_for_sale = create_assets::<()>(SEASON_ID_0, BOB, 1)[0];
			let price = 7357;

			assert_eq!(AssetTradePrices::<Test, ()>::get(SEASON_ID_0, asset_for_sale), None);
			assert_ok!(Sage::set_asset_price(RuntimeOrigin::signed(BOB), asset_for_sale, price));
			assert_eq!(AssetTradePrices::<Test, ()>::get(SEASON_ID_0, asset_for_sale), Some(price));
			System::assert_last_event(RuntimeEvent::Sage(Event::AssetPriceSet {
				asset_id: asset_for_sale,
				price,
			}));
		});
}

#[test]
fn set_price_should_reject_when_trading_is_closed() {
	ExtBuilder::default().build().execute_with(|| {
		GeneralConfigStore::<Test, ()>::mutate(|config| config.trade.open = false);
		assert_noop!(
			Sage::set_asset_price(RuntimeOrigin::signed(ALICE), 13, 1),
			Error::<Test, ()>::TradeClosed,
		);
	});
}

#[test]
fn set_price_should_reject_unsigned_calls() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(Sage::set_asset_price(RuntimeOrigin::none(), 13, 1), DispatchError::BadOrigin,);
	});
}

#[test]
fn set_price_should_reject_setting_price_to_a_non_owned_asset() {
	ExtBuilder::default().build().execute_with(|| {
		let asset_ids = create_assets::<()>(SEASON_ID_0, BOB, 2);
		let asset_id = asset_ids[0];

		assert_noop!(
			Sage::set_asset_price(RuntimeOrigin::signed(CHARLIE), asset_id, 101),
			Error::<Test, ()>::AssetNotOwned
		);
	});
}

#[test]
fn set_price_should_reject_asset_not_matching_trade_filters() {
	// This test relies on the implementation of `MockFilterHandler` to work
	ExtBuilder::default()
		.organizer(ALICE)
		.locks(&[(BOB, SEASON_ID_0, Locks::all_unlocked())])
		.build()
		.execute_with(|| {
			let trade_filter = AssetType::None;
			assert_ok!(Sage::update_asset_filter(
				RuntimeOrigin::signed(ALICE),
				SEASON_ID_0,
				AssetFilter::Trade(trade_filter)
			));

			let asset_ids = create_assets::<()>(SEASON_ID_0, BOB, 2);
			let asset_id_1 = asset_ids[0];
			let asset_id_2 = asset_ids[1];

			// Since asset_id_1 doest have its type match the filter we cannot set price for it
			let (_, asset_1) = Assets::<Test, ()>::get(asset_id_1).expect("Should get asset");
			match &asset_1.asset_variant {
				HeroJam(hero_jam_asset) => {
					assert_eq!(hero_jam_asset.asset_type, AssetType::Hero);
				},
			}

			assert_noop!(
				Sage::set_asset_price(RuntimeOrigin::signed(BOB), asset_id_1, 101),
				Error::<Test, ()>::AssetCannotBeTraded
			);

			// We change asset_id_2 type so that it matches the filter, allowing us to put it on
			// sale
			Assets::<Test, ()>::mutate(asset_id_2, |maybe_asset| {
				if let Some((_, ref mut asset)) = maybe_asset {
					match asset.asset_variant {
						HeroJam(ref mut hero_jam_asset) => {
							hero_jam_asset.asset_type = trade_filter;
						},
					}
				}
			});
			assert_ok!(Sage::set_asset_price(RuntimeOrigin::signed(BOB), asset_id_2, 101));
		});
}
