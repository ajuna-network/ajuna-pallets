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
use frame_support::traits::fungible::Mutate;

#[test]
fn transfer_asset_works() {
	let alice_initial_balance = MockExistentialDeposit::get() * 100;
	ExtBuilder::default()
		.organizer(ALICE)
		.balances(&[(ALICE, alice_initial_balance)])
		.locks(&[
			(ALICE, SEASON_ID_0, Locks::all_unlocked()),
			(BOB, SEASON_ID_0, Locks::all_unlocked()),
			(ALICE, SEASON_ID_1, Locks::all_unlocked()),
			(BOB, SEASON_ID_1, Locks::all_unlocked()),
		])
		.build()
		.execute_with(|| {
			let filter = AssetFilter::Transfer(AssetType::Hero);
			assert_ok!(Sage::update_asset_filter(
				RuntimeOrigin::signed(ALICE),
				SEASON_ID_0,
				filter
			));
			assert_ok!(Sage::update_asset_filter(
				RuntimeOrigin::signed(ALICE),
				SEASON_ID_1,
				filter
			));

			let season_config =
				<Test as Config<()>>::SeasonHandler::get_season_config_for(&SEASON_ID_0)
					.expect("Should get season config");
			let transfer_fee = season_config.fee.transfer_asset;

			let alice_asset_ids = create_assets::<()>(SEASON_ID_0, ALICE, 3);
			let bob_asset_ids = create_assets::<()>(SEASON_ID_1, BOB, 6);
			let asset_id = alice_asset_ids[0];

			assert_ok!(Sage::transfer_asset(
				RuntimeOrigin::signed(ALICE),
				BOB,
				asset_id,
				SOME_NATIVE_PAYMENT
			));
			System::assert_last_event(RuntimeEvent::Sage(Event::AssetTransferred {
				from: ALICE,
				to: BOB,
				asset_id,
			}));

			// Asset transferred from Alice.
			assert_eq!(
				AssetOwners::<Test, ()>::iter_prefix((ALICE, SEASON_ID_0)).count(),
				alice_asset_ids.len() - 1
			);
			let alice_current_assets = {
				let mut assets = AssetOwners::<Test, ()>::iter_prefix((ALICE, SEASON_ID_0))
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
			assert_eq!(AssetOwners::<Test, ()>::iter_prefix((BOB, SEASON_ID_0)).count(), 1);
			assert_eq!(Assets::<Test, ()>::get(asset_id).unwrap().0, BOB);

			let bob_current_assets_season_0 = {
				let mut assets = AssetOwners::<Test, ()>::iter_prefix((BOB, SEASON_ID_0))
					.map(|(asset_id, _)| asset_id)
					.collect::<Vec<_>>();
				assets.sort();
				assets
			};
			assert_eq!(bob_current_assets_season_0, vec![asset_id]);

			// Bob's original assets are safe.
			assert_eq!(AssetOwners::<Test, ()>::iter_prefix((BOB, SEASON_ID_1)).count(), 6);
			let bob_current_assets_season_1 = {
				let mut assets = AssetOwners::<Test, ()>::iter_prefix((BOB, SEASON_ID_1))
					.map(|(asset_id, _)| asset_id)
					.collect::<Vec<_>>();
				assets.sort();
				assets
			};
			let expected_bob_asset_ids_season_1 = {
				let mut assets = bob_asset_ids.clone();
				assets.sort();
				assets
			};
			assert_eq!(bob_current_assets_season_1, expected_bob_asset_ids_season_1);

			// balance checks
			assert_eq!(Balances::free_balance(ALICE), alice_initial_balance - transfer_fee);

			// Organizer can transfer even when trade is closed.
			GeneralConfigStore::<Test, ()>::mutate(|config| config.transfer.open = false);
			Balances::set_balance(&BOB, transfer_fee + MockExistentialDeposit::get());
			assert_ok!(Sage::set_organizer(RuntimeOrigin::root(), BOB));
			assert_ok!(Sage::transfer_asset(
				RuntimeOrigin::signed(BOB),
				CHARLIE,
				bob_asset_ids[0],
				SOME_NATIVE_PAYMENT
			));
			assert_eq!(Balances::free_balance(BOB), MockExistentialDeposit::get());
			assert_eq!(
				AssetOwners::<Test, ()>::iter_prefix((BOB, SEASON_ID_1)).count(),
				bob_asset_ids.len() - 1
			);
			assert_eq!(AssetOwners::<Test, ()>::iter_prefix((CHARLIE, SEASON_ID_1)).count(), 1);
		});
}

#[test]
fn transfer_asset_rejects_on_transfer_closed() {
	ExtBuilder::default().build().execute_with(|| {
		GeneralConfigStore::<Test, ()>::mutate(|config| config.transfer.open = false);
		assert_noop!(
			Sage::transfer_asset(RuntimeOrigin::signed(BOB), CHARLIE, 13, SOME_NATIVE_PAYMENT),
			Error::<Test, ()>::TransferClosed
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
			let filter = AssetFilter::Transfer(AssetType::Hero);
			assert_ok!(Sage::update_asset_filter(RuntimeOrigin::signed(BOB), SEASON_ID_0, filter));

			GeneralConfigStore::<Test, ()>::mutate(|config| config.transfer.open = false);
			let bob_asset_ids = create_assets::<()>(SEASON_ID_0, BOB, 1);
			let asset_id = bob_asset_ids[0];
			assert_ok!(Sage::transfer_asset(
				RuntimeOrigin::signed(BOB),
				DAVE,
				asset_id,
				SOME_NATIVE_PAYMENT
			));
		});
}

#[test]
fn transfer_asset_rejects_transferring_to_self() {
	ExtBuilder::default().build().execute_with(|| {
		for who in [ALICE, BOB, CHARLIE, DAVE] {
			let asset_ids = create_assets::<()>(SEASON_ID_0, who, 1);
			let asset_id = asset_ids[0];
			assert_noop!(
				Sage::transfer_asset(
					RuntimeOrigin::signed(who),
					who,
					asset_id,
					SOME_NATIVE_PAYMENT
				),
				Error::<Test, ()>::CannotTransferToSelf
			);
		}
	});
}

#[test]
fn transfer_asset_rejects_asset_in_trade() {
	ExtBuilder::default()
		.organizer(ALICE)
		.locks(&[(CHARLIE, SEASON_ID_0, Locks::all_unlocked())])
		.build()
		.execute_with(|| {
			let filter = AssetFilter::Trade(AssetType::Hero);
			assert_ok!(Sage::update_asset_filter(
				RuntimeOrigin::signed(ALICE),
				SEASON_ID_0,
				filter
			));

			let asset_ids = create_assets::<()>(SEASON_ID_0, CHARLIE, 1);
			let asset_id = asset_ids[0];
			assert_ok!(Sage::set_asset_price(RuntimeOrigin::signed(CHARLIE), asset_id, 999));
			assert_noop!(
				Sage::transfer_asset(
					RuntimeOrigin::signed(CHARLIE),
					DAVE,
					asset_id,
					SOME_NATIVE_PAYMENT
				),
				Error::<Test, ()>::CannotTransferAssetInTrade
			);
		});
}

#[test]
fn transfer_asset_rejects_unowned_assets() {
	ExtBuilder::default().build().execute_with(|| {
		let asset_id = create_assets::<()>(SEASON_ID_0, CHARLIE, 1)[0];
		assert_noop!(
			Sage::transfer_asset(RuntimeOrigin::signed(ALICE), BOB, asset_id, SOME_NATIVE_PAYMENT),
			Error::<Test, ()>::AssetNotOwned
		);
	});
}

#[test]
fn transfer_asset_rejects_unknown_assets() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Sage::transfer_asset(RuntimeOrigin::signed(ALICE), BOB, 13, SOME_NATIVE_PAYMENT),
			Error::<Test, ()>::UnknownAsset
		);
	});
}

