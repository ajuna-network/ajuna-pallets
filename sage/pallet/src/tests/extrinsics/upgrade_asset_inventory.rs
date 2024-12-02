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
fn upgrade_asset_inventory_should_work() {
	let alice_initial_balance = MockExistentialDeposit::get() * 10;
	ExtBuilder::default()
		.balances(&[(ALICE, alice_initial_balance)])
		.build()
		.execute_with(|| {
			let season_config =
				<Test as Config<()>>::SeasonHandler::get_season_config_for(&SEASON_ID_0)
					.expect("Should get season config");
			let upgrade_fee = season_config.fee.upgrade_asset_inventory;

			assert_eq!(
				PlayerSeasonConfigs::<Test, ()>::get(ALICE, SEASON_ID_0).inventory_tier,
				InventoryTier::One
			);
			assert_eq!(
				PlayerSeasonConfigs::<Test, ()>::get(ALICE, SEASON_ID_0)
					.inventory_tier
					.get_asset_slots(),
				25
			);
			assert_eq!(Balances::free_balance(ALICE), alice_initial_balance);

			assert_ok!(Sage::upgrade_asset_inventory(RuntimeOrigin::signed(ALICE), None, None));
			System::assert_last_event(RuntimeEvent::Sage(Event::InventoryTierUpgraded {
				account: ALICE,
				season_id: SEASON_ID_0,
				new_tier: InventoryTier::Two,
			}));

			assert_eq!(
				PlayerSeasonConfigs::<Test, ()>::get(ALICE, SEASON_ID_0).inventory_tier,
				InventoryTier::Two
			);
			assert_eq!(
				PlayerSeasonConfigs::<Test, ()>::get(ALICE, SEASON_ID_0)
					.inventory_tier
					.get_asset_slots(),
				50
			);
			assert_eq!(Balances::free_balance(ALICE), alice_initial_balance - upgrade_fee);

			assert_ok!(Sage::upgrade_asset_inventory(RuntimeOrigin::signed(ALICE), None, None));
			System::assert_last_event(RuntimeEvent::Sage(Event::InventoryTierUpgraded {
				account: ALICE,
				season_id: SEASON_ID_0,
				new_tier: InventoryTier::Three,
			}));

			assert_eq!(
				PlayerSeasonConfigs::<Test, ()>::get(ALICE, SEASON_ID_0).inventory_tier,
				InventoryTier::Three
			);
			assert_eq!(
				PlayerSeasonConfigs::<Test, ()>::get(ALICE, SEASON_ID_0)
					.inventory_tier
					.get_asset_slots(),
				75
			);
			assert_eq!(Balances::free_balance(ALICE), alice_initial_balance - (upgrade_fee * 2));

			assert_ok!(Sage::upgrade_asset_inventory(RuntimeOrigin::signed(ALICE), None, None));
			System::assert_last_event(RuntimeEvent::Sage(Event::InventoryTierUpgraded {
				account: ALICE,
				season_id: SEASON_ID_0,
				new_tier: InventoryTier::Four,
			}));

			assert_eq!(
				PlayerSeasonConfigs::<Test, ()>::get(ALICE, SEASON_ID_0).inventory_tier,
				InventoryTier::Four
			);
			assert_eq!(
				PlayerSeasonConfigs::<Test, ()>::get(ALICE, SEASON_ID_0)
					.inventory_tier
					.get_asset_slots(),
				100
			);
			assert_eq!(Balances::free_balance(ALICE), alice_initial_balance - (upgrade_fee * 3));

			assert_ok!(Sage::upgrade_asset_inventory(RuntimeOrigin::signed(ALICE), None, None));
			System::assert_last_event(RuntimeEvent::Sage(Event::InventoryTierUpgraded {
				account: ALICE,
				season_id: SEASON_ID_0,
				new_tier: InventoryTier::Five,
			}));

			assert_eq!(
				PlayerSeasonConfigs::<Test, ()>::get(ALICE, SEASON_ID_0).inventory_tier,
				InventoryTier::Five
			);
			assert_eq!(
				PlayerSeasonConfigs::<Test, ()>::get(ALICE, SEASON_ID_0)
					.inventory_tier
					.get_asset_slots(),
				150
			);
			assert_eq!(Balances::free_balance(ALICE), alice_initial_balance - (upgrade_fee * 4));

			assert_ok!(Sage::upgrade_asset_inventory(RuntimeOrigin::signed(ALICE), None, None));
			System::assert_last_event(RuntimeEvent::Sage(Event::InventoryTierUpgraded {
				account: ALICE,
				season_id: SEASON_ID_0,
				new_tier: InventoryTier::Max,
			}));

			assert_eq!(
				PlayerSeasonConfigs::<Test, ()>::get(ALICE, SEASON_ID_0).inventory_tier,
				InventoryTier::Max
			);
			assert_eq!(
				PlayerSeasonConfigs::<Test, ()>::get(ALICE, SEASON_ID_0)
					.inventory_tier
					.get_asset_slots(),
				200
			);
			assert_eq!(Balances::free_balance(ALICE), alice_initial_balance - (upgrade_fee * 5));
		});
}

