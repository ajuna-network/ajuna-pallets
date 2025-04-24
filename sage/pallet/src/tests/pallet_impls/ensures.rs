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
			assert_eq!(Organizer::<Test, ()>::get(), None);
			assert_ok!(Sage::set_organizer(RuntimeOrigin::root(), dave()));
			assert_eq!(Organizer::<Test, ()>::get(), Some(dave()));
			assert_ok!(Sage::ensure_organizer(&dave()));
		});
	}

	#[test]
	fn ensure_organizer_should_reject_when_no_organizer_is_set() {
		ExtBuilder::default().build().execute_with(|| {
			assert_eq!(Organizer::<Test, ()>::get(), None);
			assert_noop!(Sage::ensure_organizer(&dave()), Error::<Test, ()>::OrganizerNotSet);
		});
	}

	#[test]
	fn ensure_organizer_should_reject_non_organizer_calls() {
		ExtBuilder::default().organizer(alice()).build().execute_with(|| {
			assert_noop!(Sage::ensure_organizer(&dave()), DispatchError::BadOrigin);
		});
	}

	#[test]
	fn ensure_organizer_should_validate_newly_set_organizer() {
		ExtBuilder::default().organizer(charlie()).build().execute_with(|| {
			assert_ok!(Sage::ensure_organizer(&charlie()));
		});
	}
}

mod ensure_ownership {
	use super::*;

	#[test]
	fn ensure_ownership_works() {
		ExtBuilder::default().build().execute_with(|| {
			let asset_ids = create_assets::<()>(SEASON_ID_0, bob(), 1);
			let asset_id = asset_ids[0];
			assert_ok!(Sage::ensure_ownership(&bob(), &asset_id));
		});
	}

	#[test]
	fn ensure_ownership_should_reject_non_owned_asset() {
		ExtBuilder::default().build().execute_with(|| {
			let asset_ids = create_assets::<()>(SEASON_ID_0, bob(), 1);
			let asset_id = asset_ids[0];
			assert_noop!(
				Sage::ensure_ownership(&dave(), &asset_id),
				Error::<Test, ()>::AssetNotOwned
			);
		});
	}

	#[test]
	fn ensure_ownership_should_reject_unknown_asset() {
		ExtBuilder::default().build().execute_with(|| {
			assert_noop!(Sage::ensure_ownership(&dave(), &13), Error::<Test, ()>::UnknownAsset);
		});
	}
}

mod ensure_for_trade {
	use super::*;

	#[test]
	fn ensure_for_trade_works() {
		ExtBuilder::default().build().execute_with(|| {
			let asset_ids = create_assets::<()>(SEASON_ID_0, bob(), 1);
			let asset_id = asset_ids[0];
			let season_id = <Test as Config<()>>::SeasonHandler::get_current_season_id()
				.expect("Should get season_id");

			AssetTradePrices::<Test, ()>::insert(season_id, asset_id, 100);
			assert_ok!(Sage::ensure_for_trade(&asset_id));
		});
	}

	#[test]
	fn ensure_for_trade_should_reject_unknown_asset() {
		ExtBuilder::default().build().execute_with(|| {
			assert_noop!(Sage::ensure_for_trade(&13), Error::<Test, ()>::UnknownAsset);
		});
	}

	#[test]
	fn ensure_for_trade_should_reject_asset_not_in_trade() {
		ExtBuilder::default().build().execute_with(|| {
			let asset_ids = create_assets::<()>(SEASON_ID_0, bob(), 1);
			let asset_id = asset_ids[0];

			assert_noop!(Sage::ensure_for_trade(&asset_id), Error::<Test, ()>::AssetNotInTrade);
		});
	}
}
