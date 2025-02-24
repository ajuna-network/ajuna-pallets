use crate::transition::utils::{BASE_RESERVATION_TIME, BLOCKS_PER_DAY};
use frame_support::pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum PlayerType {
	Human,
	Tracker,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum MachineType {
	Bandit = 1,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum TokenType {
	#[default]
	T1 = 0,
	T10 = 1,
	T100 = 2,
	T1000 = 3,
	T10000 = 4,
	T100000 = 5,
	T1000000 = 6,
}

impl TokenType {
	pub fn as_value(&self) -> u32 {
		10_u32.pow(*self as u32)
	}

	pub fn get_value_for(&self, amount: MultiplierType) -> u32 {
		self.as_value().saturating_mul(amount as u32)
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum RentDuration {
	None = 0,
	Day1 = 1,
	Days2 = 2,
	Days3 = 3,
	Days5 = 4,
	Days7 = 5,
	Days14 = 6,
	Days28 = 7,
	Days56 = 8,
	Days112 = 9,
}

impl RentDuration {
	pub(crate) fn get_rent_duration_blocks(&self) -> u32 {
		let multiplier = match self {
			RentDuration::None => 0,
			RentDuration::Day1 => 1,
			RentDuration::Days2 => 2,
			RentDuration::Days3 => 3,
			RentDuration::Days5 => 5,
			RentDuration::Days7 => 7,
			RentDuration::Days14 => 14,
			RentDuration::Days28 => 28,
			RentDuration::Days56 => 56,
			RentDuration::Days112 => 112,
		};

		multiplier * BLOCKS_PER_DAY
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum ReservationDuration {
	None = 0,
	Mins5 = 1,
	Mins10 = 2,
	Mins15 = 3,
	Mins30 = 4,
	Mins45 = 5,
	Hour1 = 6,
	Hours2 = 7,
	Hours3 = 8,
	Hours4 = 9,
	Hours6 = 10,
	Hours8 = 11,
	Hours12 = 12,
}

impl ReservationDuration {
	pub(crate) fn get_reservation_duration_blocks(&self) -> u32 {
		let multiplier = match self {
			ReservationDuration::None => 0,
			ReservationDuration::Mins5 => 1,
			ReservationDuration::Mins10 => 2,
			ReservationDuration::Mins15 => 3,
			ReservationDuration::Mins30 => 6,
			ReservationDuration::Mins45 => 9,
			ReservationDuration::Hour1 => 12,
			ReservationDuration::Hours2 => 24,
			ReservationDuration::Hours3 => 36,
			ReservationDuration::Hours4 => 48,
			ReservationDuration::Hours6 => 72,
			ReservationDuration::Hours8 => 96,
			ReservationDuration::Hours12 => 144,
		};

		multiplier * BASE_RESERVATION_TIME
	}

	pub(crate) fn get_reservation_duration_fees(&self, player_fee: u32) -> u32 {
		player_fee.saturating_mul(*self as u32)
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum MultiplierType {
	#[default]
	V0 = 0,
	V1 = 1,
	V2 = 2,
	V3 = 3,
	V4 = 4,
	V5 = 5,
	V6 = 6,
	V7 = 7,
	V8 = 8,
	V9 = 9,
}

impl MultiplierType {
	pub fn as_value(&self) -> u32 {
		*self as u32
	}

	pub fn as_seat_validity_period(&self) -> u16 {
		(*self as u16).saturating_mul(600)
	}

	pub fn as_reservation_duration(&self) -> u16 {
		(*self as u16).saturating_mul(30)
	}
}
