use std::marker::PhantomData;
use crate::{
	asset::{Asset, AssetId},
	tournament::{HeroJamTournamentCategoryId, HeroJamTournamentConfig},
};

use ajuna_primitives::{
	asset_manager::AssetInspector, tournament_manager::TournamentBenchmarkHelper,
};

pub struct GameTournamentBenchmarkHelper<AccountId, BlockNumber, AssetHandler>(PhantomData<(AccountId, BlockNumber, AssetHandler)>);

pub type HeroJamTournamentAsset<BlockNumber> = (AssetId, Asset<BlockNumber>);

impl<AccountId, BlockNumber, AssetHandler>
	TournamentBenchmarkHelper<
		HeroJamTournamentCategoryId,
		HeroJamTournamentConfig,
		AccountId,
		HeroJamTournamentAsset<BlockNumber>,
	> for GameTournamentBenchmarkHelper<AccountId, BlockNumber, AssetHandler>
where
	AssetHandler: AssetInspector,
{
	fn create_category_id() -> HeroJamTournamentCategoryId {
		HeroJamTournamentCategoryId::default()
	}

	fn create_config() -> HeroJamTournamentConfig {}

	fn create_entities(
		_owner: &AccountId,
		_count: usize,
	) -> Vec<HeroJamTournamentAsset<BlockNumber>> {
		todo!()
	}
}
