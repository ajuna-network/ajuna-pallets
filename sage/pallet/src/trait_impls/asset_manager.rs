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
use ajuna_primitives::asset_manager::{AssetFundsManager, AssetInspector};
use frame_support::traits::{
	fungible,
	tokens::{Fortitude, Preservation},
};
use sp_core::hexdisplay::AsBytesRef;
use sp_runtime::traits::Hash;

impl<T: Config<I>, I: 'static> AssetManager for Pallet<T, I> {
	type AccountId = AccountIdOf<T>;
	type AssetId = AssetIdOf<T, I>;
	type Asset = AssetOf<T, I>;

	fn ensure_ownership(
		account: &Self::AccountId,
		asset_id: &Self::AssetId,
	) -> Result<Self::Asset, DispatchError> {
		Pallet::<T, I>::ensure_ownership(account, asset_id)
	}

	fn lock_asset(
		lock_id: LockIdentifier,
		owner: Self::AccountId,
		asset_id: Self::AssetId,
	) -> Result<Self::Asset, DispatchError> {
		let asset = Self::ensure_ownership(&owner, &asset_id)?;
		ensure!(Self::ensure_for_trade(&asset_id).is_err(), Error::<T, I>::CannotLockAssetInTrade);
		ensure!(Self::is_locked(&asset_id).is_none(), Error::<T, I>::AssetLocked);

		let asset_season_id = T::SeasonHandler::get_season_id_for(&asset_id)?;
		Self::do_transfer_asset(
			&owner,
			&Self::technical_account_id(),
			&asset_season_id,
			&asset_id,
		)?;

		let lock = Lock::new(lock_id, owner);
		LockedAssets::<T, I>::insert(&asset_id, &lock);
		Self::deposit_event(Event::AssetLocked { asset_id, lock });

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

		let asset_season_id = T::SeasonHandler::get_season_id_for(&asset_id)?;
		Self::do_transfer_asset(
			&Self::technical_account_id(),
			&owner,
			&asset_season_id,
			&asset_id,
		)?;

		LockedAssets::<T, I>::remove(&asset_id);
		Self::deposit_event(Event::AssetUnlocked { asset_id, lock });

		Ok(asset)
	}

	fn is_locked(asset_id: &Self::AssetId) -> Option<Lock<Self::AccountId>> {
		LockedAssets::<T, I>::get(asset_id)
	}
}

impl<T: Config<I>, I: 'static> AssetInspector for Pallet<T, I> {
	type AccountId = AccountIdOf<T>;
	type AssetId = AssetIdOf<T, I>;
	type Asset = AssetOf<T, I>;

	fn get_asset(asset_id: &Self::AssetId) -> Result<Self::Asset, DispatchError> {
		Assets::<T, I>::get(asset_id)
			.map(|(_, asset)| asset)
			.ok_or(Error::<T, I>::UnknownAsset.into())
	}

	fn iter_assets_from(
		account_id: &Self::AccountId,
	) -> impl Iterator<Item = (Self::AssetId, Self::Asset)> {
		AssetOwners::<T, I>::iter_key_prefix((account_id,)).filter_map(|(_, asset_id)| {
			if let Some((_, asset)) = Assets::<T, I>::get(&asset_id) {
				Some((asset_id, asset))
			} else {
				None
			}
		})
	}
}

impl<T: Config<I>, I: 'static> AssetFundsManager for Pallet<T, I> {
	type AccountId = AccountIdOf<T>;
	type AssetId = AssetIdOf<T, I>;
	type Balance = BalanceOf<T, I>;

	fn inspect_asset_funds(asset_id: &Self::AssetId) -> Self::Balance {
		let asset_hash = <T as frame_system::Config>::Hashing::hash_of(&asset_id);
		let asset_account =
			Self::AccountId::decode(&mut asset_hash.encode().as_bytes_ref()).unwrap();

		<T::Fungible as fungible::Inspect<_>>::reducible_balance(
			&asset_account,
			Preservation::Expendable,
			Fortitude::Polite,
		)
	}

	fn deposit_funds_to_asset(
		asset_id: &Self::AssetId,
		from: &Self::AccountId,
		amount: Self::Balance,
	) -> Result<(), DispatchError> {
		let asset_hash = <T as frame_system::Config>::Hashing::hash_of(&asset_id);
		let asset_account =
			Self::AccountId::decode(&mut asset_hash.encode().as_bytes_ref()).unwrap();

		<T::Fungible as fungible::Mutate<_>>::transfer(
			from,
			&asset_account,
			amount,
			Preservation::Preserve,
		)?;

		Ok(())
	}

	fn transfer_funds_from_asset(
		asset_id: &Self::AssetId,
		to: &Self::AccountId,
		amount: Self::Balance,
	) -> Result<(), DispatchError> {
		let asset_hash = <T as frame_system::Config>::Hashing::hash_of(&asset_id);
		let asset_account =
			Self::AccountId::decode(&mut asset_hash.encode().as_bytes_ref()).unwrap();

		<T::Fungible as fungible::Mutate<_>>::transfer(
			&asset_account,
			to,
			amount,
			Preservation::Preserve,
		)?;
		Ok(())
	}

	fn transfer_all_from_asset(
		asset_id: &Self::AssetId,
		to: &Self::AccountId,
	) -> Result<(), DispatchError> {
		let asset_hash = <T as frame_system::Config>::Hashing::hash_of(&asset_id);
		let asset_account =
			Self::AccountId::decode(&mut asset_hash.encode().as_bytes_ref()).unwrap();

		let reducible_balance = <T::Fungible as fungible::Inspect<_>>::reducible_balance(
			&asset_account,
			Preservation::Expendable,
			Fortitude::Polite,
		);
		<T::Fungible as fungible::Mutate<_>>::transfer(
			&asset_account,
			to,
			reducible_balance,
			Preservation::Expendable,
		)?;

		Ok(())
	}
}
