use crate::{asset, asset::hero_jam, rules::RuleVerifier};

use ajuna_primitives::{
	asset_manager::{AssetInspector, AssetManager},
	chain_inspector::ChainInspector,
};
use sage_api::Error;

use crate::asset::AssetVariant;
use frame_support::pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo};
use parity_scale_codec::Codec;
use sp_runtime::traits::{BlockNumber as BlockNumberT, Member};
use std::marker::PhantomData;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum Rule<AccountId, AssetHandler, ChainHandler> {
	AllAssetType(hero_jam::AssetType),
	AllStateType(hero_jam::StateType),
	CanStateChange(PhantomData<ChainHandler>),
	AccountHasAssetOfType(hero_jam::AssetType, PhantomData<(AccountId, AssetHandler)>),
}

impl<AccountId, BlockNumber, AssetHandler, ChainHandler> RuleVerifier
	for Rule<AccountId, AssetHandler, ChainHandler>
where
	AccountId: Member + Codec,
	BlockNumber: BlockNumberT,
	AssetHandler: AssetManager<
			AccountId = AccountId,
			AssetId = asset::AssetId,
			Asset = asset::Asset<BlockNumber>,
		> + AssetInspector<
			AccountId = AccountId,
			AssetId = asset::AssetId,
			Asset = asset::Asset<BlockNumber>,
		>,
	ChainHandler: ChainInspector<BlockNumber = BlockNumber>,
{
	type AccountId = AccountId;
	type AssetId = asset::AssetId;

	fn verify(
		&self,
		account_id: &Self::AccountId,
		asset_ids: &[Self::AssetId],
	) -> Result<(), Error> {
		let filter_result = match self {
			Rule::AllAssetType(expected_asset_type) => asset_ids.iter().all(|asset_id| {
				if let Ok(asset) = AssetHandler::get_asset(asset_id) {
					match asset.asset_variant {
						AssetVariant::HeroJam(hero_jam_asset) =>
							&hero_jam_asset.asset_type == expected_asset_type,
					}
				} else {
					false
				}
			}),
			Rule::AllStateType(expected_state_type) => asset_ids.iter().all(|asset_id| {
				if let Ok(asset) = AssetHandler::get_asset(asset_id) {
					match asset.asset_variant {
						AssetVariant::HeroJam(hero_jam_asset) =>
							&hero_jam_asset.state_type == expected_state_type,
					}
				} else {
					false
				}
			}),
			Rule::CanStateChange(_) => asset_ids.iter().all(|asset_id| {
				if let Ok(asset) = AssetHandler::get_asset(asset_id) {
					match asset.asset_variant {
						AssetVariant::HeroJam(hero_jam_asset) =>
							hero_jam_asset.state_change_block_number <
								ChainHandler::get_current_block_number(),
					}
				} else {
					false
				}
			}),
			Rule::AccountHasAssetOfType(expected_asset_type, _) =>
				AssetHandler::iter_assets_from(account_id)
					.filter(|asset| match asset.asset_variant {
						AssetVariant::HeroJam(hero_jam_asset) =>
							&hero_jam_asset.asset_type == expected_asset_type,
					})
					.count()
					.eq(&1),
		};

		filter_result.then_some(()).ok_or(Error::FeeError)
	}
}
