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
use example_transition::transition::{hero_jam::Action, TransitionIdentifier};

#[test]
fn can_lock_asset_successfully_with_sage_lock_id() {
	ExtBuilder::default()
		.balances(&[(ALICE, 1_000_000)])
		.locks(&[
			(ALICE, SEASON_ID_0, Locks::all_unlocked()),
			(BOB, SEASON_ID_0, Locks::all_unlocked()),
			(Sage::technical_account_id(), SEASON_ID_0, Locks::all_unlocked()),
		])
		.build()
		.execute_with(|| {
			let asset_ids = create_assets::<()>(SEASON_ID_0, ALICE, 1);
			let asset_id = asset_ids[0];
			let expected_lock = Lock { id: *SAGE_LOCK_ID, locker: ALICE };

			assert_ok!(Sage::lock_asset(RuntimeOrigin::signed(ALICE), asset_id));
			assert!(LockedAssets::<Test, ()>::contains_key(asset_id));
			System::assert_has_event(RuntimeEvent::Sage(Event::AssetLocked {
				asset_id,
				lock: expected_lock,
			}));

			// Ensure ownership transferred to technical account
			let technical_account = Sage::technical_account_id();

			assert!(!AssetOwners::<Test, ()>::contains_key((ALICE, SEASON_ID_0, asset_id)));
			assert!(!AssetOwners::<Test, ()>::contains_key((
				technical_account,
				SEASON_ID_0,
				asset_id
			)));
			assert_eq!(Assets::<Test, ()>::get(asset_id).unwrap().0, technical_account);

			// Ensure locked assets cannot be used in trading, transferring and forging
			for extrinsic in [
				Sage::set_asset_price(RuntimeOrigin::signed(technical_account), asset_id, 1_000),
				Sage::transfer_asset(
					RuntimeOrigin::signed(technical_account),
					BOB,
					asset_id,
					SOME_NATIVE_PAYMENT,
				),
				Sage::state_transition(
					RuntimeOrigin::signed(technical_account),
					TransitionIdentifier::HeroJam(Action::CreateHero),
					vec![asset_id],
					(),
					SOME_NATIVE_PAYMENT,
				),
			] {
				assert_noop!(extrinsic, Error::<Test, ()>::AssetLocked);
			}
		});
}
