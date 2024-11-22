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
		AssetSeasonRegister::<T, I>::get(asset_id).ok_or(Error::<T, I>::AssetNotRegistered.into())
	}

	fn get_current_season_id() -> Result<Self::SeasonId, DispatchError> {
		CurrentSeasonStatus::<T, I>::get()
			.map(|status| status.season_id)
			.map_err(|e| e.into())
	}

	fn is_valid_season(season_id: &Self::SeasonId) -> Result<(), DispatchError> {
		if Seasons::<T, I>::contains_key(season_id) {
			Ok(())
		} else {
			Err(Error::<T, I>::InvalidSeason.into())
		}
	}

	fn get_season_config_for(
		season_id: &Self::SeasonId,
	) -> Result<SeasonConfig<Self::Balance, Self::SeasonData>, DispatchError> {
		Seasons::<T, I>::get(season_id).ok_or(Error::<T, I>::InvalidSeason.into())
	}

	fn register_asset_in(
		asset_id: &Self::AssetId,
		season_id: &Self::SeasonId,
	) -> Result<(), DispatchError> {
		Self::is_valid_season(season_id)?;
		AssetSeasonRegister::<T, I>::insert(asset_id, season_id);

		Ok(())
	}
}
