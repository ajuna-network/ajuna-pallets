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

mod ensure_organizer {
	use super::*;

	#[test]
	fn ensure_organizer_should_works() {
		ExtBuilder::default().build().execute_with(|| {
			assert_eq!(Organizer::<Test, Instance1>::get(), None);
			assert_ok!(Sage::set_organizer(RuntimeOrigin::root(), DAVE));
			assert_eq!(Organizer::<Test, Instance1>::get(), Some(DAVE));
			assert_ok!(Sage::ensure_organizer(RuntimeOrigin::signed(DAVE)));
		});
	}

	#[test]
	fn ensure_organizer_should_reject_when_no_organizer_is_set() {
		ExtBuilder::default().build().execute_with(|| {
			assert_eq!(Organizer::<Test, Instance1>::get(), None);
			assert_noop!(
				Sage::ensure_organizer(RuntimeOrigin::signed(DAVE)),
				Error::<Test, Instance1>::OrganizerNotSet
			);
		});
	}

	#[test]
	fn ensure_organizer_should_reject_non_organizer_calls() {
		ExtBuilder::default().organizer(ALICE).build().execute_with(|| {
			assert_noop!(
				Sage::ensure_organizer(RuntimeOrigin::signed(DAVE)),
				DispatchError::BadOrigin
			);
		});
	}

	#[test]
	fn ensure_organizer_should_validate_newly_set_organizer() {
		ExtBuilder::default().organizer(CHARLIE).build().execute_with(|| {
			assert_ok!(Sage::ensure_organizer(RuntimeOrigin::signed(CHARLIE)));
		});
	}
}

mod ensure_ownership {
	use super::*;

	#[test]
	fn ensure_ownership_works() {
		ExtBuilder::default().build().execute_with(|| {
			let asset_ids = create_assets::<Instance1>(SEASON_ID_0, BOB, 1);
			let asset_id = asset_ids[0];
			assert_ok!(Sage::ensure_ownership(&BOB, &asset_id));
		});
	}

	#[test]
	fn ensure_ownership_should_reject_non_owned_asset() {
		ExtBuilder::default().build().execute_with(|| {
			let asset_ids = create_assets::<Instance1>(SEASON_ID_0, BOB, 1);
			let asset_id = asset_ids[0];
			assert_noop!(
				Sage::ensure_ownership(&DAVE, &asset_id),
				Error::<Test, Instance1>::AssetNotOwned
			);
		});
	}

	#[test]
	fn ensure_ownership_should_reject_unknown_asset() {
		ExtBuilder::default().build().execute_with(|| {
			let asset_id = AssetId::random();
			assert_noop!(
				Sage::ensure_ownership(&DAVE, &asset_id),
				Error::<Test, Instance1>::UnknownAsset
			);
		});
	}
}

mod ensure_for_trade {
	use super::*;

	#[test]
	fn ensure_for_trade_works() {
		ExtBuilder::default().build().execute_with(|| {
			let asset_ids = create_assets::<Instance1>(SEASON_ID_0, BOB, 1);
			let asset_id = asset_ids[0];
			let season_id = <Test as Config<Instance1>>::SeasonHandler::get_current_season_id();

			AssetTradePrices::<Test, Instance1>::insert(season_id, asset_id, 100);
			assert_ok!(Sage::ensure_for_trade(&asset_id));
		});
	}

	#[test]
	fn ensure_for_trade_should_reject_unknown_asset() {
		ExtBuilder::default().build().execute_with(|| {
			let asset_id = AssetId::random();
			assert_noop!(Sage::ensure_for_trade(&asset_id), Error::<Test, Instance1>::UnknownAsset);
		});
	}

	#[test]
	fn ensure_for_trade_should_reject_asset_not_in_trade() {
		ExtBuilder::default().build().execute_with(|| {
			let asset_ids = create_assets::<Instance1>(SEASON_ID_0, BOB, 1);
			let asset_id = asset_ids[0];

			assert_noop!(
				Sage::ensure_for_trade(&asset_id),
				Error::<Test, Instance1>::AssetNotInTrade
			);
		});
	}
}
