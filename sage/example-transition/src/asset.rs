use crate::{error::*, transition::*};

use sage_api::{traits::GetId, TransitionError};

use frame_support::pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo};
use sp_runtime::traits::BlockNumber as BlockNumberT;

pub type AssetId = u32;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct Asset<BlockNumber> {
	pub id: AssetId,
	pub collection_id: u8,
	pub genesis: BlockNumber,
	pub variant: AssetVariant<BlockNumber>,
}

impl<BlockNumber> Asset<BlockNumber>
where
	BlockNumber: BlockNumberT,
{
	pub fn new_player(player_id: AssetId, genesis: BlockNumber) -> Self {
		Self {
			id: player_id,
			collection_id: ASSET_COLLECTION_ID,
			genesis,
			variant: AssetVariant::Player(PlayerVariant::Human(HumanVariant { seat_id: None })),
		}
	}

	pub fn new_tracker(player_id: AssetId, genesis: BlockNumber) -> Self {
		Self {
			id: player_id,
			collection_id: ASSET_COLLECTION_ID,
			genesis,
			variant: AssetVariant::Player(PlayerVariant::Tracker(TrackerVariant {
				slot_a_result: (0, 0),
				slot_b_result: (0, 0),
				slot_c_result: (0, 0),
				slot_d_result: (0, 0),
				last_reward: 0,
			})),
		}
	}

	pub fn new_bandit_machine(machine_id: AssetId, genesis: BlockNumber) -> Self {
		Asset::<BlockNumber> {
			id: machine_id,
			collection_id: ASSET_COLLECTION_ID,
			genesis,
			variant: AssetVariant::Machine(MachineVariant {
				seat_linked: 0,
				seat_limit: 1,
				value_1_factor: TokenType::T1,
				value_1_mul: MultiplierType::V1,
				value_2_factor: TokenType::T1,
				value_2_mul: MultiplierType::V0,
				value_3_factor: TokenType::T1,
				value_3_mul: MultiplierType::V0,
				sub_variant: MachineSubVariant::Bandit(BanditVariant {
					max_spins: BANDIT_MAX_SPINS,
					jackpot: 0,
				}),
			}),
		}
	}

	pub fn new_seat(
		seat_id: AssetId,
		genesis: BlockNumber,
		machine_id: AssetId,
		rent_duration: RentDuration,
	) -> Self {
		Asset::<BlockNumber> {
			id: seat_id,
			collection_id: ASSET_COLLECTION_ID,
			genesis,
			variant: AssetVariant::Seat(SeatVariant {
				rent_duration,
				player_fee: 1,
				player_grace_period: 30,
				reservation_start_block: 0_u32.into(),
				reservation_duration: ReservationDuration::None,
				last_action_block: 0,
				player_action_count: 0,
				player_id: None,
				machine_id: Some(machine_id),
			}),
		}
	}

	pub fn try_as_player(&mut self) -> Result<&mut PlayerVariant, TransitionError> {
		match &mut self.variant {
			AssetVariant::Player(player_variant) => Ok(player_variant),
			_ => Err(TransitionError::Transition { code: ASSET_VARIANT_IS_NOT_PLAYER }),
		}
	}

	pub fn try_as_machine(&mut self) -> Result<&mut MachineVariant, TransitionError> {
		match &mut self.variant {
			AssetVariant::Machine(machine_variant) => Ok(machine_variant),
			_ => Err(TransitionError::Transition { code: ASSET_VARIANT_IS_NOT_MACHINE }),
		}
	}

	pub fn try_as_seat(&mut self) -> Result<&mut SeatVariant<BlockNumber>, TransitionError> {
		match &mut self.variant {
			AssetVariant::Seat(seat_variant) => Ok(seat_variant),
			_ => Err(TransitionError::Transition { code: ASSET_VARIANT_IS_NOT_SEAT }),
		}
	}
}

