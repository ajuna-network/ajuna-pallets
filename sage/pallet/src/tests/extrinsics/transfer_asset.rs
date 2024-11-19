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
fn transfer_asset_works() {
	let alice_initial_balance = MockExistentialDeposit::get() * 100;
	ExtBuilder::default()
		.balances(&[(ALICE, alice_initial_balance)])
		.locks(&[
			(ALICE, SEASON_ID_0, Locks::all_unlocked()),
			(BOB, SEASON_ID_0, Locks::all_unlocked()),
			(ALICE, SEASON_ID_1, Locks::all_unlocked()),
			(BOB, SEASON_ID_1, Locks::all_unlocked()),
		])
		.build()
		.execute_with(|| {
			let season_config =
				<Test as Config<Instance1>>::SeasonHandler::get_season_config_for(&SEASON_ID_0)
					.expect("Should get season config");
			let transfer_fee = season_config.fee.transfer_asset;

			let alice_asset_ids = create_assets::<Instance1>(SEASON_ID_0, ALICE, 3);
			let bob_asset_ids = create_assets::<Instance1>(SEASON_ID_1, BOB, 6);
			let asset_id = alice_asset_ids[0];

			assert_ok!(Sage::transfer_asset(RuntimeOrigin::signed(ALICE), BOB, asset_id));
			System::assert_last_event(RuntimeEvent::Sage(Event::AssetTransferred {
				from: ALICE,
				to: BOB,
				asset_id,
			}));

			// Asset transferred from Alice.
			assert_eq!(
				AssetOwners::<Test, Instance1>::iter_prefix(ALICE).count(),
				alice_asset_ids.len() - 1
			);
			let alice_current_assets = {
				let mut assets = AssetOwners::<Test, Instance1>::iter_prefix(ALICE)
					.map(|(asset_id, _)| asset_id)
					.collect::<Vec<_>>();

				assets.sort();
				assets
			};
			let alice_expected_assets = {
				let mut assets = alice_asset_ids[1..].to_vec();
				assets.sort();
				assets
			};
			assert_eq!(alice_current_assets, alice_expected_assets);

			// Asset transferred to Bob.
			assert_eq!(AssetOwners::<Test, Instance1>::iter_prefix(BOB).count(), 7);
			assert_eq!(Assets::<Test, Instance1>::get(asset_id).unwrap().0, BOB);

			let bob_current_assets = {
				let mut assets = AssetOwners::<Test, Instance1>::iter_prefix(BOB)
					.map(|(asset_id, _)| asset_id)
					.collect::<Vec<_>>();

				assets.sort();
				assets
			};
			let bob_expected_assets = {
				let mut assets =
					bob_asset_ids.iter().cloned().chain(vec![asset_id]).collect::<Vec<_>>();
				assets.sort();
				assets
			};
			assert_eq!(bob_current_assets, bob_expected_assets);

			// balance checks
			assert_eq!(Balances::free_balance(ALICE), alice_initial_balance - transfer_fee);

			// Organizer can transfer even when trade is closed.
			GeneralConfigStore::<Test, Instance1>::mutate(|config| config.transfer.open = false);
			Balances::make_free_balance_be(&BOB, transfer_fee + MockExistentialDeposit::get());
			assert_ok!(Sage::set_organizer(RuntimeOrigin::root(), BOB));
			assert_ok!(Sage::transfer_asset(RuntimeOrigin::signed(BOB), CHARLIE, bob_asset_ids[0]));
			assert_eq!(Balances::free_balance(BOB), MockExistentialDeposit::get());
			assert_eq!(
				AssetOwners::<Test, Instance1>::iter_prefix(BOB).count(),
				bob_current_assets.len() - 1
			);
			assert_eq!(AssetOwners::<Test, Instance1>::iter_prefix(CHARLIE).count(), 1);
		});
}

