// Ajuna Node
// Copyright (C) 2022 BlogaTech AG
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

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
