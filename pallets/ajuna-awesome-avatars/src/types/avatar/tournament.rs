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
pub struct AvatarRanker<T> {
	pub category: AvatarRankingCategory,
	pub _marker: PhantomData<T>,
}

impl<T> EntityRank for AvatarRanker<T>
where
	T: Member,
{
	type EntityId = T;
	type Entity = Avatar;

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
