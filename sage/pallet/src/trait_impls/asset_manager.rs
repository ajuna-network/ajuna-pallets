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

impl<T: Config<I>, I: 'static> AssetManager for Pallet<T, I> {
	type AccountId = AccountIdOf<T>;
	type AssetId = AssetIdOf<T, I>;
	type Asset = AssetOf<T, I>;

	fn ensure_ownership(
		account: &Self::AccountId,
		asset_id: &Self::AssetId,
	) -> Result<Self::Asset, DispatchError> {
		let (owner, avatar) = Self::asset_with_owner(asset_id)?;

		if account == &owner ||
			Self::is_locked(asset_id).map(|lock| &lock.locker == account).unwrap_or(false)
		{
			return Ok(avatar)
		}

		Err(Error::<T, I>::AssetNotOwned.into())
	}

	fn lock_asset(
		lock_id: LockIdentifier,
		owner: Self::AccountId,
		asset_id: Self::AssetId,
	) -> Result<Self::Asset, DispatchError> {
		let asset = Self::ensure_ownership(&owner, &asset_id)?;
		ensure!(Self::ensure_for_trade(&asset_id).is_err(), Error::<T, I>::CannotLockAssetInTrade);
		ensure!(Self::is_locked(&asset_id).is_none(), Error::<T, I>::AssetLocked);

		let asset_season_id = T::SeasonHandler::get_season_for(&asset_id);
		AssetOwners::<T, I>::mutate(&owner, &asset_season_id, |asset_ids| {
			asset_ids.retain(|id| id != &asset_id);
		});

		Assets::<T, I>::try_mutate(&asset_id, |maybe_asset| -> DispatchResult {
			let (from_owner, _) = maybe_asset.as_mut().ok_or(Error::<T, I>::UnknownAsset)?;
			*from_owner = Self::technical_account_id();
			Ok(())
		})?;

		LockedAssets::<T, I>::insert(&asset_id, Lock::new(lock_id, owner));
		Self::deposit_event(Event::AssetLocked { asset_id });

		Ok(asset)
	}

	fn unlock_asset(
		lock_id: LockIdentifier,
		owner: Self::AccountId,
		asset_id: Self::AssetId,
	) -> Result<Self::Asset, DispatchError> {
		let asset = Self::ensure_ownership(&Self::technical_account_id(), &asset_id)?;
		let lock = Self::is_locked(&asset_id).ok_or(Error::<T, I>::AssetNotLocked)?;
		ensure!(lock.id == lock_id, Error::<T, I>::AssetLockedByOtherApplication);
		ensure!(lock.locker == owner, Error::<T, I>::AssetNotOwned);

		let asset_season_id = T::SeasonHandler::get_season_for(&asset_id);
		AssetOwners::<T, I>::try_mutate(&owner, &asset_season_id, |asset_ids| {
			asset_ids
				.try_push(asset_id.clone())
				.map_err(|_| Error::<T, I>::MaxOwnershipReached)?;
			ensure!(
				asset_ids.len() <=
					PlayerSeasonConfigs::<T, I>::get(&owner, &asset_season_id).storage_tier
						as usize,
				Error::<T, I>::MaxOwnershipReached
			);
			Ok::<_, DispatchError>(())
		})?;

		Assets::<T, I>::try_mutate(&asset_id, |maybe_asset| -> DispatchResult {
			let (from_owner, _) = maybe_asset.as_mut().ok_or(Error::<T, I>::UnknownAsset)?;
			*from_owner = owner.clone();
			Ok(())
		})?;

		LockedAssets::<T, I>::remove(&asset_id);
		Self::deposit_event(Event::AssetUnlocked { asset_id });

		Ok(asset)
	}

	fn is_locked(asset_id: &Self::AssetId) -> Option<Lock<Self::AccountId>> {
		LockedAssets::<T, I>::get(asset_id)
	}

	fn nft_transfer_open() -> bool {
		// TODO
		true
	}

	fn handle_asset_prepare_fee(
		_asset: &Self::Asset,
		_player: &Self::AccountId,
		_fee_recipient: &Self::AccountId,
	) -> Result<(), DispatchError> {
		// TODO
		Ok(())
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn create_assets(_owner: Self::AccountId, _count: u32) -> Vec<(Self::AssetId, Self::Asset)> {
		// TODO
		Vec::with_capacity(0)
	}
}
