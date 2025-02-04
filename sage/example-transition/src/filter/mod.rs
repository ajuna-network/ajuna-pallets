use crate::asset::{hero_jam::AssetType, Asset, AssetVariant};

use ajuna_primitives::trade_manager::*;

use core::marker::PhantomData;
use frame_support::traits::tokens::Balance as BalanceT;
use sp_runtime::traits::BlockNumber as BlockNumberT;

mod hero_jam;

#[derive(Default)]
pub struct GameFilter<BlockNumber, Balance>(PhantomData<(BlockNumber, Balance)>);

impl<BlockNumber, Balance> TradeManager for GameFilter<BlockNumber, Balance>
where
	BlockNumber: BlockNumberT,
	Balance: BalanceT,
{
	type TradeFilter = AssetType;
	type Asset = Asset<BlockNumber, Balance>;

	fn can_be_traded_using(asset: &Self::Asset, filter: &Self::TradeFilter) -> bool {
		match &asset.asset_variant {
			AssetVariant::HeroJam(hero_jam) =>
				hero_jam::HeroJamFilter::can_be_traded_using(hero_jam, filter),
		}
	}
}

impl<BlockNumber, Balance> TransferManager for GameFilter<BlockNumber, Balance>
where
	BlockNumber: BlockNumberT,
	Balance: BalanceT,
{
	type TransferFilter = AssetType;
	type Asset = Asset<BlockNumber, Balance>;

	fn can_be_transferred_using(asset: &Self::Asset, filter: &Self::TransferFilter) -> bool {
		match &asset.asset_variant {
			AssetVariant::HeroJam(hero_jam) =>
				hero_jam::HeroJamFilter::can_be_transferred_using(hero_jam, filter),
		}
	}
}
