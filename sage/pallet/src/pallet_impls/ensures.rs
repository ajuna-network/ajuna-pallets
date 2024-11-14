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
	AccountIdOf, AssetIdOf, AssetOf, AssetTradePrices, BalanceOf, Config, Error, LockedAssets,
	Organizer, Pallet, SeasonTradeFilters,
};
use ajuna_primitives::{season_manager::SeasonManager, trade_manager::TradeManager};
use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;

impl<T: Config<I>, I: 'static> Pallet<T, I> {
	/// Check if origin is the current organizer account
	pub(crate) fn ensure_organizer(origin: OriginFor<T>) -> Result<AccountIdOf<T>, DispatchError> {
		let maybe_organizer = ensure_signed(origin)?;
		let existing_organizer = Organizer::<T, I>::get().ok_or(Error::<T, I>::OrganizerNotSet)?;
		ensure!(maybe_organizer == existing_organizer, DispatchError::BadOrigin);
		Ok(maybe_organizer)
	}

	pub(crate) fn ensure_ownership(
		account: &AccountIdOf<T>,
		asset_id: &AssetIdOf<T, I>,
	) -> Result<AssetOf<T, I>, DispatchError> {
		let (owner, asset) = Self::asset_with_owner(asset_id)?;

		if account == &owner ||
			Self::is_locked(asset_id).map(|lock| &lock.locker == account).unwrap_or(false)
		{
			return Ok(asset)
		}

		Err(Error::<T, I>::AssetNotOwned.into())
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

	pub(crate) fn ensure_can_be_set_for_trade(
		asset_id: &AssetIdOf<T, I>,
		asset: &AssetOf<T, I>,
	) -> DispatchResult {
		let asset_season_id = T::SeasonHandler::get_season_id_for(asset_id)?;
		let trade_filter = SeasonTradeFilters::<T, I>::get(&asset_season_id);

		ensure!(
			T::TradeHandler::is_tradeable_using(asset, &trade_filter),
			Error::<T, I>::AssetCannotBeTraded
		);

		Ok(())
	}
}
