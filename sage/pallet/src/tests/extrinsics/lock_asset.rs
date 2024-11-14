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
use example_transition::types::ExampleTransitionId;

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
			let asset_ids = create_assets::<Instance1>(SEASON_ID_0, ALICE, 1);
			let asset_id = asset_ids[0];

			assert_ok!(Sage::lock_asset(RuntimeOrigin::signed(ALICE), asset_id));
			assert!(LockedAssets::<Test, Instance1>::contains_key(asset_id));
			System::assert_has_event(RuntimeEvent::Sage(Event::AssetLocked { asset_id }));

			// Ensure ownership transferred to technical account
			let technical_account = Sage::technical_account_id();

			assert!(!AssetOwners::<Test, Instance1>::get(ALICE, SEASON_ID_0).contains(&asset_id));
			assert!(AssetOwners::<Test, Instance1>::get(technical_account, SEASON_ID_0).is_empty());
			assert_eq!(Assets::<Test, Instance1>::get(asset_id).unwrap().0, technical_account);

			// Ensure locked assets cannot be used in trading, transferring and forging
			for extrinsic in [
				Sage::set_asset_price(RuntimeOrigin::signed(technical_account), asset_id, 1_000),
				Sage::transfer_asset(RuntimeOrigin::signed(technical_account), BOB, asset_id),
				Sage::state_transition(
					RuntimeOrigin::signed(technical_account),
					ExampleTransitionId::UpgradeAsset,
					vec![asset_id],
					(),
				),
			] {
				assert_noop!(extrinsic, Error::<Test, Instance1>::AssetLocked);
			}
		});
}

#[test]
fn cannot_lock_unowned_asset() {
	ExtBuilder::default()
		.balances(&[(ALICE, 1_000), (BOB, 1_000)])
		.build()
		.execute_with(|| {
			let asset_ids = create_assets::<Instance1>(SEASON_ID_0, BOB, 1);
			let asset_id = asset_ids[0];
			assert_noop!(
				Sage::lock_asset(RuntimeOrigin::signed(ALICE), asset_id),
				Error::<Test, Instance1>::AssetNotOwned
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
			let asset_ids = create_assets::<Instance1>(SEASON_ID_0, CHARLIE, 1);
			let asset_id = asset_ids[0];
			assert_ok!(Sage::set_asset_price(RuntimeOrigin::signed(CHARLIE), asset_id, 1_000));
			assert_noop!(
				Sage::lock_asset(RuntimeOrigin::signed(CHARLIE), asset_id),
				Error::<Test, Instance1>::CannotLockAssetInTrade
			);
		});
}

#[test]
fn cannot_lock_already_locked_asset() {
	ExtBuilder::default().balances(&[(ALICE, 1_000)]).build().execute_with(|| {
		let asset_ids = create_assets::<Instance1>(SEASON_ID_0, DAVE, 1);
		let asset_id = asset_ids[0];
		assert_ok!(Sage::lock_asset(RuntimeOrigin::signed(DAVE), asset_id));
		assert_noop!(
			Sage::lock_asset(RuntimeOrigin::signed(Sage::technical_account_id()), asset_id),
			Error::<Test, Instance1>::AssetLocked
		);
	});
}
