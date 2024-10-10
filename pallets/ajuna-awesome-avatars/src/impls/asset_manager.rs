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

use crate::*;

impl<T: Config> AssetManager for Pallet<T> {
	type AccountId = AccountIdFor<T>;
	type AssetId = AvatarIdOf<T>;
	type Asset = AvatarOf<T>;

	fn ensure_organizer(account: &Self::AccountId) -> Result<(), DispatchError> {
		let existing_organizer = Organizer::<T>::get().ok_or(Error::<T>::OrganizerNotSet)?;
		ensure!(account == &existing_organizer, DispatchError::BadOrigin);
		Ok(())
	}

	fn ensure_ownership(
		account: &Self::AccountId,
		asset_id: &Self::AssetId,
	) -> Result<Self::Asset, DispatchError> {
		let (owner, avatar) = Self::avatars(asset_id)?;

		if account == &owner ||
			Self::is_locked(asset_id).map(|lock| &lock.locker == account).unwrap_or(false)
		{
			return Ok(avatar)
		}

		Err(Error::<T>::Ownership.into())
	}

	fn lock_asset(
		lock_id: LockIdentifier,
		owner: Self::AccountId,
		asset_id: Self::AssetId,
	) -> Result<Self::Asset, DispatchError> {
		let avatar = Self::ensure_ownership(&owner, &asset_id)?;
		ensure!(Self::ensure_for_trade(&asset_id).is_err(), Error::<T>::AvatarInTrade);
		ensure!(Self::is_locked(&asset_id).is_none(), Error::<T>::AvatarLocked);

		Self::try_remove_avatar_ownership_from(&owner, &avatar.season_id, &asset_id)?;

		LockedAvatars::<T>::insert(asset_id, Lock::new(lock_id, owner));
		Self::deposit_event(Event::AvatarLocked { avatar_id: asset_id });

		Ok(avatar)
	}

	fn unlock_asset(
		lock_id: LockIdentifier,
		owner: Self::AccountId,
		asset_id: Self::AssetId,
	) -> Result<Self::Asset, DispatchError> {
		let avatar = Self::ensure_ownership(&Self::technical_account_id(), &asset_id)?;

		let lock = Self::is_locked(&asset_id).ok_or(Error::<T>::AvatarNotLocked)?;
		ensure!(lock.id == lock_id, Error::<T>::AvatarLockedByOtherApplication);
		ensure!(lock.locker == owner, Error::<T>::Ownership);

		Self::try_restore_avatar_ownership_to(&owner, &avatar.season_id, &asset_id)?;

		LockedAvatars::<T>::remove(asset_id);
		Self::deposit_event(Event::AvatarUnlocked { avatar_id: asset_id });

		Ok(avatar)
	}

	fn is_locked(asset_id: &Self::AssetId) -> Option<Lock<Self::AccountId>> {
		LockedAvatars::<T>::get(asset_id)
	}

	fn nft_transfer_open() -> bool {
		GlobalConfigs::<T>::get().nft_transfer.open
	}

	fn handle_asset_prepare_fee(
		asset: &Self::Asset,
		player: &Self::AccountId,
		fee_recipient: &Self::AccountId,
	) -> Result<(), DispatchError> {
		let Season { fee, .. } = Self::seasons(&asset.season_id)?;
		T::Currency::transfer(player, fee_recipient, fee.prepare_avatar, AllowDeath)
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn create_assets(owner: Self::AccountId, count: u32) -> Vec<Self::AssetId> {
		benchmark_helper::create_avatars::<T>(owner.clone(), count).unwrap();

		let season_id = CurrentSeasonStatus::<T>::get().season_id;
		let avatar_ids = Owners::<T>::get(owner, season_id);

		avatar_ids.into()
	}

	#[cfg(feature = "runtime-benchmarks")]
	fn set_organizer(organizer: Self::AccountId) {
		Organizer::<T>::put(organizer)
	}
}
