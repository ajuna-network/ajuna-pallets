use crate::asset::{Asset, AssetId, AssetVariant};

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
pub struct HeroJamAsset<BlockNumber, Balance> {
	pub id: AssetId,
	pub asset_type: AssetType,
	pub asset_subtype: AssetSubType,
	pub energy: u8,
	pub fatigue: u8,
	pub state_type: StateType,
	pub state_sub_type: u8,
	pub state_sub_value: u8,
	pub state_change_block_number: BlockNumber,
	pub balance: Balance,
}

impl<BlockNumber, Balance> GetId<AssetId> for HeroJamAsset<BlockNumber, Balance> {
	fn get_id(&self) -> AssetId {
		self.id
	}
}

impl<BlockNumber, Balance> TryFrom<Asset<BlockNumber, Balance>>
	for HeroJamAsset<BlockNumber, Balance>
{
	type Error = ();

	fn try_from(value: Asset<BlockNumber, Balance>) -> Result<Self, Self::Error> {
		match value.asset_variant {
			AssetVariant::HeroJam(hero_jam_asset) => Ok(hero_jam_asset),
		}
	}
}
