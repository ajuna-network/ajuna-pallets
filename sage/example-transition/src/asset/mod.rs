use frame_support::pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo};

pub mod hero_jam;

pub type AssetId = u64;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct Asset<BlockNumber> {
	pub asset_variant: AssetVariant<BlockNumber>,
}

impl<BlockNumber> From<hero_jam::HeroJamAsset<BlockNumber>> for Asset<BlockNumber> {
	fn from(value: hero_jam::HeroJamAsset<BlockNumber>) -> Self {
		Self { asset_variant: AssetVariant::HeroJam(value) }
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum AssetVariant<BlockNumber> {
	HeroJam(hero_jam::HeroJamAsset<BlockNumber>),
}
