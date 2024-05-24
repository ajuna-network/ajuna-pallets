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
use crate::{mock::*, types::*, *};
use frame_support::{assert_noop, assert_ok};

mod organizer {
	use super::*;

	#[test]
	fn set_organizer_should_work() {
		ExtBuilder::default().build().execute_with(|| {
			assert_eq!(Organizer::<Test>::get(), None);
			assert_ok!(Omega::set_organizer(RuntimeOrigin::root(), CHARLIE));
			assert_eq!(Organizer::<Test>::get(), Some(CHARLIE));
			System::assert_last_event(mock::RuntimeEvent::Omega(crate::Event::OrganizerSet {
				organizer: CHARLIE,
			}));
		});
	}

	#[test]
	fn set_organizer_should_reject_non_root_calls() {
		ExtBuilder::default().build().execute_with(|| {
			assert_noop!(
				Omega::set_organizer(RuntimeOrigin::signed(ALICE), BOB),
				DispatchError::BadOrigin
			);
		});
	}

	#[test]
	fn set_organizer_should_replace_existing_organizer() {
		ExtBuilder::default().build().execute_with(|| {
			assert_ok!(Omega::set_organizer(RuntimeOrigin::root(), CHARLIE));
			assert_eq!(Organizer::<Test>::get(), Some(CHARLIE));
			assert_ok!(Omega::set_organizer(RuntimeOrigin::root(), DAVE));
			assert_eq!(Organizer::<Test>::get(), Some(DAVE));
			System::assert_last_event(mock::RuntimeEvent::Omega(crate::Event::OrganizerSet {
				organizer: DAVE,
			}));
		});
	}

	#[test]
	fn ensure_organizer_should_reject_when_no_organizer_is_set() {
		ExtBuilder::default().build().execute_with(|| {
			assert_eq!(Organizer::<Test>::get(), None);
			assert_noop!(
				Omega::ensure_organizer(RuntimeOrigin::signed(DAVE)),
				Error::<Test>::OrganizerNotSet
			);
		});
	}

	#[test]
	fn ensure_organizer_should_reject_non_organizer_calls() {
		ExtBuilder::default().build().execute_with(|| {
			assert_ok!(Omega::set_organizer(RuntimeOrigin::root(), CHARLIE));
			assert_noop!(
				Omega::ensure_organizer(RuntimeOrigin::signed(DAVE)),
				DispatchError::BadOrigin
			);
		});
	}
}

mod buy_loot_crate {
	use super::*;

	#[test]
	fn buy_loot_crate_should_work() {
		ExtBuilder::default().build().execute_with(|| {
			assert_eq!(Commanders::<Test>::iter_key_prefix(ALICE).count(), 0);

			assert_ok!(Omega::buy_loot_crate(RuntimeOrigin::signed(ALICE)));

			let mut commanders = Commanders::<Test>::iter_key_prefix(ALICE).collect::<Vec<_>>();
			assert_eq!(commanders.len(), 1);

			System::assert_last_event(mock::RuntimeEvent::Omega(crate::Event::CommanderRolled {
				account: ALICE,
				commander_id: commanders.pop().expect("Should contain commander_id"),
			}));
		});
	}
}

mod add_ship {
	use super::*;

	#[test]
	fn add_ship_should_work() {
		let initial_balance = 100;
		ExtBuilder::default()
			.balances(&[(ALICE, initial_balance)])
			.build()
			.execute_with(|| {
				let ship = Ship::default();

				assert_eq!(Ships::<Test>::iter_key_prefix(ALICE).count(), 0);

				assert_eq!(Balances::free_balance(ALICE), initial_balance);

				assert_ok!(Omega::add_ship(RuntimeOrigin::signed(ALICE), ship));

				assert_eq!(Balances::free_balance(ALICE), initial_balance - 10);

				let mut ships = Ships::<Test>::iter_key_prefix(ALICE).collect::<Vec<_>>();
				assert_eq!(ships.len(), 1);

				let ship_id = ships.pop().expect("Should contain ship");
				let added_ship = Ships::<Test>::get(ALICE, ship_id).expect("Should contain ship");
				assert_eq!(added_ship, ship);

				System::assert_last_event(mock::RuntimeEvent::Omega(
					crate::Event::ShipRegistered { account: ALICE, ship_id, ship: added_ship },
				));

				assert_ok!(Omega::add_ship(RuntimeOrigin::signed(ALICE), ship));
				let ships = Ships::<Test>::iter_key_prefix(ALICE).collect::<Vec<_>>();
				assert_eq!(ships.len(), 2);
			});
	}
}

