use frame_support::pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum AssetType {
	None = 0,
	Hero = 1,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum AssetSubType {
	None = 0,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum StateType {
	None = 0,
	Sleep = 1,
	Idle = 2,
	Work = 3,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct HeroJamAsset<BlockNumber> {
	pub asset_type: AssetType,
	pub asset_subtype: AssetSubType,
	pub energy: u8,
	pub state_type: StateType,
	pub state_value: u8,
	pub state_change_block_number: BlockNumber,
}
