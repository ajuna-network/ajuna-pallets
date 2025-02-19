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

use crate::{
	AccountIdOf, AssetIdOf, AssetOf, AssetTradePrices, Assets, BalanceOf, Config, Error,
	LockedAssets, Organizer, Pallet,
};

use ajuna_primitives::season_manager::SeasonManager;

use frame_support::pallet_prelude::*;

impl<T: Config<I>, I: 'static> Pallet<T, I> {
	/// Check if origin is the current organizer account
	pub(crate) fn ensure_organizer(maybe_organizer: &AccountIdOf<T>) -> Result<(), DispatchError> {
		let existing_organizer = Organizer::<T, I>::get().ok_or(Error::<T, I>::OrganizerNotSet)?;
		ensure!(maybe_organizer == &existing_organizer, DispatchError::BadOrigin);
		Ok(())
	}

	pub(crate) fn ensure_ownership(
		account: &AccountIdOf<T>,
		asset_id: &AssetIdOf<T, I>,
	) -> Result<AssetOf<T, I>, DispatchError> {
		if let Some((ref owner, asset)) = Assets::<T, I>::get(asset_id) {
			ensure!(owner == account, Error::<T, I>::AssetNotOwned);
			Ok(asset)
		} else {
			Err(Error::<T, I>::UnknownAsset.into())
		}
	}

	pub(crate) fn ensure_unlocked(asset_id: &AssetIdOf<T, I>) -> DispatchResult {
		ensure!(!LockedAssets::<T, I>::contains_key(asset_id), Error::<T, I>::AssetLocked);
		Ok(())
	}

	pub(crate) fn ensure_for_trade(
		asset_id: &AssetIdOf<T, I>,
	) -> Result<(AccountIdOf<T>, BalanceOf<T, I>), DispatchError> {
		let (seller, _) = Self::asset_with_owner(asset_id)?;
		let season_id = T::SeasonHandler::get_season_id_for(asset_id)?;
		let price = AssetTradePrices::<T, I>::get(season_id, asset_id)
			.ok_or(Error::<T, I>::AssetNotInTrade)?;
		Ok((seller, price))
	}
}
