use crate::asset::{hero_jam::AssetType, Asset, AssetVariant};

use ajuna_primitives::trade_manager::*;

use sp_runtime::traits::BlockNumber as BlockNumberT;
use sp_std::marker::PhantomData;

mod hero_jam;

#[derive(Default)]
pub struct GameFilter<BlockNumber>(PhantomData<BlockNumber>);

impl<BlockNumber> TradeManager for GameFilter<BlockNumber>
where
	BlockNumber: BlockNumberT,
{
	type TradeFilter = AssetType;
	type Asset = Asset<BlockNumber>;

	fn can_be_traded_using(asset: &Self::Asset, filter: &Self::TradeFilter) -> bool {
		match &asset.asset_variant {
			AssetVariant::HeroJam(hero_jam) =>
				hero_jam::HeroJamFilter::can_be_traded_using(hero_jam, filter),
		}
	}
}

impl<BlockNumber> TransferManager for GameFilter<BlockNumber>
where
	BlockNumber: BlockNumberT,
{
	type TransferFilter = AssetType;
	type Asset = Asset<BlockNumber>;

	fn can_be_transferred_using(asset: &Self::Asset, filter: &Self::TransferFilter) -> bool {
		match &asset.asset_variant {
			AssetVariant::HeroJam(hero_jam) =>
				hero_jam::HeroJamFilter::can_be_transferred_using(hero_jam, filter),
		}
	}
}
