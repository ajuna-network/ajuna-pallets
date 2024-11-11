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
		/*let (owner, avatar) = Self::avatars(asset_id)?;

		if account == &owner ||
			Self::is_locked(asset_id).map(|lock| &lock.locker == account).unwrap_or(false)
		{
			return Ok(avatar)
		}

		Err(Error::<T>::Ownership.into())*/
		todo!()
	}

	fn lock_asset(
		lock_id: LockIdentifier,
		owner: Self::AccountId,
		asset_id: Self::AssetId,
	) -> Result<Self::Asset, DispatchError> {
		/*let avatar = Self::ensure_ownership(&owner, &asset_id)?;
		ensure!(Self::ensure_for_trade(&asset_id).is_err(), Error::<T>::AvatarInTrade);
		ensure!(Self::is_locked(&asset_id).is_none(), Error::<T>::AvatarLocked);

		Self::try_remove_avatar_ownership_from(&owner, &avatar.season_id, &asset_id)?;

		LockedAvatars::<T>::insert(asset_id, Lock::new(lock_id, owner));
		Self::deposit_event(Event::AvatarLocked { avatar_id: asset_id });

		Ok(avatar)*/
		todo!()
	}

	fn unlock_asset(
		lock_id: LockIdentifier,
		owner: Self::AccountId,
		asset_id: Self::AssetId,
	) -> Result<Self::Asset, DispatchError> {
		/*let avatar = Self::ensure_ownership(&Self::technical_account_id(), &asset_id)?;

		let lock = Self::is_locked(&asset_id).ok_or(Error::<T>::AvatarNotLocked)?;
		ensure!(lock.id == lock_id, Error::<T>::AvatarLockedByOtherApplication);
		ensure!(lock.locker == owner, Error::<T>::Ownership);

		Self::try_restore_avatar_ownership_to(&owner, &avatar.season_id, &asset_id)?;

		LockedAvatars::<T>::remove(asset_id);
		Self::deposit_event(Event::AvatarUnlocked { avatar_id: asset_id });

		Ok(avatar)*/
		todo!()
	}

	fn is_locked(asset_id: &Self::AssetId) -> Option<Lock<Self::AccountId>> {
		//LockedAvatars::<T>::get(asset_id)
		todo!()
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
