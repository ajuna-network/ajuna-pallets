use crate::{mock::*, *};
use frame_support::{assert_noop, assert_ok, BoundedBTreeSet};

fn generate_action_secret_action_pair_for(
	action: PlayerAction,
	hash_fill: [u8; 28],
) -> (PlayerActionHash, PlayerActionHash) {
	let action_hash = {
		let mut hash = [0_u8; 32];

		hash[0..=27].copy_from_slice(&hash_fill);
		hash[28..].copy_from_slice(&action.generate_payload_for());

		hash
	};

	(sp_crypto_hashing::blake2_256(&action_hash), action_hash)
}

fn assert_all_storage_empty() {
	assert_eq!(BattleSchedules::<Test, Instance1>::iter().count(), 0);
	assert_eq!(BattleStateStore::<Test, Instance1>::get(), BattleStateFor::<Test>::Inactive);
	assert_eq!(PlayerDetails::<Test, Instance1>::iter().count(), 0);
	assert_eq!(GridOccupancy::<Test, Instance1>::iter().count(), 0);
}

fn assert_battle_state_is_active_with(
	battle_phase: BattlePhase,
	max_players: u8,
	runs_until: MockBlockNumber,
	grid_size: Coordinates,
) {
	assert_eq!(
		BattleStateStore::<Test, Instance1>::get(),
		BattleStateFor::<Test>::Active {
			phase: battle_phase,
			config: BattleConfig { max_players, run_until: runs_until },
			boundaries: GridBoundaries { top_left: Coordinates::new(1, 1), down_right: grid_size },
		}
	);
}

fn start_battle_with_config(
	battle_duration: u32,
	battle_max_players: u8,
	battle_grid_size: Coordinates,
) {
	assert_all_storage_empty();

	assert_ok!(BattleRoyale::try_start_battle(
		battle_duration,
		battle_max_players,
		battle_grid_size,
		vec![]
	));

	System::assert_last_event(mock::RuntimeEvent::BattleRoyale(crate::Event::BattleStarted));

	let current_block = System::block_number();
	let battle_runs_until = current_block + (battle_duration + QUEUE_DURATION) as MockBlockNumber;
	assert_battle_state_is_active_with(
		BattlePhase::Queueing,
		battle_max_players,
		battle_runs_until,
		battle_grid_size,
	);
}

fn queue_player(account: &MockAccountId, initial_weapon: PlayerWeapon) -> Coordinates {
	assert_ok!(BattleRoyale::try_queue_player(account, initial_weapon));

	System::assert_last_event(mock::RuntimeEvent::BattleRoyale(crate::Event::PlayerQueued {
		player: account.clone(),
	}));

	let maybe_player_details = PlayerDetails::<Test, Instance1>::get(account);
	assert!(maybe_player_details.is_some());
	let player_details = maybe_player_details.unwrap();
	let player_position = player_details.position;

	assert_eq!(
		player_details,
		PlayerData {
			position: player_position,
			weapon: initial_weapon,
			state: PlayerState::Inactive,
		}
	);

	let expected_set = {
		let mut set = BoundedBTreeSet::new();
		set.try_insert(account.clone()).expect("Account inserted into account set");
		set
	};

	assert_eq!(
		GridOccupancy::<Test, Instance1>::get(player_position),
		OccupancyState::Open(expected_set)
	);

	player_position
}

mod try_start_battle {
	use super::*;

	#[test]
	fn try_start_battle_works() {
		ExtBuilder::default().balances(&[(ALICE, 1000)]).build().execute_with(|| {
			let battle_duration = 500;
			let battle_max_players = 20;
			let battle_grid_size = Coordinates::new(20, 20);

			run_to_block(10);

			assert_all_storage_empty();

			assert_ok!(BattleRoyale::try_start_battle(
				battle_duration,
				battle_max_players,
				battle_grid_size,
				vec![]
			));

			System::assert_last_event(mock::RuntimeEvent::BattleRoyale(
				crate::Event::BattleStarted,
			));

			assert_eq!(BattleSchedules::<Test, Instance1>::iter().count(), 1);
			let current_block = System::block_number();
			let battle_runs_until =
				current_block + (battle_duration + QUEUE_DURATION) as MockBlockNumber;
			assert_battle_state_is_active_with(
				BattlePhase::Queueing,
				battle_max_players,
				battle_runs_until,
				battle_grid_size,
			);
			assert_eq!(PlayerDetails::<Test, Instance1>::iter().count(), 0);
			assert_eq!(GridOccupancy::<Test, Instance1>::iter().count(), 0);
		});
	}

	#[test]
	fn try_start_battle_fails_with_invalid_config() {
		ExtBuilder::default().balances(&[(ALICE, 1000)]).build().execute_with(|| {
			run_to_block(10);

			assert_all_storage_empty();

			assert_noop!(
				BattleRoyale::try_start_battle(
					MIN_BATTLE_DURATION - 1,
					20,
					Coordinates::new(15, 15),
					vec![]
				),
				Error::<Test, Instance1>::BattleConfigDurationTooLow
			);

			assert_all_storage_empty();

			assert_noop!(
				BattleRoyale::try_start_battle(
					MIN_BATTLE_DURATION,
					2,
					Coordinates::new(15, 15),
					vec![]
				),
				Error::<Test, Instance1>::BattleConfigTooFewPlayers
			);

			assert_noop!(
				BattleRoyale::try_start_battle(
					MIN_BATTLE_DURATION,
					124,
					Coordinates::new(15, 15),
					vec![]
				),
				Error::<Test, Instance1>::BattleConfigTooManyPlayers
			);

			assert_all_storage_empty();

			assert_noop!(
				BattleRoyale::try_start_battle(
					MIN_BATTLE_DURATION,
					20,
					Coordinates::new(5, 15),
					vec![]
				),
				Error::<Test, Instance1>::BattleConfigGridSizeInvalid
			);

			assert_all_storage_empty();

			assert_noop!(
				BattleRoyale::try_start_battle(
					MIN_BATTLE_DURATION,
					20,
					Coordinates::new(15, 5),
					vec![]
				),
				Error::<Test, Instance1>::BattleConfigGridSizeInvalid
			);

			assert_all_storage_empty();

			assert_noop!(
				BattleRoyale::try_start_battle(
					MIN_BATTLE_DURATION,
					20,
					Coordinates::new(200, 15),
					vec![]
				),
				Error::<Test, Instance1>::BattleConfigGridSizeInvalid
			);

			assert_all_storage_empty();

			assert_noop!(
				BattleRoyale::try_start_battle(
					MIN_BATTLE_DURATION,
					20,
					Coordinates::new(15, 200),
					vec![]
				),
				Error::<Test, Instance1>::BattleConfigGridSizeInvalid
			);

			assert_all_storage_empty();
		});
	}

