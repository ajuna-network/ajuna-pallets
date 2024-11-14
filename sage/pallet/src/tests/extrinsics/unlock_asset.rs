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
fn can_unlock_asset_successfully() {
	ExtBuilder::default().balances(&[(ALICE, 1_000)]).build().execute_with(|| {
		let asset_ids = create_assets::<Instance1>(SEASON_ID_0, ALICE, 1);
		let asset_id = asset_ids[0];
		assert_ok!(Sage::lock_asset(RuntimeOrigin::signed(ALICE), asset_id));
		assert_eq!(
			LockedAssets::<Test, Instance1>::get(asset_id),
			Some(Lock { id: *SAGE_LOCK_ID, locker: ALICE })
		);
		assert_ok!(Sage::unlock_asset(RuntimeOrigin::signed(ALICE), asset_id));
		assert_eq!(LockedAssets::<Test, Instance1>::get(asset_id), None);
		System::assert_has_event(RuntimeEvent::Sage(Event::AssetUnlocked { asset_id }));
	});
}

#[test]
fn cannot_unlock_non_owned_asset() {
	ExtBuilder::default()
		.balances(&[(ALICE, 1_000), (BOB, 5_000)])
		.build()
		.execute_with(|| {
			let asset_ids = create_assets::<Instance1>(SEASON_ID_0, BOB, 1);
			let asset_id = asset_ids[0];
			assert_ok!(Sage::lock_asset(RuntimeOrigin::signed(BOB), asset_id));
			assert_noop!(
				Sage::unlock_asset(RuntimeOrigin::signed(ALICE), asset_id),
				Error::<Test, Instance1>::AssetNotOwned
			);
		});
}

#[test]
fn cannot_unlock_asset_locked_by_other_application() {
	ExtBuilder::default().balances(&[(ALICE, 1_000)]).build().execute_with(|| {
		let asset_ids = create_assets::<Instance1>(SEASON_ID_0, ALICE, 1);
		let asset_id = asset_ids[0];

		let other_lock_id = b"otherapp";
		assert_ok!(<Sage as AssetManager>::lock_asset(*other_lock_id, ALICE, asset_id));

		assert_noop!(
			Sage::unlock_asset(RuntimeOrigin::signed(ALICE), asset_id),
			crate::Error::<Test, Instance1>::AssetLockedByOtherApplication
		);
	});
}
