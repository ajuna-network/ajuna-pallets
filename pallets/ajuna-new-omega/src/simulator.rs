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

use consts::{MAX_FLEET_WINGS, MAX_ROUNDS};

use sp_std::cmp::{max, min};

pub struct BattleSimulator<T: Config> {
	_marker: PhantomData<T>,
}

impl<T: Config> BattleSimulator<T> {
	fn generate_ship_store_for<const N: usize>(
		account: &AccountIdFor<T>,
		fleet_wings: &[FleetWing; N],
	) -> Result<[Ship; N], DispatchError> {
		let mut ships: [Ship; N] = [Default::default(); N];

		for (i, FleetWing { ship_id, .. }) in fleet_wings.iter().enumerate() {
			let ship = {
				let maybe_ship = Ships::<T>::get(account, ship_id);

				ensure!(maybe_ship.is_some(), Error::<T>::ShipNotFound);

				maybe_ship.unwrap()
			};

			ships[i] = ship;
		}

		Ok(ships)
	}

	fn is_dead(ship_hps: &[i32]) -> bool {
		ship_hps.iter().all(|hp| *hp <= 0)
	}

	fn get_target(
		wing_index: usize,
		ships_atk: &[Ship],
		ship_positions_atk: &[BoardPosition; MAX_FLEET_WINGS],
		ship_positions_def: &[BoardPosition; MAX_FLEET_WINGS],
		ship_hps_enemy: &[i32; MAX_FLEET_WINGS],
	) -> (bool, u8, u8) {
		let position = ship_positions_atk[wing_index];
		let mut proposed_move: u8 = 0;
		let mut min_distance_index: u8 = MAX_FLEET_WINGS as u8;

		for enemy_wing_index in (0..MAX_FLEET_WINGS as u8).rev() {
			let position_diff = position - ship_positions_def[enemy_wing_index as usize];
			let delta = position_diff.unsigned_abs() as u8;

			if (delta <= ships_atk[wing_index].range + ships_atk[wing_index].speed) &&
				ship_hps_enemy[enemy_wing_index as usize] > 0
			{
				// We have found a target
				min_distance_index = enemy_wing_index;
				// Do we need to move?
				if delta > ships_atk[wing_index].range {
					proposed_move = delta - ships_atk[wing_index].range;
				} else {
					proposed_move = 0;
				}

				break;
			}
		}

		(min_distance_index < (MAX_FLEET_WINGS as u8), min_distance_index, proposed_move)
	}

	fn get_number_of_ships_from_hp(wing_hp: u32, ship_hp: u16) -> u16 {
		if wing_hp % (ship_hp as u32) == 0 {
			(wing_hp / (ship_hp as u32)) as u16
		} else {
			(wing_hp / (ship_hp as u32)) as u16 + 1
		}
	}

	fn calculate_damage(
		variables: &[u16; MAX_FLEET_WINGS],
		atk_wing: &FleetWing,
		def_wing: &FleetWing,
		atk_hangar: &[Ship],
		def_hangar: &[Ship],
		atk_wing_index: usize,
		def_wing_index: usize,
		atk_wing_hp: u32,
	) -> u32 {
		let attack = atk_hangar[atk_wing_index].get_effective_attack(atk_wing.formation) +
			variables[atk_wing_index];
		let source_ships_count: u16 =
			Self::get_number_of_ships_from_hp(atk_wing_hp, atk_hangar[atk_wing_index].hit_points);
		let cap_damage =
			(source_ships_count as u32) * (def_hangar[def_wing_index].hit_points as u32);

		let defence = def_hangar[def_wing_index].get_effective_defense(def_wing.formation);
		let damage = attack.saturating_sub(defence) as u32 * (source_ships_count as u32);

		min(max(0, damage as i32), cap_damage as i32) as u32
	}

	fn log_shoot(
		round: u8,
		moves: &mut Vec<Move>,
		from: ShipId,
		to: ShipId,
		damage: u32,
		target_position: BoardPosition,
	) {
		moves.push(Move::Shoot { round, from, to, damage, target_position });
	}

	fn log_move(round: u8, moves: &mut Vec<Move>, from: u8, target_position: BoardPosition) {
		moves.push(Move::Reposition { round, from, target_position });
	}

