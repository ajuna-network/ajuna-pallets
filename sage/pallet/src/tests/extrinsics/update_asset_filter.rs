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
fn update_asset_filter_should_work_for_trade_filter() {
	ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
		let filter_core = VariantType::Player(PlayerType::Human);
		let filter = AssetFilter::Trade(filter_core);

		assert_eq!(SeasonTradeFilters::<Test, ()>::get(SEASON_ID_0), None);

		assert_ok!(Sage::update_asset_filter(RuntimeOrigin::signed(ALICE), SEASON_ID_0, filter));
		System::assert_last_event(RuntimeEvent::Sage(Event::UpdatedTradeFilter {
			season_id: SEASON_ID_0,
			filter: filter_core,
		}));

		assert_eq!(SeasonTradeFilters::<Test, ()>::get(SEASON_ID_0), Some(filter_core));
	});
}

#[test]
fn update_asset_filter_should_work_for_transfer_filter() {
	ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
		let filter_core = VariantType::Player(PlayerType::Human);
		let filter = AssetFilter::Transfer(filter_core);

		assert_eq!(SeasonTradeFilters::<Test, ()>::get(SEASON_ID_0), None);

		assert_ok!(Sage::update_asset_filter(RuntimeOrigin::signed(ALICE), SEASON_ID_0, filter));
		System::assert_last_event(RuntimeEvent::Sage(Event::UpdatedTransferFilter {
			season_id: SEASON_ID_0,
			filter: filter_core,
		}));

		assert_eq!(SeasonTransferFilters::<Test, ()>::get(SEASON_ID_0), Some(filter_core));
	});
}

#[test]
fn update_asset_filter_should_reject_non_organizer_calls() {
	ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
		assert_noop!(
			Sage::update_asset_filter(
				RuntimeOrigin::signed(BOB),
				SEASON_ID_0,
				AssetFilter::Trade(VariantType::Player(PlayerType::Human)),
			),
			DispatchError::BadOrigin
		);
	});
}