	#[test]
	fn try_start_battle_fails_with_already_running_battle() {
		ExtBuilder::default().balances(&[(ALICE, 1000)]).build().execute_with(|| {
			run_to_block(10);

			assert_all_storage_empty();

			assert_ok!(BattleRoyale::try_start_battle(
				MIN_BATTLE_DURATION,
				20,
				Coordinates::new(20, 20),
				vec![]
			));

			assert_noop!(
				BattleRoyale::try_start_battle(
					MIN_BATTLE_DURATION,
					20,
					Coordinates::new(15, 200),
					vec![]
				),
				Error::<Test, Instance1>::BattleAlreadyStarted
			);
		});
	}
}

mod try_finish_battle {
	use super::*;

	#[test]
	fn try_finish_battle_works() {
		ExtBuilder::default().balances(&[(ALICE, 1000)]).build().execute_with(|| {
			let battle_duration = 500;
			let battle_max_players = 20;
			let battle_grid_size = Coordinates::new(20, 20);

			run_to_block(10);
			let battle_runs_until = 10 + (battle_duration + QUEUE_DURATION) as MockBlockNumber;
			start_battle_with_config(battle_duration, battle_max_players, battle_grid_size);

			let alice_weapon = PlayerWeapon::Rock;
			let _ = queue_player(&ALICE, alice_weapon);
			let bob_weapon = PlayerWeapon::Scissors;
			let _ = queue_player(&BOB, bob_weapon);

			run_to_block((10 + QUEUE_DURATION) as u64);

			assert_eq!(PlayerDetails::<Test, Instance1>::iter().count(), 2);

			assert_battle_state_is_active_with(
				BattlePhase::Input,
				battle_max_players,
				battle_runs_until,
				battle_grid_size,
			);

			run_to_block(battle_runs_until);

			System::assert_last_event(mock::RuntimeEvent::BattleRoyale(
				crate::Event::BattleFinished,
			));

			let shrunk_coordinates = Coordinates::new(16, 15);
			assert_battle_state_is_active_with(
				BattlePhase::Finished,
				battle_max_players,
				battle_runs_until,
				shrunk_coordinates,
			);

			let results = BattleRoyale::try_finish_battle();
			assert_ok!(&results);

			let mut defeated_accounts = results.expect("Get defeated accounts");
			assert_eq!(defeated_accounts.len(), 1);
			assert_eq!(defeated_accounts.pop(), Some(BOB));
		});
	}

	#[test]
	fn try_finish_battle_works_with_no_players() {
		ExtBuilder::default().balances(&[(ALICE, 1000)]).build().execute_with(|| {
			let battle_duration = 500;
			let battle_max_players = 20;
			let battle_grid_size = Coordinates::new(20, 20);

			run_to_block(10);
			let battle_runs_until = 10 + (battle_duration + QUEUE_DURATION) as MockBlockNumber;
			start_battle_with_config(battle_duration, battle_max_players, battle_grid_size);

			run_to_block((10 + QUEUE_DURATION) as u64);

			assert_eq!(PlayerDetails::<Test, Instance1>::iter().count(), 0);

			assert_battle_state_is_active_with(
				BattlePhase::Input,
				battle_max_players,
				battle_runs_until,
				battle_grid_size,
			);

			// We move 8 blocks to reach the Verification phase.
			// Because there were 0 players in it, the battle will then finish in its first
			// phase cycle.
			run_to_block(System::block_number() + 8);

			System::assert_last_event(mock::RuntimeEvent::BattleRoyale(
				crate::Event::BattleFinished,
			));

			assert_battle_state_is_active_with(
				BattlePhase::Finished,
				battle_max_players,
				battle_runs_until,
				battle_grid_size,
			);

			let results = BattleRoyale::try_finish_battle();
			assert_ok!(&results);

			let defeated_accounts = results.expect("Get defeated accounts");
			assert_eq!(defeated_accounts.len(), 0);
		});
	}

	#[test]
	fn try_finish_battle_fails_with_battle_not_in_finished_state() {
		ExtBuilder::default().balances(&[(ALICE, 1000)]).build().execute_with(|| {
			let battle_duration = 500;
			let battle_max_players = 20;
			let battle_grid_size = Coordinates::new(20, 20);

			run_to_block(10);
			let battle_runs_until = 10 + (battle_duration + QUEUE_DURATION) as MockBlockNumber;
			start_battle_with_config(battle_duration, battle_max_players, battle_grid_size);

			run_to_block((10 + QUEUE_DURATION) as u64);

			assert_eq!(PlayerDetails::<Test, Instance1>::iter().count(), 0);

			assert_battle_state_is_active_with(
				BattlePhase::Input,
				battle_max_players,
				battle_runs_until,
				battle_grid_size,
			);

			assert_noop!(
				BattleRoyale::try_finish_battle(),
				Error::<Test, Instance1>::BattleNotInFinishedPhase
			);
		});
	}
}

mod try_queue_player {
	use super::*;

