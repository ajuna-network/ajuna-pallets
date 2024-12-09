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

const TEST_LOCK_ID: &[u8; 8] = b"testlock";

mod lock_asset {
	use super::*;

	#[test]
	fn can_lock_asset_successfully() {
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
				let expected_lock = Lock { id: *TEST_LOCK_ID, locker: ALICE };

				assert_ok!(<Sage as AssetManager>::lock_asset(*TEST_LOCK_ID, ALICE, asset_id));
				assert!(<Sage as AssetManager>::is_locked(&asset_id).is_some());
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
			});
	}

	#[test]
	fn cannot_lock_unowned_asset() {
		ExtBuilder::default()
			.balances(&[(ALICE, 1_000), (BOB, 1_000)])
			.build()
			.execute_with(|| {
				let asset_ids = create_assets::<()>(SEASON_ID_0, BOB, 1);
				let asset_id = asset_ids[0];
				assert_noop!(
					<Sage as AssetManager>::lock_asset(*TEST_LOCK_ID, ALICE, asset_id),
					Error::<Test, ()>::AssetNotOwned
				);
			});
	}

	#[test]
	fn cannot_lock_asset_on_trade() {
		ExtBuilder::default()
			.balances(&[(ALICE, 1_000)])
			.locks(&[(CHARLIE, SEASON_ID_0, Locks::all_unlocked())])
			.build()
			.execute_with(|| {
				let asset_ids = create_assets::<()>(SEASON_ID_0, CHARLIE, 1);
				let asset_id = asset_ids[0];
				assert_ok!(Sage::set_asset_price(RuntimeOrigin::signed(CHARLIE), asset_id, 1_000));
				assert_noop!(
					<Sage as AssetManager>::lock_asset(*TEST_LOCK_ID, CHARLIE, asset_id),
					Error::<Test, ()>::CannotLockAssetInTrade
				);
			});
	}

	#[test]
	fn cannot_lock_already_locked_asset() {
		ExtBuilder::default().balances(&[(ALICE, 1_000)]).build().execute_with(|| {
			let asset_ids = create_assets::<()>(SEASON_ID_0, DAVE, 1);
			let asset_id = asset_ids[0];
			assert_ok!(<Sage as AssetManager>::lock_asset(*TEST_LOCK_ID, DAVE, asset_id));
			assert_noop!(
				<Sage as AssetManager>::lock_asset(
					*TEST_LOCK_ID,
					Sage::technical_account_id(),
					asset_id
				),
				Error::<Test, ()>::AssetLocked
			);
		});
	}
}

mod unlock_asset {
	use super::*;

	#[test]
	fn can_unlock_asset_successfully() {
		ExtBuilder::default().balances(&[(ALICE, 1_000)]).build().execute_with(|| {
			let asset_ids = create_assets::<()>(SEASON_ID_0, ALICE, 1);
			let asset_id = asset_ids[0];
			let expected_lock = Lock { id: *TEST_LOCK_ID, locker: ALICE };

			assert_ok!(<Sage as AssetManager>::lock_asset(*TEST_LOCK_ID, ALICE, asset_id));
			assert_eq!(
				LockedAssets::<Test, ()>::get(asset_id),
				Some(Lock { id: *TEST_LOCK_ID, locker: ALICE })
			);
			assert_ok!(<Sage as AssetManager>::unlock_asset(*TEST_LOCK_ID, ALICE, asset_id));
			System::assert_has_event(RuntimeEvent::Sage(Event::AssetLocked {
				asset_id,
				lock: expected_lock,
			}));
			assert_eq!(LockedAssets::<Test, ()>::get(asset_id), None);
		});
	}

	#[test]
	fn cannot_unlock_non_owned_asset() {
		ExtBuilder::default()
			.balances(&[(ALICE, 1_000), (BOB, 5_000)])
			.build()
			.execute_with(|| {
				let asset_ids = create_assets::<()>(SEASON_ID_0, BOB, 1);
				let asset_id = asset_ids[0];
				assert_ok!(<Sage as AssetManager>::lock_asset(*TEST_LOCK_ID, BOB, asset_id));
				assert_noop!(
					<Sage as AssetManager>::unlock_asset(*TEST_LOCK_ID, ALICE, asset_id),
					Error::<Test, ()>::AssetNotOwned
				);
			});
	}

	#[test]
	fn cannot_unlock_asset_locked_by_other_application() {
		ExtBuilder::default().balances(&[(ALICE, 1_000)]).build().execute_with(|| {
			let asset_ids = create_assets::<()>(SEASON_ID_0, ALICE, 1);
			let asset_id = asset_ids[0];

			let other_lock_id = b"otherapp";
			assert_ok!(<Sage as AssetManager>::lock_asset(*other_lock_id, ALICE, asset_id));

			assert_noop!(
				<Sage as AssetManager>::unlock_asset(*TEST_LOCK_ID, ALICE, asset_id),
				Error::<Test, ()>::AssetLockedByOtherApplication
			);
		});
	}
}
