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

use frame_support::{
	pallet_prelude::{DispatchError, Member},
	Parameter,
};
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use std::marker::PhantomData;

pub trait SeasonManager {
	type SeasonId: Member + Parameter + MaxEncodedLen;

	type AssetId: Member + Parameter + MaxEncodedLen;

	fn get_season_for(asset: &Self::AssetId) -> Option<Self::SeasonId>;

	fn get_current_season() -> Option<Self::SeasonId>;

	fn is_valid_season(season_id: &Self::SeasonId) -> Result<(), DispatchError>;
}

pub struct EmptySeasonManager<AssetId> {
	_phantom: PhantomData<AssetId>,
}

pub const NO_SEASON_AVAILABLE: &str = "NO SEASON AVAILABLE";

impl<AssetId> SeasonManager for EmptySeasonManager<AssetId>
where
	AssetId: Member + Parameter + MaxEncodedLen,
{
	type SeasonId = ();
	type AssetId = AssetId;

	fn get_season_for(_asset: &Self::AssetId) -> Option<Self::SeasonId> {
		None
	}

	fn get_current_season() -> Option<Self::SeasonId> {
		None
	}

	fn is_valid_season(_season_id: &Self::SeasonId) -> Result<(), DispatchError> {
		Err(DispatchError::Other(NO_SEASON_AVAILABLE))
	}
}