	#[test]
	fn try_queue_player_works() {
		ExtBuilder::default().balances(&[(ALICE, 1000)]).build().execute_with(|| {
			let battle_duration = 500;
			let battle_max_players = 20;
			let battle_grid_size = Coordinates::new(20, 20);

			run_to_block(10);
			let battle_runs_until = 10 + (battle_duration + QUEUE_DURATION) as MockBlockNumber;
			start_battle_with_config(battle_duration, battle_max_players, battle_grid_size);

			let alice_weapon = PlayerWeapon::Paper;
			assert_ok!(BattleRoyale::try_queue_player(&ALICE, alice_weapon));

			System::assert_last_event(mock::RuntimeEvent::BattleRoyale(
				crate::Event::PlayerQueued { player: ALICE },
			));

			assert_eq!(PlayerDetails::<Test, Instance1>::iter().count(), 1);
			let alice_details =
				PlayerDetails::<Test, Instance1>::get(ALICE).expect("Get ALICE PlayerDetails");
			let alice_position = alice_details.position;
			assert_eq!(
				alice_details,
				PlayerData {
					position: alice_position,
					weapon: alice_weapon,
					state: PlayerState::Inactive,
				}
			);

			assert_eq!(GridOccupancy::<Test, Instance1>::iter().count(), 1);
			let expected_set = {
				let mut set = BoundedBTreeSet::new();
				set.try_insert(ALICE).expect("Account inserted into account set");
				set
			};
			assert_eq!(
				GridOccupancy::<Test, Instance1>::get(alice_position),
				OccupancyState::Open(expected_set)
			);

			run_to_block(25);

			let bob_weapon = PlayerWeapon::Rock;
			assert_ok!(BattleRoyale::try_queue_player(&BOB, bob_weapon));

			System::assert_last_event(mock::RuntimeEvent::BattleRoyale(
				crate::Event::PlayerQueued { player: BOB },
			));

			assert_eq!(PlayerDetails::<Test, Instance1>::iter().count(), 2);
			let bob_details =
				PlayerDetails::<Test, Instance1>::get(BOB).expect("Get BOB PlayerDetails");
			let bob_position = bob_details.position;
			assert_eq!(
				bob_details,
				PlayerData {
					position: bob_position,
					weapon: bob_weapon,
					state: PlayerState::Inactive,
				}
			);
			assert_eq!(GridOccupancy::<Test, Instance1>::iter().count(), 2);
			let expected_set = {
				let mut set = BoundedBTreeSet::new();
				set.try_insert(BOB).expect("Account inserted into account set");
				set
			};
			assert_eq!(
				GridOccupancy::<Test, Instance1>::get(bob_position),
				OccupancyState::Open(expected_set)
			);

			run_to_block((10 + QUEUE_DURATION) as u64);

			assert_eq!(BattleSchedules::<Test, Instance1>::iter().count(), 4);
			assert_battle_state_is_active_with(
				BattlePhase::Input,
				battle_max_players,
				battle_runs_until,
				battle_grid_size,
			);
			assert_eq!(PlayerDetails::<Test, Instance1>::iter().count(), 2);
			assert_eq!(GridOccupancy::<Test, Instance1>::iter().count(), 2);
		});
	}

	#[test]
	fn try_queue_player_fails_with_battle_not_in_queueing_state() {
		ExtBuilder::default().balances(&[(ALICE, 1000)]).build().execute_with(|| {
			run_to_block(10);

			assert_all_storage_empty();

			assert_noop!(
				BattleRoyale::try_queue_player(&ALICE, PlayerWeapon::Paper),
				Error::<Test, Instance1>::BattleNotInQueueingPhase
			);
		});
	}

	#[test]
	fn try_queue_player_fails_with_already_queued_player() {
		ExtBuilder::default().balances(&[(ALICE, 1000)]).build().execute_with(|| {
			let battle_duration = 500;
			let battle_max_players = 20;
			let battle_grid_size = Coordinates::new(20, 20);

			run_to_block(10);
			start_battle_with_config(battle_duration, battle_max_players, battle_grid_size);

			let _ = queue_player(&ALICE, PlayerWeapon::Rock);

			assert_noop!(
				BattleRoyale::try_queue_player(&ALICE, PlayerWeapon::Paper),
				Error::<Test, Instance1>::PlayerAlreadyQueued
			);
		});
	}

	#[test]
	fn try_queue_player_fails_with_player_queue_full() {
		ExtBuilder::default().balances(&[(ALICE, 1000)]).build().execute_with(|| {
			let battle_duration = 500;
			let battle_max_players = 4;
			let battle_grid_size = Coordinates::new(20, 20);

			run_to_block(10);
			start_battle_with_config(battle_duration, battle_max_players, battle_grid_size);

			queue_player(&ALICE, PlayerWeapon::Paper);
			queue_player(&BOB, PlayerWeapon::Rock);
			queue_player(&CHARLIE, PlayerWeapon::Scissors);
			queue_player(&DAVE, PlayerWeapon::Paper);

			assert_eq!(PlayerDetails::<Test, Instance1>::iter().count(), 4);
			assert_eq!(GridOccupancy::<Test, Instance1>::iter().count(), 4);

			assert_noop!(
				BattleRoyale::try_queue_player(&EDWARD, PlayerWeapon::Scissors),
				Error::<Test, Instance1>::PlayerQueueFull
			);
		});
	}
}

mod try_perform_player_action {
	use super::*;

