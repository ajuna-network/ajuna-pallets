use frame_support::pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo};
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
/// Each block is approximately **6s** in time
pub(crate) const MIN_BATTLE_DURATION: u32 = 300;

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

pub type PlayerSecretAction = [u8; 32];
pub type PlayerSecret = u32;

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Copy, Clone, Debug, PartialEq)]
pub enum PlayerWeapon {
	Rock,
	Paper,
	Scissors,
}

impl PlayerWeapon {
	pub(crate) fn as_byte(&self) -> u8 {
		match self {
			PlayerWeapon::Rock => 0x00,
			PlayerWeapon::Paper => 0x01,
			PlayerWeapon::Scissors => 0x02,
		}
	}

	pub(crate) fn battle_against(&self, other: PlayerWeapon) -> bool {
		match self {
			PlayerWeapon::Rock => other != PlayerWeapon::Paper,
			PlayerWeapon::Paper => other != PlayerWeapon::Scissors,
			PlayerWeapon::Scissors => other != PlayerWeapon::Rock,
		}
	}
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Copy, Clone, Debug, PartialEq)]
pub enum PlayerAction {
	Input(PlayerSecretAction, PlayerSecret),
	Reveal(ActionReveal),
}

#[cfg(test)]
impl PlayerAction {
	pub(crate) fn get_input_details(&self) -> (PlayerSecretAction, PlayerSecret) {
		match self {
			PlayerAction::Input(action, secret) => (*action, *secret),
			PlayerAction::Reveal(_) => panic!("PlayerAction not in Input state!"),
		}
	}
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Copy, Clone, Debug, PartialEq)]
pub enum ActionReveal {
	Move(Coordinates),
	SwapWeapon(PlayerWeapon),
	MoveAndSwap(Coordinates, PlayerWeapon),
}

impl ActionReveal {
	#[inline]
	pub fn generate_hash_for(&self) -> Vec<u8> {
		let mut bytes = Vec::new();

		match self {
			ActionReveal::Move(coordinates) => {
				bytes.extend(&[0x00, coordinates.x, coordinates.y]);
			},
			ActionReveal::SwapWeapon(weapon) => {
				bytes.extend(&[0x01, weapon.as_byte()]);
			},
			ActionReveal::MoveAndSwap(coordinates, weapon) => {
				bytes.extend(&[0x02, coordinates.x, coordinates.y, weapon.as_byte()]);
			},
		}

		bytes
	}

	#[inline]
	pub fn generate_secret_with(&self, secret: PlayerSecret) -> PlayerSecretAction {
		let mut bytes = secret.to_le_bytes().to_vec();
		bytes.extend(self.generate_hash_for());
		sp_crypto_hashing::blake2_256(&bytes)
	}

	#[inline]
	pub fn compare_with(&self, secret: PlayerSecret, other_secret: &PlayerSecretAction) -> bool {
		self.generate_secret_with(secret) == *other_secret
	}
}

#[cfg(test)]
mod hash_tests {
	use super::*;

	#[test]
	fn test_hash_consistency() {
		let test_move = ActionReveal::Move(Coordinates::new(5, 7));
		assert_eq!(test_move.generate_hash_for(), vec![0x00, 0x05, 0x07]);

		let test_weapon_swap = ActionReveal::SwapWeapon(PlayerWeapon::Scissors);
		assert_eq!(test_weapon_swap.generate_hash_for(), vec![0x01, 0x02]);

		let test_move_and_swap =
			ActionReveal::MoveAndSwap(Coordinates::new(17, 9), PlayerWeapon::Paper);
		assert_eq!(test_move_and_swap.generate_hash_for(), vec![0x02, 0x11, 0x09, 0x01]);
	}

	#[test]
	fn test_secret_consistency() {
		let test_move = ActionReveal::Move(Coordinates::new(11, 3));
		let secret = 73;
		assert_eq!(
			test_move.generate_secret_with(secret),
			[
				0x3B, 0xE4, 0x64, 0x76, 0x37, 0xA4, 0x3F, 0x91, 0xCC, 0x21, 0x52, 0x9A, 0xC9, 0x02,
				0xC1, 0x72, 0xF6, 0x28, 0xD7, 0x7B, 0x5D, 0xD9, 0x63, 0x91, 0x3A, 0x4C, 0xB8, 0x2C,
				0x44, 0xB3, 0x0B, 0x0E
			]
		);

		let test_weapon_swap = ActionReveal::SwapWeapon(PlayerWeapon::Rock);
		let secret = 583;
		assert_eq!(
			test_weapon_swap.generate_secret_with(secret),
			[
				0xB6, 0x34, 0xEC, 0x2B, 0xC3, 0x25, 0x45, 0xE3, 0x48, 0x95, 0xBB, 0x95, 0x06, 0x48,
				0x16, 0x2A, 0x2D, 0xAE, 0x1D, 0x1D, 0xF3, 0x2F, 0xA2, 0x12, 0x62, 0xB0, 0xFF, 0xCC,
				0x23, 0xE1, 0x04, 0x74
			]
		);

		let test_move_and_swap =
			ActionReveal::MoveAndSwap(Coordinates::new(1, 22), PlayerWeapon::Paper);
		let secret = 1;
		assert_eq!(
			test_move_and_swap.generate_secret_with(secret),
			[
				0x29, 0x13, 0xAD, 0x42, 0x3D, 0x5A, 0x33, 0xDA, 0xD2, 0x78, 0xD0, 0xFE, 0x5B, 0xB1,
				0xC1, 0x03, 0x43, 0x14, 0x1E, 0x1A, 0x5D, 0xE7, 0x9D, 0x8D, 0x98, 0xD1, 0x7E, 0x34,
				0x37, 0xBA, 0x73, 0x76
			]
		);
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
	PerformedAction(PlayerSecretAction, PlayerSecret),
	RevealedAction,
	Defeated,
}

pub trait BattleProvider<AccountId> {
	fn try_start_battle(
		game_duration: u32,
		max_players: u8,
		grid_size: Coordinates,
	) -> DispatchResult;

	fn try_finish_battle() -> Result<Vec<AccountId>, DispatchError>;

	fn try_queue_player(
		account: &AccountId,
		initial_position: Coordinates,
		initial_weapon: PlayerWeapon,
	) -> DispatchResult;

	fn try_perform_player_action(account: &AccountId, action: PlayerAction) -> DispatchResult;
}
