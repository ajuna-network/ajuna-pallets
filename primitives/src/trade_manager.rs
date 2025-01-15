// Ajuna Node
// Copyright (C) 2022 BlogaTech AG

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use frame_support::{pallet_prelude::Member, Parameter};
use parity_scale_codec::MaxEncodedLen;

pub trait TradeManager {
	type TradeFilter: Member + Parameter + MaxEncodedLen;

	type Asset: Member + Parameter + MaxEncodedLen;

	fn can_be_traded_using(asset: &Self::Asset, filter: &Self::TradeFilter) -> bool;
}

pub trait TransferManager {
	type TransferFilter: Member + Parameter + MaxEncodedLen;

	type Asset: Member + Parameter + MaxEncodedLen;

	fn can_be_transferred_using(asset: &Self::Asset, filter: &Self::TransferFilter) -> bool;
}
