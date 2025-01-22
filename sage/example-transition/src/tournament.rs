use crate::asset::{Asset, AssetId};

use ajuna_primitives::tournament_manager::EntityRanker;

use frame_support::pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo};
use sp_std::{cmp::Ordering, marker::PhantomData};

pub type HeroJamTournamentCategoryId = u8;

pub type HeroJamTournamentConfig = ();

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, PartialEq, Eq)]
pub struct HeroJamEntityRanker<BlockNumber>(PhantomData<BlockNumber>);

impl<BlockNumber> EntityRanker for HeroJamEntityRanker<BlockNumber> {
	type EntityId = AssetId;
	type Entity = Asset<BlockNumber>;

	fn can_rank(&self, _entity: (&Self::EntityId, &Self::Entity)) -> bool {
		true
	}

	fn rank_against(
		&self,
		_entity: (&Self::EntityId, &Self::Entity),
		_other: (&Self::EntityId, &Self::Entity),
	) -> Ordering {
		Ordering::Equal
	}
}
