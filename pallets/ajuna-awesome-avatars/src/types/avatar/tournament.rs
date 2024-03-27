use super::*;
use pallet_ajuna_tournament::traits::EntityRank;
use sp_std::cmp::Ordering;

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq, Eq)]
pub enum AvatarRankingCategory {
	#[default]
	MinSoulPoints,
	MaxSoulPoints,
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
					entity.1.souls.cmp(&other.1.souls).reverse(),
				AvatarRankingCategory::MaxSoulPoints => entity.1.souls.cmp(&other.1.souls),
			}
		}
	}
}
