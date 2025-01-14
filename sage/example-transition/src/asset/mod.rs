use sage_api::traits::GetId;

use frame_support::pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo};

pub mod hero_jam;

pub type AssetId = u32;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum AssetVariant<BlockNumber> {
	HeroJam(hero_jam::HeroJamAsset<BlockNumber>),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct Asset<BlockNumber> {
	pub asset_variant: AssetVariant<BlockNumber>,
}

impl<BlockNumber> From<hero_jam::HeroJamAsset<BlockNumber>> for Asset<BlockNumber> {
	fn from(value: hero_jam::HeroJamAsset<BlockNumber>) -> Self {
		Self { asset_variant: AssetVariant::HeroJam(value) }
	}
}

impl<BlockNumber> GetId<AssetId> for Asset<BlockNumber> {
	fn get_id(&self) -> AssetId {
		match &self.asset_variant {
			AssetVariant::HeroJam(asset) => asset.get_id(),
		}
	}
}