	#[test]
	fn try_perform_player_action_works() {
		ExtBuilder::default().balances(&[(ALICE, 1000)]).build().execute_with(|| {
			let battle_duration = 500;
			let battle_max_players = 20;
			let battle_grid_size = Coordinates::new(20, 20);

			run_to_block(10);
			let battle_runs_until = 10 + (battle_duration + QUEUE_DURATION) as MockBlockNumber;
			start_battle_with_config(battle_duration, battle_max_players, battle_grid_size);

			let alice_weapon = PlayerWeapon::Rock;
			let alice_initial_position = queue_player(&ALICE, alice_weapon);
			let bob_weapon = PlayerWeapon::Rock;
			let bob_initial_position = queue_player(&BOB, bob_weapon);

			run_to_block(22);

			let charlie_weapon = PlayerWeapon::Scissors;
			let charlie_initial_position = queue_player(&CHARLIE, charlie_weapon);

			run_to_block((10 + QUEUE_DURATION) as u64);

			assert_eq!(PlayerDetails::<Test, Instance1>::iter().count(), 3);
			assert_battle_state_is_active_with(
				BattlePhase::Input,
				battle_max_players,
				battle_runs_until,
				battle_grid_size,
			);

			// ALICE - Performs action
			let alice_moved_to_position = Coordinates::new(3, 4);
			let alice_payload_fill = [2_u8; 28];
			let (alice_input_hash, alice_reveal_hash) = generate_action_secret_action_pair_for(
				PlayerAction::Move(alice_moved_to_position),
				alice_payload_fill,
			);
			assert_ok!(BattleRoyale::try_perform_player_action(&ALICE, alice_input_hash));

			System::assert_last_event(mock::RuntimeEvent::BattleRoyale(
				crate::Event::PlayerPerformedAction { player: ALICE },
			));

			assert_eq!(
				PlayerDetails::<Test, Instance1>::get(ALICE),
				Some(PlayerData {
					position: alice_initial_position,
					weapon: alice_weapon,
					state: PlayerState::PerformedAction(alice_input_hash),
				})
			);

			// BOB - Performs action
			let bob_weapon_change = PlayerWeapon::Scissors;
			let bob_payload_fill = [6_u8; 28];
			let (bob_input_hash, bob_reveal_hash) = generate_action_secret_action_pair_for(
				PlayerAction::SwapWeapon(bob_weapon_change),
				bob_payload_fill,
			);
			assert_ok!(BattleRoyale::try_perform_player_action(&BOB, bob_input_hash));

			System::assert_last_event(mock::RuntimeEvent::BattleRoyale(
				crate::Event::PlayerPerformedAction { player: BOB },
			));

			assert_eq!(
				PlayerDetails::<Test, Instance1>::get(BOB),
				Some(PlayerData {
					position: bob_initial_position,
					weapon: bob_weapon,
					state: PlayerState::PerformedAction(bob_input_hash),
				})
			);

			// We advance 1 block, we should still be in the Input phase
			run_to_block(System::block_number() + 1);
			assert_battle_state_is_active_with(
				BattlePhase::Input,
				battle_max_players,
				battle_runs_until,
				battle_grid_size,
			);

			// CHARLIE - Performs action
			let charlie_moved_to_position = Coordinates::new(5, 5);
			let charlie_weapon_change = PlayerWeapon::Paper;

			let charlie_payload_fill = [46_u8; 28];
			let (charlie_input_hash, charlie_reveal_hash) = generate_action_secret_action_pair_for(
				PlayerAction::MoveAndSwap(charlie_moved_to_position, charlie_weapon_change),
				charlie_payload_fill,
			);
			assert_ok!(BattleRoyale::try_perform_player_action(&CHARLIE, charlie_input_hash));

			System::assert_last_event(mock::RuntimeEvent::BattleRoyale(
				crate::Event::PlayerPerformedAction { player: CHARLIE },
			));

			assert_eq!(
				PlayerDetails::<Test, Instance1>::get(CHARLIE),
				Some(PlayerData {
					position: charlie_initial_position,
					weapon: charlie_weapon,
					state: PlayerState::PerformedAction(charlie_input_hash),
				})
			);

			// We advance 2 more blocks, this should put us in the Reveal phase
			run_to_block(System::block_number() + 2);
			assert_battle_state_is_active_with(
				BattlePhase::Reveal,
				battle_max_players,
				battle_runs_until,
				battle_grid_size,
			);

			// ALICE - Reveals action
			assert_ok!(BattleRoyale::try_perform_player_action(&ALICE, alice_reveal_hash));

			System::assert_last_event(mock::RuntimeEvent::BattleRoyale(
				crate::Event::PlayerRevealedAction { player: ALICE },
			));

			assert_eq!(
				PlayerDetails::<Test, Instance1>::get(ALICE),
				Some(PlayerData {
					position: alice_moved_to_position,
					weapon: alice_weapon,
					state: PlayerState::RevealedAction,
				})
			);

			// We advance 1 block, we should still be in the Reveal phase
			run_to_block(System::block_number() + 1);
			assert_battle_state_is_active_with(
				BattlePhase::Reveal,
				battle_max_players,
				battle_runs_until,
				battle_grid_size,
			);

			// CHARLIE - Reveals action
			assert_ok!(BattleRoyale::try_perform_player_action(&CHARLIE, charlie_reveal_hash));

			System::assert_last_event(mock::RuntimeEvent::BattleRoyale(
				crate::Event::PlayerRevealedAction { player: CHARLIE },
			));

			assert_eq!(
				PlayerDetails::<Test, Instance1>::get(CHARLIE),
				Some(PlayerData {
					position: charlie_moved_to_position,
					weapon: charlie_weapon_change,
					state: PlayerState::RevealedAction,
				})
			);

			// We advance 1 block, we should still be in the Reveal phase
			run_to_block(System::block_number() + 1);
			assert_battle_state_is_active_with(
				BattlePhase::Reveal,
				battle_max_players,
				battle_runs_until,
				battle_grid_size,
			);

			// BOB - Reveals action
			assert_ok!(BattleRoyale::try_perform_player_action(&BOB, bob_reveal_hash));

			System::assert_last_event(mock::RuntimeEvent::BattleRoyale(
				crate::Event::PlayerRevealedAction { player: BOB },
			));

			assert_eq!(
				PlayerDetails::<Test, Instance1>::get(BOB),
				Some(PlayerData {
					position: bob_initial_position,
					weapon: bob_weapon_change,
					state: PlayerState::RevealedAction,
				})
			);

			// We advance 1 block, this should put us in the Execution phase
			run_to_block(System::block_number() + 1);
			assert_battle_state_is_active_with(
				BattlePhase::Execution,
				battle_max_players,
				battle_runs_until,
				battle_grid_size,
			);

			// No one should have been defeated since all players are in different positions
			assert_eq!(PlayerDetails::<Test, Instance1>::iter().count(), 3);
			// Since two players have moved the total amount of cells stored in the map
			// should have gone up by 2 from the original 3
			assert_eq!(GridOccupancy::<Test, Instance1>::iter().count(), 5);

			// We advance 1 more block, this should put us in the Verification phase
			run_to_block(System::block_number() + 1);
			assert_battle_state_is_active_with(
				BattlePhase::Verification,
				battle_max_players,
				battle_runs_until,
				battle_grid_size,
			);

			// We advance 1 more block, this should put us in the Input phase again
			// since the game will not have ended in the latest Verification phase
			run_to_block(System::block_number() + 1);
			assert_battle_state_is_active_with(
				BattlePhase::Input,
				battle_max_players,
				battle_runs_until,
				battle_grid_size,
			);
		});
	}

