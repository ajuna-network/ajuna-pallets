use crate::{
	asset::{Asset, AssetId},
	tournament::HeroJamTournamentCategoryId,
};
use sp_std::{marker::PhantomData, vec::Vec};

use crate::tournament::HeroJamEntityRanker;
use ajuna_primitives::{
	asset_manager::AssetInspector, tournament_manager::TournamentBenchmarkHelper,
};

pub struct GameTournamentBenchmarkHelper<AccountId, BlockNumber, AssetHandler>(
	PhantomData<(AccountId, BlockNumber, AssetHandler)>,
);

pub type HeroJamTournamentAsset<BlockNumber> = (AssetId, Asset<BlockNumber>);

impl<AccountId, BlockNumber, AssetHandler>
	TournamentBenchmarkHelper<
		HeroJamTournamentCategoryId,
		HeroJamEntityRanker<BlockNumber>,
		AccountId,
		HeroJamTournamentAsset<BlockNumber>,
	> for GameTournamentBenchmarkHelper<AccountId, BlockNumber, AssetHandler>
where
	AssetHandler: AssetInspector,
{
	fn create_category_id() -> HeroJamTournamentCategoryId {
		HeroJamTournamentCategoryId::default()
	}

	fn create_ranker() -> HeroJamEntityRanker<BlockNumber> {
		HeroJamEntityRanker(PhantomData)
	}

	fn create_entities(
		_owner: &AccountId,
		_count: usize,
	) -> Vec<HeroJamTournamentAsset<BlockNumber>> {
		todo!()
	}
}
