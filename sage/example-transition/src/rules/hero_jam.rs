use crate::{asset, asset::hero_jam, rules::RuleVerifier};
use frame_support::pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo};

use ajuna_primitives::{
	asset_manager::{AssetInspector, AssetManager},
	chain_inspector::ChainInspector,
};
use sage_api::Error;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum Rule<AssetHandler, ChainHandler> {
	AllAssetType(hero_jam::AssetType),
	AllStateType(hero_jam::StateType),
	CanStateChange,
	AccountHasAssetOfType(hero_jam::AssetType),
}

impl<AccountId, BlockNumber, AssetHandler, ChainHandler> RuleVerifier
	for Rule<AssetHandler, ChainHandler>
where
	AssetHandler: AssetManager<
			AccountId = AccountId,
			AssetId = asset::AssetId,
			Asset = hero_jam::HeroJamAsset<BlockNumber>,
		> + AssetInspector<
			AccountId = AccountId,
			AssetId = asset::AssetId,
			Asset = hero_jam::HeroJamAsset<BlockNumber>,
		>,
	ChainHandler: ChainInspector<BlockNumber = BlockNumber>,
{
	type AccountId = AccountId;
	type Asset = hero_jam::HeroJamAsset<BlockNumber>;

	fn verify(&self, account_id: &Self::AccountId, assets: &[Self::Asset]) -> Result<(), Error> {
		let filter_result = match self {
			Rule::AllAssetType(expected_asset_type) =>
				assets.iter().all(|asset| &asset.asset_type == expected_asset_type),
			Rule::AllStateType(expected_state_type) =>
				assets.iter().all(|asset| &asset.state_type == expected_state_type),
			Rule::CanStateChange => assets.iter().all(|asset| {
				asset.state_change_block_number < ChainHandler::get_current_block_number()
			}),
			Rule::AccountHasAssetOfType(expected_asset_type) =>
				AssetHandler::iter_assets_from(account_id)
					.iter()
					.filter(|asset| &asset.asset_type == expected_asset_type)
					.count()
					.eq(&1),
		};

		filter_result.then_some(()).ok_or(Error::FeeError)
	}
}