mod register_fleet {
	use super::*;

	#[test]
	fn register_defense_and_offense_should_work() {
		ExtBuilder::default().build().execute_with(|| {
			let commander_id = 1;
			prepare_hangar_for::<Test>(&ALICE);
			add_commander_to::<Test>(&ALICE, commander_id);

			let fleet = Fleet {
				composition: [
					FleetWing::new(0, WingFormation::Neutral, 10),
					FleetWing::new(2, WingFormation::Defensive, 10),
					FleetWing::new(1, WingFormation::Offensive, 10),
					FleetWing::new(1, WingFormation::Neutral, 10),
				],
				commander: commander_id,
			};

			assert_eq!(DefenseFleets::<Test>::get(ALICE), None);
			assert_ok!(Omega::register_defense(RuntimeOrigin::signed(ALICE), fleet));
			assert_eq!(DefenseFleets::<Test>::get(ALICE), Some(fleet));

			System::assert_last_event(mock::RuntimeEvent::Omega(
				crate::Event::DefenseFleetRegistered { by: ALICE },
			));

			assert_eq!(OffenseFleets::<Test>::get(ALICE), None);
			assert_ok!(Omega::register_offense(RuntimeOrigin::signed(ALICE), fleet));
			assert_eq!(OffenseFleets::<Test>::get(ALICE), Some(fleet));

			System::assert_last_event(mock::RuntimeEvent::Omega(
				crate::Event::OffenseFleetRegistered { by: ALICE },
			));
		});
	}

	#[test]
	fn cannot_register_fleet_with_non_owned_commander() {
		ExtBuilder::default().build().execute_with(|| {
			let commander_id = 3;
			prepare_hangar_for::<Test>(&ALICE);

			let fleet = Fleet {
				composition: [
					FleetWing::new(1, WingFormation::Defensive, 100),
					FleetWing::new(2, WingFormation::Neutral, 2),
					FleetWing::new(3, WingFormation::Offensive, 5),
					FleetWing::new(3, WingFormation::Offensive, 5),
				],
				commander: commander_id,
			};

			assert_noop!(
				Omega::register_defense(RuntimeOrigin::signed(ALICE), fleet),
				Error::<Test>::CommanderNotFound
			);
			assert_noop!(
				Omega::register_offense(RuntimeOrigin::signed(ALICE), fleet),
				Error::<Test>::CommanderNotFound
			);
		});
	}

	#[test]
	fn cannot_register_fleet_with_non_registered_ships() {
		ExtBuilder::default().build().execute_with(|| {
			let commander_id = 3;
			add_commander_to::<Test>(&ALICE, commander_id);

			let fleet = Fleet {
				composition: [
					FleetWing::new(0, WingFormation::Defensive, 100),
					FleetWing::new(1, WingFormation::Neutral, 2),
					FleetWing::new(1, WingFormation::Offensive, 5),
					FleetWing::new(3, WingFormation::Offensive, 5),
				],
				commander: commander_id,
			};

			assert_noop!(
				Omega::register_defense(RuntimeOrigin::signed(ALICE), fleet),
				Error::<Test>::ShipNotFound
			);
			assert_noop!(
				Omega::register_offense(RuntimeOrigin::signed(ALICE), fleet),
				Error::<Test>::ShipNotFound
			);
		});
	}
}

mod engage_player {
	use super::*;
	use crate::consts::{XP_PER_LOOT_CRATE, XP_PER_RANKED_WIN};

