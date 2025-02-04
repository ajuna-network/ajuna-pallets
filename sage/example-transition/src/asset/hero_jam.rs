use crate::asset::{Asset, AssetId, AssetVariant};

use sage_api::traits::GetId;

use frame_support::pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum StateType {
	None = 0,
	Sleep = 1,
	Work = 2,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum VariantType {
	Hero,
	Item,
	Map,
	Disassemblable,
	Consumable,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct HeroJamAsset<BlockNumber> {
	pub id: AssetId,
	pub collection_id: u8,
	pub score: u32,
	pub genesis: BlockNumber,
	pub balance: u32,
	pub variant: HeroJamVariant<BlockNumber>,
}

impl<BlockNumber> HeroJamAsset<BlockNumber> {
	pub fn is_variant(&self, variant: VariantType) -> bool {
		self.variant.get_variant() == variant
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum HeroJamVariant<BlockNumber> {
	Hero(HeroVariant<BlockNumber>),
	Item(ItemVariant),
	Map(MapVariant),
	Disassemblable(DisassemblableVariant<BlockNumber>),
	Consumable(ConsumableVariant),
}

impl<BlockNumber> HeroJamVariant<BlockNumber> {
	pub fn get_variant(&self) -> VariantType {
		match self {
			HeroJamVariant::Hero(_) => VariantType::Hero,
			HeroJamVariant::Item(_) => VariantType::Item,
			HeroJamVariant::Map(_) => VariantType::Map,
			HeroJamVariant::Disassemblable(_) => VariantType::Disassemblable,
			HeroJamVariant::Consumable(_) => VariantType::Consumable,
		}
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct HeroVariant<BlockNumber> {
	pub location_x: u8,
	pub location_y: u8,
	pub energy: u8,
	pub fatigue: u8,
	pub state_type: StateType,
	pub state_sub_type: u8,
	pub state_sub_value: u8,
	pub state_change_block_number: BlockNumber,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct ItemVariant;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct MapVariant {
	pub target_x: u8,
	pub target_y: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct DisassemblableVariant<BlockNumber> {
	pub result: HeroJamVariant<BlockNumber>,
	pub amount: u8,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct ConsumableVariant {
	pub affected_state: u8,
	pub effect_value: i8,
}

impl<BlockNumber> GetId<AssetId> for HeroJamAsset<BlockNumber> {
	fn get_id(&self) -> AssetId {
		self.id
	}
}

impl<BlockNumber> TryFrom<Asset<BlockNumber>> for HeroJamAsset<BlockNumber> {
	type Error = ();

	fn try_from(value: Asset<BlockNumber>) -> Result<Self, Self::Error> {
		match value.asset_variant {
			AssetVariant::HeroJam(hero_jam_asset) => Ok(hero_jam_asset),
		}
	}
}
