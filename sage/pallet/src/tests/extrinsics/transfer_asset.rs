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

use example_transition::asset::BanditVariant;
use frame_support::traits::fungible::Mutate;
use sage_testing::ExistentialDeposit;

#[test]
fn transfer_asset_works() {
	let alice_initial_balance = ExistentialDeposit::get() * 100;
	ExtBuilder::default()
		.organizer(alice())
		.balances(&[(alice(), alice_initial_balance)])
		.locks(&[
			(alice(), SEASON_ID_0, Locks::all_unlocked()),
			(bob(), SEASON_ID_0, Locks::all_unlocked()),
			(alice(), SEASON_ID_1, Locks::all_unlocked()),
			(bob(), SEASON_ID_1, Locks::all_unlocked()),
		])
		.build()
		.execute_with(|| {
			let filter = AssetFilter::Transfer(VariantType::Player(PlayerType::Human));
			assert_ok!(Sage::update_asset_filter(
				RuntimeOrigin::signed(alice()),
				SEASON_ID_0,
				filter
			));
			assert_ok!(Sage::update_asset_filter(
				RuntimeOrigin::signed(alice()),
				SEASON_ID_1,
				filter
			));

			let season_config =
				<Test as Config<()>>::SeasonHandler::get_season_config_for(&SEASON_ID_0)
					.expect("Should get season config");
			let transfer_fee = season_config.fee.transfer_asset;

			let alice_asset_ids = create_assets::<()>(SEASON_ID_0, alice(), 3);
			let bob_asset_ids = create_assets::<()>(SEASON_ID_1, bob(), 6);
			let asset_id = alice_asset_ids[0];

			assert_ok!(Sage::transfer_asset(
				RuntimeOrigin::signed(alice()),
				bob(),
				asset_id,
				SOME_NATIVE_PAYMENT
			));
			System::assert_last_event(RuntimeEvent::Sage(Event::AssetTransferred {
				from: alice(),
				to: bob(),
				asset_id,
			}));

			// Asset transferred from Alice.
			assert_eq!(
				AssetOwners::<Test, ()>::iter_prefix((alice(), SEASON_ID_0)).count(),
				alice_asset_ids.len() - 1
			);
			let alice_current_assets = {
				let mut assets = AssetOwners::<Test, ()>::iter_prefix((alice(), SEASON_ID_0))
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
			assert_eq!(AssetOwners::<Test, ()>::iter_prefix((bob(), SEASON_ID_0)).count(), 1);
			assert_eq!(Assets::<Test, ()>::get(asset_id).unwrap().0, bob());

			let bob_current_assets_season_0 = {
				let mut assets = AssetOwners::<Test, ()>::iter_prefix((bob(), SEASON_ID_0))
					.map(|(asset_id, _)| asset_id)
					.collect::<Vec<_>>();
				assets.sort();
				assets
			};
			assert_eq!(bob_current_assets_season_0, vec![asset_id]);

			// Bob's original assets are safe.
			assert_eq!(AssetOwners::<Test, ()>::iter_prefix((bob(), SEASON_ID_1)).count(), 6);
			let bob_current_assets_season_1 = {
				let mut assets = AssetOwners::<Test, ()>::iter_prefix((bob(), SEASON_ID_1))
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
			assert_eq!(Balances::free_balance(alice()), alice_initial_balance - transfer_fee);

			// Organizer can transfer even when trade is closed.
			GeneralConfigStore::<Test, ()>::mutate(|config| config.transfer.open = false);
			Balances::set_balance(&bob(), transfer_fee + ExistentialDeposit::get());
			assert_ok!(Sage::set_organizer(RuntimeOrigin::root(), bob()));
			assert_ok!(Sage::transfer_asset(
				RuntimeOrigin::signed(bob()),
				charlie(),
				bob_asset_ids[0],
				SOME_NATIVE_PAYMENT
			));
			assert_eq!(Balances::free_balance(bob()), ExistentialDeposit::get());
			assert_eq!(
				AssetOwners::<Test, ()>::iter_prefix((bob(), SEASON_ID_1)).count(),
				bob_asset_ids.len() - 1
			);
			assert_eq!(AssetOwners::<Test, ()>::iter_prefix((charlie(), SEASON_ID_1)).count(), 1);
		});
}

#[test]
fn transfer_asset_rejects_on_transfer_closed() {
	ExtBuilder::default().build().execute_with(|| {
		GeneralConfigStore::<Test, ()>::mutate(|config| config.transfer.open = false);
		assert_noop!(
			Sage::transfer_asset(RuntimeOrigin::signed(bob()), charlie(), 13, SOME_NATIVE_PAYMENT),
			Error::<Test, ()>::TransferClosed
		);
	});
}

#[test]
fn transfer_asset_works_on_transfer_closed_with_organizer() {
	ExtBuilder::default()
		.organizer(bob())
		.balances(&[(bob(), 1_000)])
		.locks(&[(bob(), SEASON_ID_0, Locks::all_unlocked())])
		.build()
		.execute_with(|| {
			let filter = AssetFilter::Transfer(VariantType::Player(PlayerType::Human));
			assert_ok!(Sage::update_asset_filter(
				RuntimeOrigin::signed(bob()),
				SEASON_ID_0,
				filter
			));

			GeneralConfigStore::<Test, ()>::mutate(|config| config.transfer.open = false);
			let bob_asset_ids = create_assets::<()>(SEASON_ID_0, bob(), 1);
			let asset_id = bob_asset_ids[0];
			assert_ok!(Sage::transfer_asset(
				RuntimeOrigin::signed(bob()),
				dave(),
				asset_id,
				SOME_NATIVE_PAYMENT
			));
		});
}

#[test]
fn transfer_asset_rejects_transferring_to_self() {
	ExtBuilder::default().build().execute_with(|| {
		for who in [alice(), bob(), charlie(), dave()] {
			let asset_ids = create_assets::<()>(SEASON_ID_0, who.clone(), 1);
			let asset_id = asset_ids[0];
			assert_noop!(
				Sage::transfer_asset(
					RuntimeOrigin::signed(who.clone()),
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
		.organizer(alice())
		.locks(&[(charlie(), SEASON_ID_0, Locks::all_unlocked())])
		.build()
		.execute_with(|| {
			let filter = AssetFilter::Trade(VariantType::Player(PlayerType::Human));
			assert_ok!(Sage::update_asset_filter(
				RuntimeOrigin::signed(alice()),
				SEASON_ID_0,
				filter
			));

			let asset_ids = create_assets::<()>(SEASON_ID_0, charlie(), 1);
			let asset_id = asset_ids[0];
			assert_ok!(Sage::set_asset_price(RuntimeOrigin::signed(charlie()), asset_id, 999));
			assert_noop!(
				Sage::transfer_asset(
					RuntimeOrigin::signed(charlie()),
					dave(),
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
		let asset_id = create_assets::<()>(SEASON_ID_0, charlie(), 1)[0];
		assert_noop!(
			Sage::transfer_asset(
				RuntimeOrigin::signed(alice()),
				bob(),
				asset_id,
				SOME_NATIVE_PAYMENT
			),
			Error::<Test, ()>::AssetNotOwned
		);
	});
}

#[test]
fn transfer_asset_rejects_unknown_assets() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Sage::transfer_asset(RuntimeOrigin::signed(alice()), bob(), 13, SOME_NATIVE_PAYMENT),
			Error::<Test, ()>::UnknownAsset
		);
	});
}

#[test]
fn transfer_asset_rejects_on_full_asset_inventory_of_recipient() {
	ExtBuilder::default()
		.organizer(alice())
		.balances(&[(alice(), 1_000)])
		.locks(&[(alice(), SEASON_ID_0, Locks::all_unlocked())])
		.build()
		.execute_with(|| {
			let filter = AssetFilter::Transfer(VariantType::Player(PlayerType::Human));
			assert_ok!(Sage::update_asset_filter(
				RuntimeOrigin::signed(alice()),
				SEASON_ID_0,
				filter
			));

			PlayerSeasonConfigs::<Test, ()>::mutate(bob(), SEASON_ID_0, |config| {
				config.inventory_tier = InventoryTier::Three
			});

			let alice_asset_ids = create_assets::<()>(SEASON_ID_0, alice(), 1);
			let asset_id = alice_asset_ids[0];
			let _ = create_assets::<()>(SEASON_ID_0, bob(), InventoryTier::Three.get_asset_slots());

			// Trying to send an asset to BOB while his inventory is already full
			assert_noop!(
				Sage::transfer_asset(
					RuntimeOrigin::signed(alice()),
					bob(),
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
		.organizer(alice())
		.balances(&[(bob(), 1_000)])
		.locks(&[(bob(), SEASON_ID_0, Locks::all_unlocked())])
		.build()
		.execute_with(|| {
			let filter = AssetFilter::Transfer(VariantType::Machine(MachineType::Bandit));
			assert_ok!(Sage::update_asset_filter(
				RuntimeOrigin::signed(alice()),
				SEASON_ID_0,
				filter,
			));

			let asset_ids = create_assets::<()>(SEASON_ID_0, bob(), 2);
			let asset_id_1 = asset_ids[0];
			let asset_id_2 = asset_ids[1];

			// Since asset_id_1 doest have its type match the filter we cannot set price for it
			let (_, asset_1) = Assets::<Test, ()>::get(asset_id_1).expect("Should get asset");
			assert!(
				asset_1.variant.is_variant(VariantType::Player(PlayerType::Human)),
				"Should be Player variant"
			);
			assert_noop!(
				Sage::transfer_asset(
					RuntimeOrigin::signed(bob()),
					alice(),
					asset_id_1,
					SOME_NATIVE_PAYMENT
				),
				Error::<Test, ()>::AssetCannotBeTransferred
			);

			// We change asset_id_2 type so that it matches the filter, allowing us to put it on
			// sale
			Assets::<Test, ()>::mutate(asset_id_2, |maybe_asset| {
				if let Some((_, ref mut asset)) = maybe_asset {
					asset.variant = AssetVariant::Machine(MachineVariant {
						seat_linked: 0,
						seat_limit: 0,
						value_1_factor: TokenType::T1,
						value_1_mul: MultiplierType::V1,
						value_2_factor: TokenType::T1,
						value_2_mul: MultiplierType::V1,
						value_3_factor: TokenType::T1,
						value_3_mul: MultiplierType::V1,
						sub_variant: MachineSubVariant::Bandit(BanditVariant {
							max_spins: 0,
							jackpot: 0,
						}),
					});
				}
			});
			assert_ok!(Sage::transfer_asset(
				RuntimeOrigin::signed(bob()),
				alice(),
				asset_id_2,
				SOME_NATIVE_PAYMENT
			));
		});
}