	#[test]
	fn try_perform_player_action_works_with_combats() {
		ExtBuilder::default().balances(&[(ALICE, 1000)]).build().execute_with(|| {
			let battle_duration = 500;
			let battle_max_players = 20;
			let battle_grid_size = Coordinates::new(20, 20);

			run_to_block(10);
			let battle_runs_until = 10 + (battle_duration + QUEUE_DURATION) as MockBlockNumber;
			start_battle_with_config(battle_duration, battle_max_players, battle_grid_size);

			let alice_weapon = PlayerWeapon::Rock;
			let alice_initial_position = queue_player(&ALICE, alice_weapon);
			let bob_weapon = PlayerWeapon::Scissors;
			let bob_initial_position = queue_player(&BOB, bob_weapon);

			run_to_block((10 + QUEUE_DURATION) as u64);

			assert_eq!(PlayerDetails::<Test, Instance1>::iter().count(), 2);

			assert_battle_state_is_active_with(
				BattlePhase::Input,
				battle_max_players,
				battle_runs_until,
				battle_grid_size,
			);

			let move_to_position = Coordinates::new(3, 4);
			let action = PlayerAction::Move(move_to_position);

			// ALICE - Performs moves first
			let alice_payload_fill = [0_u8; 28];
			let (alice_input_hash, alice_reveal_hash) =
				generate_action_secret_action_pair_for(action, alice_payload_fill);

			assert_ok!(BattleRoyale::try_perform_player_action(&ALICE, alice_input_hash));

			System::assert_last_event(mock::RuntimeEvent::BattleRoyale(
				crate::Event::PlayerPerformedAction { player: ALICE },
			));

			assert_eq!(
				PlayerDetails::<Test, Instance1>::get(ALICE),
				Some(PlayerData {
					position: alice_initial_position,
					weapon: alice_weapon,
					state: PlayerState::PerformedAction(alice_input_hash),
				})
			);

			// BOB - Performs move to the same position as ALICE
			let bob_payload_fill = [182_u8; 28];
			let (bob_input_hash, bob_reveal_hash) =
				generate_action_secret_action_pair_for(action, bob_payload_fill);
			assert_ok!(BattleRoyale::try_perform_player_action(&BOB, bob_input_hash));

			System::assert_last_event(mock::RuntimeEvent::BattleRoyale(
				crate::Event::PlayerPerformedAction { player: BOB },
			));

			assert_eq!(
				PlayerDetails::<Test, Instance1>::get(BOB),
				Some(PlayerData {
					position: bob_initial_position,
					weapon: bob_weapon,
					state: PlayerState::PerformedAction(bob_input_hash),
				})
			);

			// We advance 3 block, this will put us in the Reveal phase
			run_to_block(System::block_number() + 3);
			assert_battle_state_is_active_with(
				BattlePhase::Reveal,
				battle_max_players,
				battle_runs_until,
				battle_grid_size,
			);

			// BOB - Reveals its move first
			assert_ok!(BattleRoyale::try_perform_player_action(&BOB, bob_reveal_hash));

			System::assert_last_event(mock::RuntimeEvent::BattleRoyale(
				crate::Event::PlayerRevealedAction { player: BOB },
			));

			assert_eq!(
				PlayerDetails::<Test, Instance1>::get(BOB),
				Some(PlayerData {
					position: move_to_position,
					weapon: bob_weapon,
					state: PlayerState::RevealedAction,
				})
			);

			// We advance 1 block, we should still be in the Reveal phase
			run_to_block(System::block_number() + 1);
			assert_battle_state_is_active_with(
				BattlePhase::Reveal,
				battle_max_players,
				battle_runs_until,
				battle_grid_size,
			);

			// ALICE - Reveals its move next
			assert_ok!(BattleRoyale::try_perform_player_action(&ALICE, alice_reveal_hash));

			System::assert_last_event(mock::RuntimeEvent::BattleRoyale(
				crate::Event::PlayerRevealedAction { player: ALICE },
			));

			assert_eq!(
				PlayerDetails::<Test, Instance1>::get(ALICE),
				Some(PlayerData {
					position: move_to_position,
					weapon: alice_weapon,
					state: PlayerState::RevealedAction,
				})
			);

			// We verify that the GridOccupancy contains the accounts in the expected position
			let expect_accounts = {
				let mut set = BoundedBTreeSet::new();
				set.try_insert(BOB).expect("BOB inserted into account set");
				set.try_insert(ALICE).expect("ALICE inserted into account set");
				set
			};
			assert_eq!(
				GridOccupancy::<Test, Instance1>::get(move_to_position),
				OccupancyState::Open(expect_accounts)
			);

			// We advance 2 more blocks, this should put us in the Execution phase
			run_to_block(System::block_number() + 2);
			assert_battle_state_is_active_with(
				BattlePhase::Execution,
				battle_max_players,
				battle_runs_until,
				battle_grid_size,
			);

			// During the start of the Execution phase block a battle between ALICE and BOB
			// happened, in which BOB was defeated since Scissors loses to Rock
			System::assert_last_event(mock::RuntimeEvent::BattleRoyale(
				crate::Event::PlayerDefeated { player: BOB },
			));

			assert_eq!(
				PlayerDetails::<Test, Instance1>::get(ALICE),
				Some(PlayerData {
					position: move_to_position,
					weapon: alice_weapon,
					state: PlayerState::RevealedAction,
				})
			);

			assert_eq!(
				PlayerDetails::<Test, Instance1>::get(BOB),
				Some(PlayerData {
					position: move_to_position,
					weapon: bob_weapon,
					state: PlayerState::Defeated,
				})
			);

			assert_eq!(PlayerDetails::<Test, Instance1>::iter().count(), 2);
			// Since both players have moved and one player has been defeated
			// the total amount of cells in the map should be 2 + 2 - 1
			assert_eq!(GridOccupancy::<Test, Instance1>::iter().count(), 3);

			// We advance 1 more block, this should put us in the Finished phase,
			// since only 1 player is left which means during Verification we will switch in
			// the same block to Finished phase
			run_to_block(System::block_number() + 1);
			assert_battle_state_is_active_with(
				BattlePhase::Finished,
				battle_max_players,
				battle_runs_until,
				battle_grid_size,
			);
		});
	}

