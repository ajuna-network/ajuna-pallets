use crate::asset::AssetId;

use sage_api::traits::GetId;

use frame_support::pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo, Default)]
pub enum AssetType {
	#[default]
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
	Work = 2,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct HeroJamAsset<BlockNumber> {
	pub asset_type: AssetType,
	pub asset_subtype: AssetSubType,
	pub energy: u8,
	pub fatigue: u8,
	pub state_type: StateType,
	pub state_sub_type: u8,
	pub state_sub_value: u8,
	pub state_change_block_number: BlockNumber,
	// TODO: Use proper balance type in final version
	pub balance: u32,
}

impl<BlockNumber> GetId<AssetId> for HeroJamAsset<BlockNumber> {
	fn get_id(&self) -> AssetId {
		todo!()
	}
}
