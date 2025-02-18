use crate::asset::{Asset, VariantType};

use ajuna_primitives::trade_manager::*;

use sp_runtime::traits::BlockNumber as BlockNumberT;
use sp_std::marker::PhantomData;

#[derive(Default)]
pub struct GameFilter<BlockNumber>(PhantomData<BlockNumber>);

impl<BlockNumber> TradeManager for GameFilter<BlockNumber>
where
	BlockNumber: BlockNumberT,
{
	type TradeFilter = VariantType;
	type Asset = Asset<BlockNumber>;

	fn can_be_traded_using(asset: &Self::Asset, filter: &Self::TradeFilter) -> bool {
		asset.variant.is_variant(*filter)
	}
}

impl<BlockNumber> TransferManager for GameFilter<BlockNumber>
where
	BlockNumber: BlockNumberT,
{
	type TransferFilter = VariantType;
	type Asset = Asset<BlockNumber>;

	fn can_be_transferred_using(asset: &Self::Asset, filter: &Self::TransferFilter) -> bool {
		asset.variant.is_variant(*filter)
	}
}
