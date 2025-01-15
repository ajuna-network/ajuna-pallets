use crate::asset::hero_jam::{AssetType, HeroJamAsset};

use ajuna_primitives::trade_manager::*;

use sp_runtime::traits::BlockNumber as BlockNumberT;
use sp_std::marker::PhantomData;

pub(super) struct HeroJamFilter<BlockNumber>(PhantomData<BlockNumber>);

impl<BlockNumber> TradeManager for HeroJamFilter<BlockNumber>
where
	BlockNumber: BlockNumberT,
{
	type TradeFilter = AssetType;
	type Asset = HeroJamAsset<BlockNumber>;

	fn can_be_traded_using(asset: &Self::Asset, filter: &Self::TradeFilter) -> bool {
		asset.asset_type == *filter
	}
}

impl<BlockNumber> TransferManager for HeroJamFilter<BlockNumber>
where
	BlockNumber: BlockNumberT,
{
	type TransferFilter = AssetType;
	type Asset = HeroJamAsset<BlockNumber>;

	fn can_be_transferred_using(asset: &Self::Asset, filter: &Self::TransferFilter) -> bool {
		asset.asset_type == *filter
	}
}