#[test]
fn upgrade_asset_inventory_should_work_on_different_beneficiary() {
	let alice_initial_balance = MockExistentialDeposit::get() * 10;
	ExtBuilder::default()
		.balances(&[(ALICE, alice_initial_balance)])
		.build()
		.execute_with(|| {
			assert_eq!(
				PlayerSeasonConfigs::<Test, ()>::get(ALICE, SEASON_ID_0).inventory_tier,
				InventoryTier::One
			);
			assert_eq!(
				PlayerSeasonConfigs::<Test, ()>::get(BOB, SEASON_ID_0).inventory_tier,
				InventoryTier::One
			);

			assert_ok!(Sage::upgrade_asset_inventory(
				RuntimeOrigin::signed(ALICE),
				Some(BOB),
				None
			));
			System::assert_last_event(RuntimeEvent::Sage(Event::InventoryTierUpgraded {
				account: BOB,
				season_id: SEASON_ID_0,
				new_tier: InventoryTier::Two,
			}));

			assert_eq!(
				PlayerSeasonConfigs::<Test, ()>::get(ALICE, SEASON_ID_0).inventory_tier,
				InventoryTier::One
			);
			assert_eq!(
				PlayerSeasonConfigs::<Test, ()>::get(BOB, SEASON_ID_0).inventory_tier,
				InventoryTier::Two
			);
		});
}

#[test]
fn upgrade_asset_inventory_should_work_on_different_season() {
	let alice_initial_balance = MockExistentialDeposit::get() * 10;
	ExtBuilder::default()
		.balances(&[(ALICE, alice_initial_balance)])
		.build()
		.execute_with(|| {
			assert_eq!(
				PlayerSeasonConfigs::<Test, ()>::get(ALICE, SEASON_ID_0).inventory_tier,
				InventoryTier::One
			);
			assert_eq!(
				PlayerSeasonConfigs::<Test, ()>::get(ALICE, SEASON_ID_1).inventory_tier,
				InventoryTier::One
			);

			assert_ok!(Sage::upgrade_asset_inventory(
				RuntimeOrigin::signed(ALICE),
				None,
				Some(SEASON_ID_1)
			));

			assert_eq!(
				PlayerSeasonConfigs::<Test, ()>::get(ALICE, SEASON_ID_0).inventory_tier,
				InventoryTier::One
			);
			assert_eq!(
				PlayerSeasonConfigs::<Test, ()>::get(ALICE, SEASON_ID_1).inventory_tier,
				InventoryTier::Two
			);
		});
}

#[test]
fn upgrade_asset_inventory_should_reject_insufficient_balance() {
	ExtBuilder::default().build().execute_with(|| {
		assert_noop!(
			Sage::upgrade_asset_inventory(RuntimeOrigin::signed(ALICE), None, None),
			sp_runtime::TokenError::FundsUnavailable,
		);
	});
}

#[test]
fn upgrade_asset_inventory_should_reject_fully_upgraded_storage() {
	let alice_initial_balance = MockExistentialDeposit::get() * 10;

	ExtBuilder::default()
		.balances(&[(ALICE, alice_initial_balance)])
		.build()
		.execute_with(|| {
			PlayerSeasonConfigs::<Test, ()>::mutate(ALICE, SEASON_ID_0, |config| {
				config.inventory_tier = InventoryTier::Max
			});

			assert_noop!(
				Sage::upgrade_asset_inventory(RuntimeOrigin::signed(ALICE), None, None),
				Error::<Test, ()>::MaxStorageTierReached
			);
		});
}
