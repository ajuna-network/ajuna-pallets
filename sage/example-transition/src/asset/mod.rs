use sage_api::traits::GetId;

use frame_support::pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo};

pub mod hero_jam;

pub type AssetId = u32;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum AssetVariant<BlockNumber, Balance> {
	HeroJam(hero_jam::HeroJamAsset<BlockNumber, Balance>),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct Asset<BlockNumber, Balance> {
	pub asset_variant: AssetVariant<BlockNumber, Balance>,
}

impl<BlockNumber, Balance> From<hero_jam::HeroJamAsset<BlockNumber, Balance>>
	for Asset<BlockNumber, Balance>
{
	fn from(value: hero_jam::HeroJamAsset<BlockNumber, Balance>) -> Self {
		Self { asset_variant: AssetVariant::HeroJam(value) }
	}
}

impl<BlockNumber, Balance> GetId<AssetId> for Asset<BlockNumber, Balance> {
	fn get_id(&self) -> AssetId {
		match &self.asset_variant {
			AssetVariant::HeroJam(asset) => asset.get_id(),
		}
	}
}