	fn generate_damage_report_for(
		ships_lost: &[u8; MAX_FLEET_WINGS],
		fleet: &Fleet,
	) -> FleetDamageReport {
		ships_lost
			.iter()
			.enumerate()
			.map(|(i, ship_num)| (fleet.composition[i].ship_id, *ship_num))
			.collect::<Vec<_>>()
			.try_into()
			.unwrap()
	}

	pub(crate) fn simulate_engagement(
		attacker: &AccountIdFor<T>,
		attacker_fleet: Fleet,
		defender: &AccountIdFor<T>,
		defender_fleet: Fleet,
		seed: u64,
		log_moves: bool,
	) -> Result<(EngagementResult, Option<Vec<Move>>, Option<Vec<Move>>), DispatchError> {
		let atk_hangar = Self::generate_ship_store_for::<MAX_FLEET_WINGS>(
			attacker,
			&attacker_fleet.composition,
		)?;

		let def_hangar = Self::generate_ship_store_for::<MAX_FLEET_WINGS>(
			defender,
			&defender_fleet.composition,
		)?;

		// Starting ship positions for both sides
		let mut ship_pos_atk: [BoardPosition; MAX_FLEET_WINGS] = [10, 11, 12, 13];
		let mut ship_pos_def: [BoardPosition; MAX_FLEET_WINGS] = [-10, -11, -12, -13];
		// Current ship HPs, per ship type
		let mut ship_hp_atk: [i32; MAX_FLEET_WINGS] = [0; MAX_FLEET_WINGS];
		let mut ship_hp_def: [i32; MAX_FLEET_WINGS] = [0; MAX_FLEET_WINGS];
		// Precalculated variable damage coefficients
		let mut var_dmg_atk: [u16; MAX_FLEET_WINGS] = [0; MAX_FLEET_WINGS];
		let mut var_dmg_def: [u16; MAX_FLEET_WINGS] = [0; MAX_FLEET_WINGS];

		// Precalculate the variables and initialize the ship HPs
		for i in 0..MAX_FLEET_WINGS {
			ship_hp_atk[i] = (atk_hangar[i].hit_points as i32) *
				(attacker_fleet.composition[i].wing_size as i32);
			ship_hp_def[i] = (atk_hangar[i].hit_points as i32) *
				(defender_fleet.composition[i].wing_size as i32);
			var_dmg_atk[i] = (seed % def_hangar[i].attack_variable as u64) as u16;
			var_dmg_def[i] = ((seed / 2) % def_hangar[i].attack_variable as u64) as u16;
		}

		let mut atk_moves: Option<Vec<Move>> = log_moves.then(Vec::new);
		let mut def_moves: Option<Vec<Move>> = log_moves.then(Vec::new);
		let mut total_rounds: u8 = 0;

		for round in 0..MAX_ROUNDS as u8 {
			if Self::is_dead(&ship_hp_atk) || Self::is_dead(&ship_hp_def) {
				break;
			}

			total_rounds += 1;

			// Loop through all the ships
			for wing_index in 0..MAX_FLEET_WINGS {
				let mut atk_has_target: bool = false;
				let atk_wing_defeated = ship_hp_atk[wing_index] <= 0;
				let def_wing_defeated = ship_hp_def[wing_index] <= 0;
				let mut atk_damage: u32 = 0;
				let mut atk_target: u8 = 0;
				let mut atk_delta_move: u8 = 0;

				if !atk_wing_defeated {
					(atk_has_target, atk_target, atk_delta_move) = Self::get_target(
						wing_index,
						&atk_hangar,
						&ship_pos_atk,
						&ship_pos_def,
						&ship_hp_def,
					);

					if atk_has_target {
						atk_damage = Self::calculate_damage(
							&var_dmg_atk,
							&attacker_fleet.composition[wing_index],
							&defender_fleet.composition[wing_index],
							&atk_hangar,
							&def_hangar,
							wing_index,
							atk_target as usize,
							ship_hp_atk[wing_index] as u32,
						);

						if let Some(ref mut moves) = atk_moves {
							Self::log_shoot(
								round,
								moves,
								wing_index as u8,
								atk_target,
								atk_damage,
								ship_pos_atk[wing_index] - (atk_delta_move as BoardPosition),
							);
						}
					} else if let Some(ref mut moves) = atk_moves {
						Self::log_move(
							round,
							moves,
							wing_index as u8,
							ship_pos_atk[wing_index] -
								(atk_hangar[wing_index].speed as BoardPosition),
						);
					}
				}

				if !def_wing_defeated {
					let (def_has_target, def_target, def_delta_move) = Self::get_target(
						wing_index,
						&def_hangar,
						&ship_pos_def,
						&ship_pos_atk,
						&ship_hp_atk,
					);

					if def_has_target {
						let def_damage = Self::calculate_damage(
							&var_dmg_def,
							&defender_fleet.composition[wing_index],
							&attacker_fleet.composition[wing_index],
							&def_hangar,
							&atk_hangar,
							wing_index,
							def_target as usize,
							ship_hp_def[wing_index] as u32,
						);

						// Move the ships, apply the damage
						ship_hp_atk[def_target as usize] -= def_damage as i32;
						ship_pos_def[wing_index] += def_delta_move as BoardPosition;

						if let Some(ref mut moves) = def_moves {
							Self::log_shoot(
								round,
								moves,
								wing_index as u8,
								def_target,
								def_damage,
								ship_pos_def[wing_index],
							);
						}
					} else {
						// Move the ships
						ship_pos_def[wing_index] += def_hangar[wing_index].speed as BoardPosition;

						if let Some(ref mut moves) = def_moves {
							Self::log_move(
								round,
								moves,
								wing_index as u8,
								ship_pos_def[wing_index],
							);
						}
					}
				}

				if !atk_wing_defeated {
					if atk_has_target {
						// Move the ships, apply the damage
						ship_hp_def[atk_target as usize] -= atk_damage as i32;
						ship_pos_atk[wing_index] -= atk_delta_move as BoardPosition;
					} else {
						// Move the ships
						ship_pos_atk[wing_index] -= atk_hangar[wing_index].speed as BoardPosition;
					}
				}
			}
		}

		// Calculate ships lost according to HPs left
		let mut ships_lost_atk: [u8; MAX_FLEET_WINGS] = [0; MAX_FLEET_WINGS];
		let mut ships_lost_def: [u8; MAX_FLEET_WINGS] = [0; MAX_FLEET_WINGS];
		for i in 0..MAX_FLEET_WINGS {
			let safe_hp_atk: u32 = max(ship_hp_atk[i], 0) as u32;
			let safe_hp_def: u32 = max(ship_hp_def[i], 0) as u32;
			ships_lost_atk[i] = (((attacker_fleet.composition[i].wing_size as u32 *
				atk_hangar[i].hit_points as u32) -
				safe_hp_atk) / atk_hangar[i].hit_points as u32) as u8;
			ships_lost_def[i] = (((defender_fleet.composition[i].wing_size as u32 *
				def_hangar[i].hit_points as u32) -
				safe_hp_def) / def_hangar[i].hit_points as u32) as u8;
		}

		let mut total_def_ships: u16 = 0;
		for i in 0..MAX_FLEET_WINGS {
			total_def_ships += defender_fleet.composition[i].wing_size as u16;
		}

		let attacker_damage_report =
			Self::generate_damage_report_for(&ships_lost_atk, &attacker_fleet);
		let defender_damage_report =
			Self::generate_damage_report_for(&ships_lost_def, &defender_fleet);

		let result = EngagementResult {
			attacker_fleet,
			defender_fleet,
			attacker_defeated: total_def_ships > 0 && Self::is_dead(&ship_hp_atk),
			defender_defeated: Self::is_dead(&ship_hp_def),
			rounds: total_rounds,
			seed,
			attacker_damage_report,
			defender_damage_report,
		};

		Ok((result, atk_moves, def_moves))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::mock::{utils::*, *};

	#[test]
	fn test_fight_end_to_end() {
		ExtBuilder::default().build().execute_with(|| {
			prepare_hangar_for::<Test>(&ALICE);
			prepare_hangar_for::<Test>(&BOB);

			let seed: u64 = 1337;
			let log_moves: bool = true;

			let attacker_fleet = Fleet {
				composition: [
					FleetWing::new(0, WingFormation::Neutral, 20),
					FleetWing::new(1, WingFormation::Defensive, 20),
					FleetWing::new(2, WingFormation::Offensive, 20),
					FleetWing::new(3, WingFormation::Neutral, 20),
				],
				commander: 0,
			};

			let defender_fleet = Fleet {
				composition: [
					FleetWing::new(0, WingFormation::Defensive, 5),
					FleetWing::new(1, WingFormation::Neutral, 5),
					FleetWing::new(2, WingFormation::Defensive, 5),
					FleetWing::new(3, WingFormation::Offensive, 5),
				],
				commander: 1,
			};

			let (result, _, _) = BattleSimulator::<Test>::simulate_engagement(
				&ALICE,
				attacker_fleet,
				&BOB,
				defender_fleet,
				seed,
				log_moves,
			)
			.expect("Should simulate engagement successfully");

			assert!(result.defender_defeated);
		});
	}

	#[test]
	fn test_damage_calculation() {
		ExtBuilder::default().build().execute_with(|| {
			prepare_hangar_for::<Test>(&ALICE);
			prepare_hangar_for::<Test>(&BOB);

			let attacker_fleet = Fleet {
				composition: [
					FleetWing::new(0, WingFormation::Neutral, 10),
					FleetWing::new(1, WingFormation::Defensive, 10),
					FleetWing::new(2, WingFormation::Offensive, 10),
					FleetWing::new(3, WingFormation::Neutral, 10),
				],
				commander: 0,
			};

			let defender_fleet = Fleet {
				composition: [
					FleetWing::new(0, WingFormation::Offensive, 10),
					FleetWing::new(1, WingFormation::Neutral, 10),
					FleetWing::new(2, WingFormation::Defensive, 10),
					FleetWing::new(3, WingFormation::Offensive, 10),
				],
				commander: 1,
			};

			let atk_hangar = BattleSimulator::<Test>::generate_ship_store_for::<MAX_FLEET_WINGS>(
				&ALICE,
				&attacker_fleet.composition,
			)
			.expect("Should generated store");

			let def_hangar = BattleSimulator::<Test>::generate_ship_store_for::<MAX_FLEET_WINGS>(
				&BOB,
				&defender_fleet.composition,
			)
			.expect("Should generated store");

			let variables: [u16; MAX_FLEET_WINGS] = [0, 1, 2, 3];
			let source: usize = 0;
			let target: usize = 0;
			let source_hp = atk_hangar[source as usize].hit_points as u32;
			let damage = BattleSimulator::<Test>::calculate_damage(
				&variables,
				&attacker_fleet.composition[source],
				&defender_fleet.composition[target],
				&atk_hangar,
				&def_hangar,
				source,
				target,
				source_hp,
			);

			let source_hp_damaged = source_hp.saturating_sub(1);
			let damage_damaged = BattleSimulator::<Test>::calculate_damage(
				&variables,
				&attacker_fleet.composition[source],
				&defender_fleet.composition[target],
				&atk_hangar,
				&def_hangar,
				source,
				target,
				source_hp_damaged,
			);

			let source_hp_bigstack = source_hp * 32;
			let damage_bigstack = BattleSimulator::<Test>::calculate_damage(
				&variables,
				&attacker_fleet.composition[source],
				&defender_fleet.composition[target],
				&atk_hangar,
				&def_hangar,
				source,
				target,
				source_hp_bigstack,
			);

			assert_eq!(damage, 80);
			assert_eq!(damage_damaged, 80);
			assert_eq!(damage_bigstack, 80 * 32);
		});
	}

	#[test]
	fn test_is_dead() {
		ExtBuilder::default().build().execute_with(|| {
			let alive_ship_hps: [i32; MAX_FLEET_WINGS] = [20, -20, 0, 0];
			let is_dead_first = BattleSimulator::<Test>::is_dead(&alive_ship_hps);
			assert!(!is_dead_first);

			let dead_ship_hps: [i32; MAX_FLEET_WINGS] = [-100, -20, 0, 0];
			let is_dead_second = BattleSimulator::<Test>::is_dead(&dead_ship_hps);
			assert!(is_dead_second);
		});
	}
}