	#[test]
	fn try_perform_player_action_works_with_massive_combat() {
		ExtBuilder::default().balances(&[(ALICE, 1000)]).build().execute_with(|| {
			let battle_duration = 500;
			let battle_max_players = MAX_PLAYER_PER_BATTLE;
			let battle_grid_size = Coordinates::new(20, 20);

			run_to_block(10);
			let battle_runs_until = 10 + (battle_duration + QUEUE_DURATION) as MockBlockNumber;
			start_battle_with_config(battle_duration, battle_max_players, battle_grid_size);

			let account_vec = (0..MAX_PLAYER_PER_BATTLE)
				.map(|player_seed| MockAccountId::new([player_seed; 32]))
				.collect::<Vec<_>>();
			let account_count = account_vec.len();

			let mut initial_position_vec = Vec::with_capacity(account_vec.len());

			for (i, account) in account_vec.iter().enumerate() {
				let initial_position = if i == (account_count - 1) {
					queue_player(account, PlayerWeapon::Rock)
				} else {
					queue_player(account, PlayerWeapon::Scissors)
				};
				initial_position_vec.push(initial_position);
			}

			// We start the Input phase
			run_to_block((10 + QUEUE_DURATION) as u64);

			assert_eq!(PlayerDetails::<Test, Instance1>::iter().count(), 64);

			assert_battle_state_is_active_with(
				BattlePhase::Input,
				battle_max_players,
				battle_runs_until,
				battle_grid_size,
			);

			let move_to_coordinates = Coordinates::new(4, 4);
			let action = PlayerAction::Move(move_to_coordinates);

			for (i, account) in account_vec.iter().enumerate() {
				let payload_fill = [i as u8; 28];
				let (input_hash, _) = generate_action_secret_action_pair_for(action, payload_fill);
				assert_ok!(BattleRoyale::try_perform_player_action(account, input_hash));

				System::assert_last_event(mock::RuntimeEvent::BattleRoyale(
					crate::Event::PlayerPerformedAction { player: account.clone() },
				));

				assert_eq!(
					PlayerDetails::<Test, Instance1>::get(account),
					Some(PlayerData {
						position: initial_position_vec[i],
						weapon: if i == (account_count - 1) {
							PlayerWeapon::Rock
						} else {
							PlayerWeapon::Scissors
						},
						state: PlayerState::PerformedAction(input_hash),
					})
				);
			}

			// We advance 3 block, this will put us in the Reveal phase
			run_to_block(System::block_number() + 3);
			assert_battle_state_is_active_with(
				BattlePhase::Reveal,
				battle_max_players,
				battle_runs_until,
				battle_grid_size,
			);

			let mut account_set = BoundedBTreeSet::new();

			for (i, account) in account_vec.iter().enumerate() {
				let payload_fill = [i as u8; 28];
				let (_, reveal_hash) = generate_action_secret_action_pair_for(action, payload_fill);
				assert_ok!(BattleRoyale::try_perform_player_action(account, reveal_hash));

				System::assert_last_event(mock::RuntimeEvent::BattleRoyale(
					crate::Event::PlayerRevealedAction { player: account.clone() },
				));

				assert_eq!(
					PlayerDetails::<Test, Instance1>::get(account),
					Some(PlayerData {
						position: move_to_coordinates,
						weapon: if i == (account_count - 1) {
							PlayerWeapon::Rock
						} else {
							PlayerWeapon::Scissors
						},
						state: PlayerState::RevealedAction,
					})
				);

				account_set
					.try_insert(account.clone())
					.expect("Inserted account in account_set");
			}

			assert_eq!(
				GridOccupancy::<Test, Instance1>::get(move_to_coordinates),
				OccupancyState::Open(account_set)
			);

			// We advance 1 block, this will put us in the Execution phase
			run_to_block(System::block_number() + 3);
			assert_battle_state_is_active_with(
				BattlePhase::Execution,
				battle_max_players,
				battle_runs_until,
				battle_grid_size,
			);

			// A battle will have occurred at the start of the block and all players
			// except one will have been defeated
			assert_eq!(
				PlayerDetails::<Test, Instance1>::iter_values()
					.filter(|player| player.state == PlayerState::Defeated)
					.count(),
				(MAX_PLAYER_PER_BATTLE - 1) as usize
			);

			// We advance 1 more block, this should put us in the Finished phase,
			// since only 1 player is left which means during Verification we will switch in
			// the same block to Finished phase
			run_to_block(System::block_number() + 1);
			assert_battle_state_is_active_with(
				BattlePhase::Finished,
				battle_max_players,
				battle_runs_until,
				battle_grid_size,
			);
		});
	}

