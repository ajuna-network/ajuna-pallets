use crate::asset::hero_jam::{AssetType, HeroJamAsset};
use ajuna_primitives::trade_manager::*;
use core::marker::PhantomData;
use frame_support::traits::tokens::Balance as BalanceT;
use sp_runtime::traits::BlockNumber as BlockNumberT;

pub(super) struct HeroJamFilter<BlockNumber, Balance>(PhantomData<(BlockNumber, Balance)>);

impl<BlockNumber, Balance> TradeManager for HeroJamFilter<BlockNumber, Balance>
where
	BlockNumber: BlockNumberT,
	Balance: BalanceT,
{
	type TradeFilter = AssetType;
	type Asset = HeroJamAsset<BlockNumber, Balance>;

	fn can_be_traded_using(asset: &Self::Asset, filter: &Self::TradeFilter) -> bool {
		asset.asset_type == *filter
	}
}

impl<BlockNumber, Balance> TransferManager for HeroJamFilter<BlockNumber, Balance>
where
	BlockNumber: BlockNumberT,
	Balance: BalanceT,
{
	type TransferFilter = AssetType;
	type Asset = HeroJamAsset<BlockNumber, Balance>;

	fn can_be_transferred_using(asset: &Self::Asset, filter: &Self::TransferFilter) -> bool {
		asset.asset_type == *filter
	}
}