impl<BlockNumber> GetId<AssetId> for Asset<BlockNumber> {
	fn get_id(&self) -> AssetId {
		self.id
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum VariantType {
	Player(PlayerType),
	Machine(MachineType),
	Seat,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum AssetVariant<BlockNumber> {
	Player(PlayerVariant),
	Machine(MachineVariant),
	Seat(SeatVariant<BlockNumber>),
}

impl<BlockNumber> AssetVariant<BlockNumber> {
	pub fn get_variant_type(&self) -> VariantType {
		match self {
			AssetVariant::Player(player) => match player {
				PlayerVariant::Human(_) => VariantType::Player(PlayerType::Human),
				PlayerVariant::Tracker(_) => VariantType::Player(PlayerType::Tracker),
			},
			AssetVariant::Machine(machine) => match machine.sub_variant {
				MachineSubVariant::Bandit(_) => VariantType::Machine(MachineType::Bandit),
			},
			AssetVariant::Seat(_) => VariantType::Seat,
		}
	}

	pub fn is_variant(&self, variant_type: VariantType) -> bool {
		self.get_variant_type() == variant_type
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum PlayerVariant {
	Human(HumanVariant),
	Tracker(TrackerVariant),
}

impl PlayerVariant {
	pub fn try_as_human(&mut self) -> Result<&mut HumanVariant, TransitionError> {
		match self {
			PlayerVariant::Human(human) => Ok(human),
			_ => Err(TransitionError::Transition { code: ASSET_VARIANT_IS_NOT_HUMAN }),
		}
	}

	pub fn try_as_tracker(&mut self) -> Result<&mut TrackerVariant, TransitionError> {
		match self {
			PlayerVariant::Tracker(tracker) => Ok(tracker),
			_ => Err(TransitionError::Transition { code: ASSET_VARIANT_IS_NOT_TRACKER }),
		}
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct HumanVariant {
	pub seat_id: Option<AssetId>,
}

impl HumanVariant {
	pub(crate) fn release(&mut self) {
		self.seat_id = None;
	}

	pub(crate) fn is_linked_to(&self, seat_id: AssetId) -> bool {
		if let Some(linked_id) = self.seat_id {
			linked_id == seat_id
		} else {
			false
		}
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct TrackerVariant {
	pub slot_a_result: (u16, u8),
	pub slot_b_result: (u16, u8),
	pub slot_c_result: (u16, u8),
	pub slot_d_result: (u16, u8),
	pub last_reward: u32,
}

impl TrackerVariant {
	pub(crate) fn clear(&mut self) {
		self.slot_a_result = (0, 0);
		self.slot_b_result = (0, 0);
		self.slot_c_result = (0, 0);
		self.slot_d_result = (0, 0);
		self.last_reward = 0;
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct MachineVariant {
	pub seat_linked: u8,
	pub seat_limit: u8,
	pub value_1_factor: TokenType,
	pub value_1_mul: MultiplierType,
	pub value_2_factor: TokenType,
	pub value_2_mul: MultiplierType,
	pub value_3_factor: TokenType,
	pub value_3_mul: MultiplierType,
	pub sub_variant: MachineSubVariant,
}

impl MachineVariant {
	pub fn try_as_bandit(&mut self) -> Result<&mut BanditVariant, TransitionError> {
		match &mut self.sub_variant {
			MachineSubVariant::Bandit(bandit) => Ok(bandit),
		}
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum MachineSubVariant {
	Bandit(BanditVariant),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct BanditVariant {
	pub max_spins: u8,
	pub jackpot: u32,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct SeatVariant<BlockNumber> {
	pub rent_duration: RentDuration,
	pub player_fee: u16,
	pub player_grace_period: u8,
	pub reservation_start_block: BlockNumber,
	pub reservation_duration: ReservationDuration,
	pub last_action_block: u16,
	pub player_action_count: u16,
	pub player_id: Option<AssetId>,
	pub machine_id: Option<AssetId>,
}

impl<BlockNumber> SeatVariant<BlockNumber>
where
	BlockNumber: BlockNumberT,
{
	pub(crate) fn release(&mut self) {
		self.player_id = None;
		self.reservation_start_block = 0_u32.into();
		self.reservation_duration = ReservationDuration::None;
		self.last_action_block = 0;
		self.player_action_count = 0;
	}

	pub(crate) fn is_linked_to(&self, player_id: AssetId) -> bool {
		if let Some(linked_id) = self.player_id {
			linked_id == player_id
		} else {
			false
		}
	}
}