	#[test]
	fn try_perform_player_action_works_with_wall_shrinking() {
		ExtBuilder::default().balances(&[(ALICE, 1000)]).build().execute_with(|| {
			let battle_duration = 500;
			let battle_max_players = 20;
			let battle_grid_size = Coordinates::new(20, 20);

			run_to_block(10);
			let battle_runs_until = 10 + (battle_duration + QUEUE_DURATION) as MockBlockNumber;
			start_battle_with_config(battle_duration, battle_max_players, battle_grid_size);

			let alice_weapon = PlayerWeapon::Rock;
			let alice_initial_position = queue_player(&ALICE, alice_weapon);
			let bob_weapon = PlayerWeapon::Scissors;
			let bob_initial_position = queue_player(&BOB, bob_weapon);

			// We forcefully move ALICE so that she gets caught in the first wall shrink
			let move_to_coordinates = Coordinates::new(20, 20);
			BattleRoyale::update_player_positions_storage(
				&ALICE,
				alice_initial_position,
				move_to_coordinates,
			);
			PlayerDetails::<Test, Instance1>::mutate(&ALICE, |maybe_details| {
				if let Some(details) = maybe_details {
					details.position = move_to_coordinates;
				}
			});

			run_to_block((10 + QUEUE_DURATION) as u64);

			assert_eq!(PlayerDetails::<Test, Instance1>::iter().count(), 2);

			assert_battle_state_is_active_with(
				BattlePhase::Input,
				battle_max_players,
				battle_runs_until,
				battle_grid_size,
			);

			// We advance 23 blocks, which will put us in the Shrink phase
			run_to_block(System::block_number() + 23);
			// The grid bound has shrunk
			let battle_grid_size = Coordinates::new(20, 19);
			assert_battle_state_is_active_with(
				BattlePhase::Shrink,
				battle_max_players,
				battle_runs_until,
				battle_grid_size,
			);

			// ALICE was caught in the shrinking
			System::assert_last_event(mock::RuntimeEvent::BattleRoyale(
				crate::Event::PlayerDefeated { player: ALICE },
			));

			assert_eq!(
				PlayerDetails::<Test, Instance1>::get(ALICE),
				Some(PlayerData {
					position: move_to_coordinates,
					weapon: alice_weapon,
					state: PlayerState::Defeated,
				})
			);

			assert_eq!(
				PlayerDetails::<Test, Instance1>::get(BOB),
				Some(PlayerData {
					position: bob_initial_position,
					weapon: bob_weapon,
					state: PlayerState::Inactive,
				})
			);

			// We advance 1 more block, this should put us in the Finished phase,
			// since only 1 player is left which means during Verification we will switch in
			// the same block to Finished phase
			run_to_block(System::block_number() + 1);
			// The grid bound has shrunk
			let battle_grid_size = Coordinates::new(20, 19);
			assert_battle_state_is_active_with(
				BattlePhase::Finished,
				battle_max_players,
				battle_runs_until,
				battle_grid_size,
			);
		});
	}

	#[test]
	fn try_perform_player_action_fails_with_battle_in_a_non_playable_state() {
		ExtBuilder::default().balances(&[(ALICE, 1000)]).build().execute_with(|| {
			let battle_duration = 500;
			let battle_max_players = 20;
			let battle_grid_size = Coordinates::new(20, 20);

			run_to_block(10);
			let battle_runs_until = 10 + (battle_duration + QUEUE_DURATION) as MockBlockNumber;

			start_battle_with_config(battle_duration, battle_max_players, battle_grid_size);
			let _ = queue_player(&ALICE, PlayerWeapon::Rock);
			let bob_initial_position = queue_player(&BOB, PlayerWeapon::Scissors);

			// We forcefully move BOB so that she gets caught in the first wall shrink
			BattleRoyale::update_player_positions_storage(
				&BOB,
				bob_initial_position,
				Coordinates::new(20, 20),
			);
			PlayerDetails::<Test, Instance1>::mutate(&BOB, |maybe_details| {
				if let Some(details) = maybe_details {
					details.position = Coordinates::new(20, 20);
				}
			});

			run_to_block(20);

			let payload_fill = [88; 28];
			let action_base = PlayerAction::Move(Coordinates::new(3, 3));
			let input_hash = action_base.generate_hash_for(payload_fill);

			// Cannot perform action while battle is in Queueing state
			assert_battle_state_is_active_with(
				BattlePhase::Queueing,
				battle_max_players,
				battle_runs_until,
				battle_grid_size,
			);
			assert_noop!(
				BattleRoyale::try_perform_player_action(&ALICE, input_hash),
				Error::<Test, Instance1>::BattleNotInPlayablePhases
			);

			// We advance 46 more blocks, this should put us in the Execution phase.
			// Trying to perform an action here is not allowed
			run_to_block(System::block_number() + 46);
			assert_battle_state_is_active_with(
				BattlePhase::Execution,
				battle_max_players,
				battle_runs_until,
				battle_grid_size,
			);
			assert_noop!(
				BattleRoyale::try_perform_player_action(&ALICE, input_hash),
				Error::<Test, Instance1>::BattleNotInPlayablePhases
			);

			// We advance 1 more block, this should put us in the Verification phase.
			// Trying to perform an action here is also not allowed
			run_to_block(System::block_number() + 1);
			assert_battle_state_is_active_with(
				BattlePhase::Verification,
				battle_max_players,
				battle_runs_until,
				battle_grid_size,
			);
			assert_noop!(
				BattleRoyale::try_perform_player_action(&ALICE, input_hash),
				Error::<Test, Instance1>::BattleNotInPlayablePhases
			);

			// We advance 16 more block, this should put us in the Shrink phase.
			// Trying to perform an action here is also not allowed
			run_to_block(System::block_number() + 16);
			// The grid bound has shrunk
			let battle_grid_size = Coordinates::new(20, 19);
			assert_battle_state_is_active_with(
				BattlePhase::Shrink,
				battle_max_players,
				battle_runs_until,
				battle_grid_size,
			);
			assert_noop!(
				BattleRoyale::try_perform_player_action(&ALICE, input_hash),
				Error::<Test, Instance1>::BattleNotInPlayablePhases
			);

			// We advance 1 more block, this should put us in the Finished phase.
			// Trying to perform an action here is also not allowed
			run_to_block(System::block_number() + 1);
			assert_battle_state_is_active_with(
				BattlePhase::Finished,
				battle_max_players,
				battle_runs_until,
				battle_grid_size,
			);
			assert_noop!(
				BattleRoyale::try_perform_player_action(&ALICE, input_hash),
				Error::<Test, Instance1>::BattleNotInPlayablePhases
			);
		});
	}

