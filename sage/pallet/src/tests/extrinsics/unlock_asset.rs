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
fn can_unlock_asset_successfully_with_sage_lock_id() {
	ExtBuilder::default().balances(&[(ALICE, 1_000)]).build().execute_with(|| {
		let asset_ids = create_assets::<()>(SEASON_ID_0, ALICE, 1);
		let asset_id = asset_ids[0];
		let expected_lock = Lock { id: *SAGE_LOCK_ID, locker: ALICE };

		assert_ok!(Sage::lock_asset(RuntimeOrigin::signed(ALICE), asset_id));
		assert_eq!(
			LockedAssets::<Test, ()>::get(asset_id),
			Some(Lock { id: *SAGE_LOCK_ID, locker: ALICE })
		);
		assert_ok!(Sage::unlock_asset(RuntimeOrigin::signed(ALICE), asset_id));
		System::assert_has_event(RuntimeEvent::Sage(Event::AssetUnlocked {
			asset_id,
			lock: expected_lock,
		}));
		assert_eq!(LockedAssets::<Test, ()>::get(asset_id), None);
	});
}
