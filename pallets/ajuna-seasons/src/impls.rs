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

use super::*;
use ajuna_primitives::season_manager::SeasonConfig;

impl<T: Config<I>, I: 'static> SeasonManager for Pallet<T, I> {
	type SeasonId = SeasonIdOf<T, I>;
	type SeasonData = SeasonDataOf<T, I>;
	type AssetId = AssetIdOf<T, I>;
	type Balance = BalanceOf<T, I>;

	fn get_season_id_for(asset_id: &Self::AssetId) -> Result<Self::SeasonId, DispatchError> {
		todo!()
	}

	fn get_current_season_id() -> Self::SeasonId {
		todo!()
	}

	fn is_valid_season(season_id: &Self::SeasonId) -> Result<(), DispatchError> {
		todo!()
	}

	fn get_season_config_for(
		season_id: &Self::SeasonId,
	) -> Result<SeasonConfig<Self::Balance, Self::SeasonData>, DispatchError> {
		todo!()
	}
}