	#[test]
	fn try_perform_player_action_fails_with_non_queued_player() {
		ExtBuilder::default().balances(&[(ALICE, 1000)]).build().execute_with(|| {
			let battle_duration = 500;
			let battle_max_players = 20;
			let battle_grid_size = Coordinates::new(20, 20);

			run_to_block(10);
			let battle_runs_until = 10 + (battle_duration + QUEUE_DURATION) as MockBlockNumber;

			start_battle_with_config(battle_duration, battle_max_players, battle_grid_size);
			queue_player(&ALICE, PlayerWeapon::Rock);

			// We advance 50 more blocks, this should put us in the Input phase.
			run_to_block(System::block_number() + 50);
			assert_battle_state_is_active_with(
				BattlePhase::Input,
				battle_max_players,
				battle_runs_until,
				battle_grid_size,
			);

			let payload_fill = [23; 28];
			let action_base = PlayerAction::SwapWeapon(PlayerWeapon::Paper);
			let input_hash = action_base.generate_hash_for(payload_fill);

			// We fail to play a move since the player was not queued
			assert_noop!(
				BattleRoyale::try_perform_player_action(&BOB, input_hash),
				Error::<Test, Instance1>::PlayerNotFound
			);

			// Attempting to queue fails since we are outside the Queuing phase
			assert_noop!(
				BattleRoyale::try_queue_player(&BOB, PlayerWeapon::Paper),
				Error::<Test, Instance1>::BattleNotInQueueingPhase
			);
		});
	}

	#[test]
	fn try_perform_player_action_fails_with_defeated_player() {
		ExtBuilder::default().balances(&[(ALICE, 1000)]).build().execute_with(|| {
			let battle_duration = 500;
			let battle_max_players = 20;
			let battle_grid_size = Coordinates::new(20, 20);

			run_to_block(10);
			let battle_runs_until = 10 + (battle_duration + QUEUE_DURATION) as MockBlockNumber;

			start_battle_with_config(battle_duration, battle_max_players, battle_grid_size);
			queue_player(&ALICE, PlayerWeapon::Rock);

			// We advance 50 more blocks, this should put us in the Input phase.
			run_to_block(System::block_number() + 50);
			assert_battle_state_is_active_with(
				BattlePhase::Input,
				battle_max_players,
				battle_runs_until,
				battle_grid_size,
			);

			let _ = BattleRoyale::mark_account_as_defeated(&ALICE);

			let payload_fill = [241; 28];
			let action_base = PlayerAction::SwapWeapon(PlayerWeapon::Rock);
			let input_hash = action_base.generate_hash_for(payload_fill);

			// We fail to play a move since the player was not queued
			assert_noop!(
				BattleRoyale::try_perform_player_action(&ALICE, input_hash),
				Error::<Test, Instance1>::PlayerCannotPerformAction
			);
		});
	}

	#[test]
	fn try_perform_player_action_fails_reveal_with_incorrect_action() {
		ExtBuilder::default().balances(&[(ALICE, 1000)]).build().execute_with(|| {
			let battle_duration = 500;
			let battle_max_players = 20;
			let battle_grid_size = Coordinates::new(20, 20);

			run_to_block(10);
			let battle_runs_until = 10 + (battle_duration + QUEUE_DURATION) as MockBlockNumber;

			start_battle_with_config(battle_duration, battle_max_players, battle_grid_size);
			queue_player(&ALICE, PlayerWeapon::Rock);

			// We advance 50 more blocks, this should put us in the Input phase.
			run_to_block(System::block_number() + 50);
			assert_battle_state_is_active_with(
				BattlePhase::Input,
				battle_max_players,
				battle_runs_until,
				battle_grid_size,
			);

			let payload_fill = [177; 28];
			let action_base = PlayerAction::SwapWeapon(PlayerWeapon::Rock);
			let input_hash = action_base.generate_hash_for(payload_fill);

			assert_ok!(BattleRoyale::try_perform_player_action(&ALICE, input_hash));

			// We advance 3 more blocks, this should put us in the Reveal phase.
			run_to_block(System::block_number() + 3);
			assert_battle_state_is_active_with(
				BattlePhase::Reveal,
				battle_max_players,
				battle_runs_until,
				battle_grid_size,
			);

			// We fail to reveal action since the action is not the same as in the Input phase
			let incorrect_reveal = {
				let incorrect_action = PlayerAction::Move(Coordinates::new(8, 5));
				let mut reveal_hash = [0_u8; 32];

				reveal_hash[0..=27].copy_from_slice(&payload_fill);
				reveal_hash[28..].copy_from_slice(&incorrect_action.generate_payload_for());

				reveal_hash
			};
			assert_noop!(
				BattleRoyale::try_perform_player_action(&ALICE, incorrect_reveal),
				Error::<Test, Instance1>::PlayerRevealDoesntMatchOriginalAction
			);
		});
	}

	#[test]
	fn try_perform_player_action_fails_reveal_when_no_action_was_performed() {
		ExtBuilder::default().balances(&[(ALICE, 1000)]).build().execute_with(|| {
			let battle_duration = 500;
			let battle_max_players = 20;
			let battle_grid_size = Coordinates::new(20, 20);

			run_to_block(10);
			let battle_runs_until = 10 + (battle_duration + QUEUE_DURATION) as MockBlockNumber;

			start_battle_with_config(battle_duration, battle_max_players, battle_grid_size);
			queue_player(&ALICE, PlayerWeapon::Rock);

			// We advance 53 more blocks, this should put us in the Reveal phase.
			run_to_block(System::block_number() + 53);
			assert_battle_state_is_active_with(
				BattlePhase::Reveal,
				battle_max_players,
				battle_runs_until,
				battle_grid_size,
			);

			// We fail to reveal action since we didn't send any action during the Input phase
			assert_noop!(
				BattleRoyale::try_perform_player_action(&ALICE, [0; 32]),
				Error::<Test, Instance1>::PlayerDoesntHaveOriginalActionToReveal
			);
		});
	}
}
