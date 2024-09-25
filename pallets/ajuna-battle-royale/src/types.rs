use frame_support::{
	pallet_prelude::{ConstU32, Decode, Encode, MaxEncodedLen, TypeInfo},
	BoundedBTreeSet,
};
use sp_runtime::{DispatchError, DispatchResult};
use sp_std::prelude::*;

const LOG_TARGET: &str = "runtime::ajuna-battle-royale";

pub(crate) const MIN_PLAYER_PER_BATTLE: u8 = 4;
pub(crate) const MAX_PLAYER_PER_BATTLE: u8 = 64;

pub(crate) const MIN_GRID_SIZE: u8 = 10;
pub(crate) const MAX_GRID_SIZE: u8 = 50;

/// Amount if blocks in which a game remains open for players to queue in
pub(crate) const QUEUE_DURATION: u32 = 50;
/// Minimum amount of blocks a battle may last
/// without including QUEUE_DURATION.
/// Each block is approximately **6s** in time
pub(crate) const MIN_BATTLE_DURATION: u32 = 50;

/// Input phase duration in blocks
pub(crate) const INPUT_PHASE_DURATION: u8 = 3;
/// Reveal phase duration in blocks
pub(crate) const REVEAL_PHASE_DURATION: u8 = 3;
/// Execution phase duration in blocks
pub(crate) const EXECUTION_PHASE_DURATION: u8 = 1;
/// Shrink phase duration in blocks
pub(crate) const SHRINK_PHASE_DURATION: u8 = 1;
/// Verification phase duration in blocks
pub(crate) const VERIFICATION_PHASE_DURATION: u8 = 1;

/// Amount of input phases to go through before
/// allowing a shrink phase
pub(crate) const SHRINK_PHASE_FREQUENCY: u8 = 3;

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Copy, Debug, PartialEq)]
pub enum SchedulerAction {
	Input(u8),
	Reveal,
	Execution,
	Shrink,
	Verify,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Copy, Debug, PartialEq)]
pub enum BattlePhase {
	Queueing,
	Input,
	Reveal,
	Execution,
	Shrink,
	Verification,
	Finished,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, PartialEq)]
pub struct BattleConfig<BlockNumber> {
	pub(crate) max_players: u8,
	pub(crate) run_until: BlockNumber,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Copy, Clone, Debug, PartialEq)]
pub struct Coordinates {
	pub x: u8,
	pub y: u8,
}

impl Coordinates {
	pub fn new(x: u8, y: u8) -> Self {
		Self { x, y }
	}
}

// Coordinates in GridBoundaries are 1-indexed not 0-indexed
// This is so we can use the 0,0 as the final coordinates when the wall shrinks.
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, PartialEq)]
pub struct GridBoundaries {
	pub(crate) top_left: Coordinates,
	pub(crate) down_right: Coordinates,
}

impl GridBoundaries {
	pub(crate) fn new(coordinates: Coordinates) -> Self {
		Self { top_left: Coordinates::new(1, 1), down_right: coordinates }
	}

	pub(crate) fn shrink(&mut self) {
		if self.down_right.x > self.down_right.y {
			self.down_right.x = self.down_right.x.saturating_sub(1);
		} else {
			self.down_right.y = self.down_right.y.saturating_sub(1);
		}
	}

	pub(crate) fn is_in_boundaries(&self, coordinates: &Coordinates) -> bool {
		coordinates.x >= self.top_left.x &&
			coordinates.x <= self.down_right.x &&
			coordinates.y >= self.top_left.y &&
			coordinates.y <= self.down_right.y
	}

	pub(crate) fn random_coordinates_in(&self, bytes: &[u8], seed: usize) -> Coordinates {
		let bytes_len = bytes.len();

		let x_idx = (seed + 1) % bytes_len;
		let y_idx = (seed + 3) % bytes_len;
		let seed_diff = (seed % 13) as u8;

		let x = (bytes[x_idx].saturating_add(seed_diff) % self.down_right.x) + self.top_left.x;
		let y = (bytes[y_idx].saturating_add(seed_diff) % self.down_right.y) + self.top_left.y;

		Coordinates::new(x, y)
	}
}

#[cfg(test)]
mod random_coordinates_test {
	use super::*;

	#[test]
	fn test_random_coordinate_consistency() {
		let boundaries = GridBoundaries::new(Coordinates::new(45, 35));

		let hash = [
			0x2E, 0x11, 0x3D, 0x0B, 0xFF, 0x33, 0x7A, 0x00, 0x5C, 0xAE, 0x86, 0x25, 0x09, 0x9E,
			0xA0, 0xBB,
		];

		for i in 0..20 {
			let seed = (i as usize) * 31;
			let coordinates = boundaries.random_coordinates_in(&hash, seed);

			assert!(boundaries.is_in_boundaries(&coordinates));
		}
	}
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, PartialEq)]
pub enum OccupancyState<AccountId> {
	Blocked,
	Open(BoundedBTreeSet<AccountId, ConstU32<{ MAX_PLAYER_PER_BATTLE as u32 }>>),
}

impl<AccountId> Default for OccupancyState<AccountId>
where
	AccountId: Ord,
{
	fn default() -> Self {
		Self::Open(BoundedBTreeSet::new())
	}
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
pub enum BattleState<BlockNumber> {
	#[default]
	Inactive,
	Active {
		phase: BattlePhase,
		config: BattleConfig<BlockNumber>,
		boundaries: GridBoundaries,
	},
}

impl<BlockNumber> BattleState<BlockNumber> {
	pub(crate) fn switch_to_phase(&mut self, new_phase: BattlePhase) {
		match self {
			BattleState::Inactive => {
				log::error!(target: LOG_TARGET, "Tried to change state on an Inactive BattleState!");
			},
			BattleState::Active { phase, .. } => {
				*phase = new_phase;
			},
		}
	}
}

pub type PlayerActionHash = [u8; 32];

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Copy, Clone, Debug, PartialEq)]
pub(crate) enum BattleResult {
	Win,
	Loss,
	Draw,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Copy, Clone, Debug, PartialEq)]