	#[test]
	fn engage_player_should_work() {
		ExtBuilder::default().build().execute_with(|| {
			let commander_id = 1;
			prepare_hangar_for::<Test>(&ALICE);
			prepare_hangar_for::<Test>(&BOB);
			add_commander_to::<Test>(&ALICE, commander_id);
			add_commander_to::<Test>(&BOB, commander_id);

			let fleet = Fleet {
				composition: [
					FleetWing::new(0, WingFormation::Offensive, 100),
					FleetWing::new(2, WingFormation::Defensive, 10),
					FleetWing::new(1, WingFormation::Offensive, 10),
					FleetWing::new(1, WingFormation::Neutral, 10),
				],
				commander: commander_id,
			};

			assert_ok!(Omega::register_defense(RuntimeOrigin::signed(ALICE), fleet));
			assert_ok!(Omega::register_offense(RuntimeOrigin::signed(BOB), fleet));

			assert_eq!(
				Commanders::<Test>::get(BOB, commander_id),
				Some(CommanderData { exp: XP_PER_LOOT_CRATE })
			);

			assert_eq!(EngagementHistory::<Test>::get(BOB, 0), None);

			assert_ok!(Omega::engage_player(RuntimeOrigin::signed(BOB), ALICE));

			let engagement = {
				let result = EngagementHistory::<Test>::get(BOB, 0);
				assert!(result.is_some());
				result.unwrap()
			};

			assert!(engagement.defender_defeated && !engagement.attacker_defeated);
			assert_eq!(
				Commanders::<Test>::get(BOB, commander_id),
				Some(CommanderData { exp: XP_PER_LOOT_CRATE + XP_PER_RANKED_WIN })
			);

			System::assert_last_event(mock::RuntimeEvent::Omega(
				crate::Event::EngagementFinished {
					attacker: BOB,
					target: ALICE,
					result: engagement,
				},
			));
		});
	}

	#[test]
	fn engage_player_fails_if_target_doesnt_have_defense_fleet_registered() {
		ExtBuilder::default().build().execute_with(|| {
			let commander_id = 1;
			prepare_hangar_for::<Test>(&ALICE);
			prepare_hangar_for::<Test>(&BOB);
			add_commander_to::<Test>(&ALICE, commander_id);
			add_commander_to::<Test>(&BOB, commander_id);

			let fleet = Fleet {
				composition: [
					FleetWing::new(0, WingFormation::Neutral, 100),
					FleetWing::new(2, WingFormation::Defensive, 10),
					FleetWing::new(3, WingFormation::Neutral, 30),
					FleetWing::new(1, WingFormation::Neutral, 10),
				],
				commander: commander_id,
			};

			assert_ok!(Omega::register_offense(RuntimeOrigin::signed(ALICE), fleet));
			assert_noop!(
				Omega::engage_player(RuntimeOrigin::signed(ALICE), BOB),
				Error::<Test>::DefensiveFleetNotRegistered
			);
		});
	}

	#[test]
	fn engage_player_fails_if_attacker_doesnt_have_an_offensive_fleet_registered() {
		ExtBuilder::default().build().execute_with(|| {
			let commander_id = 1;
			prepare_hangar_for::<Test>(&ALICE);
			prepare_hangar_for::<Test>(&BOB);
			add_commander_to::<Test>(&ALICE, commander_id);
			add_commander_to::<Test>(&BOB, commander_id);

			let fleet = Fleet {
				composition: [
					FleetWing::new(1, WingFormation::Defensive, 100),
					FleetWing::new(1, WingFormation::Defensive, 10),
					FleetWing::new(1, WingFormation::Defensive, 30),
					FleetWing::new(1, WingFormation::Defensive, 10),
				],
				commander: commander_id,
			};

			assert_ok!(Omega::register_defense(RuntimeOrigin::signed(ALICE), fleet));
			assert_noop!(
				Omega::engage_player(RuntimeOrigin::signed(BOB), ALICE),
				Error::<Test>::OffensiveFleetNotRegistered
			);
		});
	}
}
