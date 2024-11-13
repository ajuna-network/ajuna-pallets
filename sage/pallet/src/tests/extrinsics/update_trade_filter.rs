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

// TODO: Add more tests

#[test]
fn update_trade_filter_should_work() {
	ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
		let filter = MockTradeFilter::from(13);

		assert_eq!(SeasonTradeFilters::<Test, Instance1>::get(SEASON_ID_0), 0);

		assert_ok!(Sage::update_trade_filter(RuntimeOrigin::signed(ALICE), SEASON_ID_0, filter,));
		System::assert_last_event(RuntimeEvent::Sage(Event::UpdatedTradeFilter {
			season_id: SEASON_ID_0,
			filter,
		}));

		assert_eq!(SeasonTradeFilters::<Test, Instance1>::get(SEASON_ID_0), filter);
	});
}

#[test]
fn update_trade_filter_should_reject_non_organizer_calls() {
	ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
		assert_noop!(
			Sage::update_trade_filter(
				RuntimeOrigin::signed(BOB),
				SEASON_ID_0,
				MockTradeFilter::from(22),
			),
			DispatchError::BadOrigin
		);
	});
}