#[test]
fn transfer_asset_rejects_on_full_asset_inventory_of_recipient() {
	ExtBuilder::default()
		.organizer(ALICE)
		.balances(&[(ALICE, 1_000)])
		.locks(&[(ALICE, SEASON_ID_0, Locks::all_unlocked())])
		.build()
		.execute_with(|| {
			let filter = AssetFilter::Transfer(AssetType::Hero);
			assert_ok!(Sage::update_asset_filter(
				RuntimeOrigin::signed(ALICE),
				SEASON_ID_0,
				filter
			));

			PlayerSeasonConfigs::<Test, ()>::mutate(BOB, SEASON_ID_0, |config| {
				config.inventory_tier = InventoryTier::Three
			});

			let alice_asset_ids = create_assets::<()>(SEASON_ID_0, ALICE, 1);
			let asset_id = alice_asset_ids[0];
			let _ = create_assets::<()>(SEASON_ID_0, BOB, InventoryTier::Three.get_asset_slots());

			// Trying to send an asset to BOB while his inventory is already full
			assert_noop!(
				Sage::transfer_asset(
					RuntimeOrigin::signed(ALICE),
					BOB,
					asset_id,
					SOME_NATIVE_PAYMENT
				),
				Error::<Test, ()>::MaxOwnershipReached
			);
		});
}

#[test]
fn transfer_asset_rejects_asset_not_matching_transfer_filters() {
	// This test relies on the implementation of `MockFilterHandler` to work
	ExtBuilder::default()
		.organizer(ALICE)
		.balances(&[(BOB, 1_000)])
		.locks(&[(BOB, SEASON_ID_0, Locks::all_unlocked())])
		.build()
		.execute_with(|| {
			let transfer_filter = AssetType::None;
			assert_ok!(Sage::update_asset_filter(
				RuntimeOrigin::signed(ALICE),
				SEASON_ID_0,
				AssetFilter::Transfer(transfer_filter)
			));

			let asset_ids = create_assets::<()>(SEASON_ID_0, BOB, 2);
			let asset_id_1 = asset_ids[0];
			let asset_id_2 = asset_ids[1];

			// Since asset_id_1 doest have its type match the filter we cannot set price for it
			let (_, asset_1) = Assets::<Test, ()>::get(asset_id_1).expect("Should get asset");
			match &asset_1.asset_variant {
				AssetVariant::HeroJam(hero_jam_asset) => {
					assert_eq!(hero_jam_asset.asset_type, AssetType::Hero);
				},
			}
			assert_noop!(
				Sage::transfer_asset(
					RuntimeOrigin::signed(BOB),
					ALICE,
					asset_id_1,
					SOME_NATIVE_PAYMENT
				),
				Error::<Test, ()>::AssetCannotBeTransfered
			);

			// We change asset_id_2 type so that it matches the filter, allowing us to put it on
			// sale
			Assets::<Test, ()>::mutate(asset_id_2, |maybe_asset| {
				if let Some((_, ref mut asset)) = maybe_asset {
					match asset.asset_variant {
						AssetVariant::HeroJam(ref mut hero_jam_asset) => {
							hero_jam_asset.asset_type = transfer_filter;
						},
					}
				}
			});
			assert_ok!(Sage::transfer_asset(
				RuntimeOrigin::signed(BOB),
				ALICE,
				asset_id_2,
				SOME_NATIVE_PAYMENT
			));
		});
}
