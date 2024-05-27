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

pub use fleet::*;

pub(crate) type CommanderId = u8;
pub(crate) type ShipId = u8;
pub(crate) type BoardPosition = i16;
pub(crate) type EngagementId = u32;
pub(crate) type LogId = u32;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct CommanderData {
	pub exp: u32,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct PlayerData {
	pub ranked_wins: u32,
	pub ranked_losses: u32,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum Move {
	Shoot { round: u8, from: ShipId, to: ShipId, damage: u32, target_position: BoardPosition },
	Reposition { round: u8, from: ShipId, target_position: BoardPosition },
}

/// Describes a single Ship on the board
#[derive(Debug, Copy, Clone, Eq, PartialEq, Default, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct Ship {
	/// Command Power (to calculate fleet weights)
	pub command_power: u16,
	/// Health Points of the ship
	pub hit_points: u16,
	/// Base attack
	pub attack_base: u16,
	/// Variable attack (subject to random)
	pub attack_variable: u16,
	/// Defence of the ship
	pub defence: u16,
	/// Speed, number of fields the ship can move in a round
	pub speed: u8,
	/// Range, number of fields in front of it the ship can shoot to in a round
	pub range: u8,
}

impl Ship {
	pub(crate) fn get_effective_defense(&self, variant: WingFormation) -> u16 {
		match variant {
			WingFormation::Neutral => self.defence,
			WingFormation::Defensive => self.defence.saturating_add(consts::FIT_TO_STAT),
			WingFormation::Offensive => self.defence.saturating_sub(consts::FIT_TO_STAT),
		}
	}

	pub(crate) fn get_effective_attack(&self, variant: WingFormation) -> u16 {
		match variant {
			WingFormation::Neutral => self.attack_base,
			WingFormation::Defensive => self.attack_base.saturating_sub(consts::FIT_TO_STAT),
			WingFormation::Offensive => self.attack_base.saturating_add(consts::FIT_TO_STAT),
		}
	}
}

pub mod fleet {
	use super::*;

	#[derive(Debug, Copy, Clone, Eq, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub enum WingFormation {
		Neutral = 0,
		Defensive = 1,
		Offensive = 2,
	}

	#[derive(Debug, Copy, Clone, Eq, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub struct FleetWing {
		pub(crate) ship_id: ShipId,
		pub(crate) formation: WingFormation,
		pub(crate) wing_size: u8,
	}

	impl FleetWing {
		pub fn new(ship_id: ShipId, formation: WingFormation, wing_size: u8) -> Self {
			Self { ship_id, formation, wing_size }
		}
	}

	#[derive(Debug, Copy, Clone, Eq, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo)]
	pub struct Fleet {
		pub composition: [FleetWing; consts::MAX_FLEET_WINGS],
		pub commander: CommanderId,
	}

	pub type FleetDamageReport = [(ShipId, u8); consts::MAX_FLEET_WINGS];
}

#[derive(Debug, Clone, Eq, PartialEq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct EngagementResult {
	/// Attacker fleet composition
	pub attacker_fleet: Fleet,
	/// Defender fleet composition
	pub defender_fleet: Fleet,
	/// Was the attacker defeated?
	pub attacker_defeated: bool,
	/// Was the defender defeated?
	pub defender_defeated: bool,
	/// Length of the engagement in rounds
	pub rounds: u8,
	/// Random seed the engagement was generated with
	pub seed: u64,
	/// Attackers ships lost
	pub attacker_damage_report: FleetDamageReport,
	/// Defenders ships lost
	pub defender_damage_report: FleetDamageReport,
}

pub type EngagementLog = BoundedVec<Move, ConstU32<{ consts::MAX_LOG_ENTRIES }>>;
