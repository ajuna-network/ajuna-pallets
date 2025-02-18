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