pub enum PlayerWeapon {
	Rock,
	Paper,
	Scissors,
}

impl TryFrom<u8> for PlayerWeapon {
	type Error = ();

	fn try_from(value: u8) -> Result<Self, Self::Error> {
		match value {
			0 => Ok(Self::Rock),
			1 => Ok(Self::Paper),
			2 => Ok(Self::Scissors),
			_ => Err(()),
		}
	}
}

impl PlayerWeapon {
	pub(crate) fn as_byte(&self) -> u8 {
		match self {
			PlayerWeapon::Rock => 0x00,
			PlayerWeapon::Paper => 0x01,
			PlayerWeapon::Scissors => 0x02,
		}
	}

	pub(crate) fn battle_against(&self, other: PlayerWeapon) -> BattleResult {
		match self {
			PlayerWeapon::Rock => match other {
				PlayerWeapon::Rock => BattleResult::Draw,
				PlayerWeapon::Paper => BattleResult::Loss,
				PlayerWeapon::Scissors => BattleResult::Win,
			},
			PlayerWeapon::Paper => match other {
				PlayerWeapon::Rock => BattleResult::Win,
				PlayerWeapon::Paper => BattleResult::Draw,
				PlayerWeapon::Scissors => BattleResult::Loss,
			},
			PlayerWeapon::Scissors => match other {
				PlayerWeapon::Rock => BattleResult::Loss,
				PlayerWeapon::Paper => BattleResult::Win,
				PlayerWeapon::Scissors => BattleResult::Draw,
			},
		}
	}
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Copy, Clone, Debug, PartialEq)]
pub enum PlayerAction {
	Move(Coordinates),
	SwapWeapon(PlayerWeapon),
	MoveAndSwap(Coordinates, PlayerWeapon),
}

impl PlayerAction {
	#[inline]
	pub fn generate_payload_for(&self) -> [u8; 4] {
		match self {
			PlayerAction::Move(coordinates) => [0x00, coordinates.x, coordinates.y, 0x00],
			PlayerAction::SwapWeapon(weapon) => [0x01, weapon.as_byte(), 0x00, 0x00],
			PlayerAction::MoveAndSwap(coordinates, weapon) =>
				[0x02, coordinates.x, coordinates.y, weapon.as_byte()],
		}
	}

	#[inline]
	pub fn generate_full_payload_for(&self, fill: [u8; 28]) -> PlayerActionHash {
		let mut hash = [0_u8; 32];

		let payload = self.generate_payload_for();

		hash[0..=27].copy_from_slice(&fill);
		hash[28..].copy_from_slice(&payload);

		hash
	}

	#[inline]
	pub fn generate_hash_for(&self, fill: [u8; 28]) -> PlayerActionHash {
		sp_crypto_hashing::blake2_256(&self.generate_full_payload_for(fill))
	}

	#[inline]
	pub fn try_decode_from_payload(action_payload: [u8; 4]) -> Option<Self> {
		match action_payload[0] {
			0 => Some(Self::Move(Coordinates::new(action_payload[1], action_payload[2]))),
			1 => PlayerWeapon::try_from(action_payload[1]).ok().map(Self::SwapWeapon),
			2 => PlayerWeapon::try_from(action_payload[3]).ok().map(|weapon| {
				Self::MoveAndSwap(Coordinates::new(action_payload[1], action_payload[2]), weapon)
			}),
			_ => None,
		}
	}
}

#[cfg(test)]
mod hash_tests {
	use super::*;

	#[test]
	fn test_payload_consistency() {
		let test_move = PlayerAction::Move(Coordinates::new(5, 7));
		assert_eq!(test_move.generate_payload_for(), [0x00, 0x05, 0x07, 0x00]);

		let test_weapon_swap = PlayerAction::SwapWeapon(PlayerWeapon::Scissors);
		assert_eq!(test_weapon_swap.generate_payload_for(), [0x01, 0x02, 0x00, 0x00]);

		let test_move_and_swap =
			PlayerAction::MoveAndSwap(Coordinates::new(17, 9), PlayerWeapon::Paper);
		assert_eq!(test_move_and_swap.generate_payload_for(), [0x02, 0x11, 0x09, 0x01]);
	}
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, PartialEq)]
pub struct PlayerData {
	pub(crate) position: Coordinates,
	pub(crate) weapon: PlayerWeapon,
	pub(crate) state: PlayerState,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
pub enum PlayerState {
	#[default]
	Inactive,
	PerformedAction(PlayerActionHash),
	RevealedAction,
	Defeated,
}

pub trait BattleProvider<AccountId> {
	fn try_start_battle(
		game_duration: u32,
		max_players: u8,
		grid_size: Coordinates,
		blocked_cells: Vec<Coordinates>,
	) -> DispatchResult;

	fn try_finish_battle() -> Result<Vec<AccountId>, DispatchError>;

	fn try_queue_player(account: &AccountId, initial_weapon: PlayerWeapon) -> DispatchResult;

	fn try_perform_player_action(account: &AccountId, action: PlayerActionHash) -> DispatchResult;
}
