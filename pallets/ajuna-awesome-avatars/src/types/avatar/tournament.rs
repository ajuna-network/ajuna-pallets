use super::*;
use pallet_ajuna_tournament::traits::EntityRank;
use sp_std::{cmp::Ordering, num::NonZeroU32};

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq, Eq)]
pub enum AvatarRankingCategory {
	#[default]
	MinSoulPoints,
	MaxSoulPoints,
	DnaAscending,
	DnaDescending,
	MinSoulPointsWithForce(Force),
	MaxSoulPointsWithForce(Force),
	MintedAtModulo(NonZeroU32),
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq, Eq)]
pub struct AvatarRanker<Id, BlockNumber> {
	pub category: AvatarRankingCategory,
	pub _marker: PhantomData<(Id, BlockNumber)>,
}

impl<Id, BlockNumber> EntityRank for AvatarRanker<Id, BlockNumber>
where
	Id: Member,
	BlockNumber: sp_runtime::traits::BlockNumber,
{
	type EntityId = Id;
	type Entity = Avatar<BlockNumber>;

	fn rank_against(
		&self,
		entity: (&Self::EntityId, &Self::Entity),
		other: (&Self::EntityId, &Self::Entity),
	) -> Ordering {
		if entity.0 == other.0 {
			Ordering::Equal
		} else {
			match self.category {
				AvatarRankingCategory::MinSoulPoints =>
					match entity.1.souls.cmp(&other.1.souls).reverse() {
						Ordering::Equal => Ordering::Less,
						ordering => ordering,
					},
				AvatarRankingCategory::MaxSoulPoints => match entity.1.souls.cmp(&other.1.souls) {
					Ordering::Equal => Ordering::Less,
					ordering => ordering,
				},
				AvatarRankingCategory::DnaAscending => entity.1.dna.cmp(&other.1.dna),
				AvatarRankingCategory::DnaDescending => entity.1.dna.cmp(&other.1.dna).reverse(),
				AvatarRankingCategory::MinSoulPointsWithForce(ref force) =>
					if entity.1.force() == force.as_byte() && entity.1.force() == other.1.force() {
						entity.1.souls.cmp(&other.1.souls).reverse()
					} else {
						// Returning Ordering::Equal makes that entity not get ranked in the table
						// in both the case the table is still empty, or when comparing to another
						// entity already in the ranks
						Ordering::Equal
					},
				AvatarRankingCategory::MaxSoulPointsWithForce(ref force) =>
					if entity.1.force() == force.as_byte() && entity.1.force() == other.1.force() {
						entity.1.souls.cmp(&other.1.souls)
					} else {
						// Returning Ordering::Equal makes that entity not get ranked in the table
						// in both the case the table is still empty, or when comparing to another
						// entity already in the ranks
						Ordering::Equal
					},
				AvatarRankingCategory::MintedAtModulo(modulo) => {
					let block_modulo = BlockNumber::from(u32::from(modulo));
					let entity_modulo = entity.1.minted_at % block_modulo;
					let other_modulo = other.1.minted_at % block_modulo;
					match entity_modulo.cmp(&other_modulo) {
						Ordering::Equal => Ordering::Less,
						ordering => ordering,
					}
				},
			}
		}
	}
}
