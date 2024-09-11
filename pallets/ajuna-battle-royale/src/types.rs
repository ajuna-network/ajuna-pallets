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

pub type PlayerSecretAction = [u8; 8];

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Copy, Clone, Debug, PartialEq)]
pub enum PlayerWeapon {
	Rock,
	Paper,
	Scissors,
}

impl PlayerWeapon {
	pub(crate) fn as_byte(&self) -> u8 {
		match self {
			PlayerWeapon::Rock => 0b0001,
			PlayerWeapon::Paper => 0b0010,
			PlayerWeapon::Scissors => 0b0100,
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
	Move(Coordinates),
	SwapWeapon(PlayerWeapon),
	MoveAndSwap(Coordinates, PlayerWeapon),
}

impl PlayerAction {
	pub fn generate_secret_with(&self, seed: u32) -> PlayerSecretAction {
		let mut bytes = Vec::new();

		match self {
			PlayerAction::Move(coordinates) => {
				bytes.extend(&[coordinates.x, coordinates.y]);
			},
			PlayerAction::SwapWeapon(weapon) => {
				bytes.extend(&[weapon.as_byte()]);
			},
			PlayerAction::MoveAndSwap(coordinates, weapon) => {
				bytes.extend(&[coordinates.x, coordinates.y, weapon.as_byte()]);
			},
		}

		bytes.extend(seed.to_ne_bytes());
		sp_crypto_hashing::twox_64(&bytes)
	}

	pub fn compare_with(&self, seed: u32, other_secret: &PlayerSecretAction) -> bool {
		self.generate_secret_with(seed) == *other_secret
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
	PerformedAction(PlayerSecretAction),
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

	fn try_perform_player_action(
		account: &AccountId,
		action: PlayerAction,
		player_seed: u32,
	) -> DispatchResult;
}