#[test]
fn transfer_asset_rejects_on_transfer_closed() {
	ExtBuilder::default().build().execute_with(|| {
		GeneralConfigStore::<Test, Instance1>::mutate(|config| config.transfer.open = false);
		assert_noop!(
			Sage::transfer_asset(RuntimeOrigin::signed(BOB), CHARLIE, AssetId::random()),
			Error::<Test, Instance1>::TransferClosed
		);
	});
}

#[test]
fn transfer_asset_works_on_transfer_closed_with_organizer() {
	ExtBuilder::default()
		.organizer(BOB)
		.balances(&[(BOB, 1_000)])
		.locks(&[(BOB, SEASON_ID_0, Locks::all_unlocked())])
		.build()
		.execute_with(|| {
			GeneralConfigStore::<Test, Instance1>::mutate(|config| config.transfer.open = false);
			let bob_asset_ids = create_assets::<Instance1>(SEASON_ID_0, BOB, 1);
			let asset_id = bob_asset_ids[0];
			assert_ok!(Sage::transfer_asset(RuntimeOrigin::signed(BOB), DAVE, asset_id));
		});
}

#[test]
fn transfer_asset_rejects_transferring_to_self() {
	ExtBuilder::default().build().execute_with(|| {
		for who in [ALICE, BOB, CHARLIE, DAVE] {
			let asset_ids = create_assets::<Instance1>(SEASON_ID_0, who, 1);
			let asset_id = asset_ids[0];
			assert_noop!(
				Sage::transfer_asset(RuntimeOrigin::signed(who), who, asset_id),
				Error::<Test, Instance1>::CannotTransferToSelf
			);
		}
	});
}

#[test]
fn transfer_asset_rejects_asset_in_trade() {
	ExtBuilder::default()
		.locks(&[(CHARLIE, SEASON_ID_0, Locks::all_unlocked())])
		.build()
		.execute_with(|| {
			let asset_ids = create_assets::<Instance1>(SEASON_ID_0, CHARLIE, 1);
			let asset_id = asset_ids[0];
			assert_ok!(Sage::set_asset_price(RuntimeOrigin::signed(CHARLIE), asset_id, 999));
			assert_noop!(
				Sage::transfer_asset(RuntimeOrigin::signed(CHARLIE), DAVE, asset_id),
				Error::<Test, Instance1>::CannotTransferAssetInTrade
			);
		});
}

#[test]
fn transfer_asset_rejects_unowned_assets() {
	ExtBuilder::default().build().execute_with(|| {
		let asset_id = create_assets::<Instance1>(SEASON_ID_0, CHARLIE, 1)[0];
		assert_noop!(
			Sage::transfer_asset(RuntimeOrigin::signed(ALICE), BOB, asset_id),
			Error::<Test, Instance1>::AssetNotOwned
		);
	});
}

#[test]
fn transfer_asset_rejects_unknown_assets() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Sage::transfer_asset(RuntimeOrigin::signed(ALICE), BOB, AssetId::random()),
			Error::<Test, Instance1>::UnknownAsset
		);
	});
}

#[test]
fn transfer_asset_rejects_on_full_asset_inventory_of_recipient() {
	ExtBuilder::default()
		.balances(&[(ALICE, 1_000)])
		.locks(&[(ALICE, SEASON_ID_0, Locks::all_unlocked())])
		.build()
		.execute_with(|| {
			PlayerSeasonConfigs::<Test, Instance1>::mutate(BOB, SEASON_ID_0, |config| {
				config.inventory_tier = InventoryTier::Three
			});

			let alice_asset_ids = create_assets::<Instance1>(SEASON_ID_0, ALICE, 1);
			let asset_id = alice_asset_ids[0];
			let _ = create_assets::<Instance1>(
				SEASON_ID_0,
				BOB,
				InventoryTier::Three.get_asset_slots(),
			);

			// Trying to send an asset to BOB while his inventory is already full
			assert_noop!(
				Sage::transfer_asset(RuntimeOrigin::signed(ALICE), BOB, asset_id),
				Error::<Test, Instance1>::MaxOwnershipReached
			);
		});
}
