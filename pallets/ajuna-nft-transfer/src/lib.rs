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

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod traits;
pub mod weights;

use crate::weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use crate::traits::*;
	use ajuna_primitives::asset_manager::AssetManager;
	use frame_support::{
		pallet_prelude::*,
		traits::{
			tokens::nonfungibles_v2::{Inspect, Mutate},
			Locker,
		},
		PalletId,
	};
	use frame_system::{ensure_root, ensure_signed, pallet_prelude::OriginFor};
	use sp_runtime::traits::{AccountIdConversion, AtLeast32BitUnsigned};

	pub(crate) type AccountIdFor<T> = <T as frame_system::Config>::AccountId;

	#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Eq, PartialEq)]
	pub enum NftStatus {
		/// The NFT exists in storage in the chain
		Stored,
		/// The NFT has been uploaded outside the chain
		Uploaded,
	}

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The NFT-transfer's pallet id, used for deriving its sovereign account ID.
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// The overarching event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// Identifier for the collection of item.
		type CollectionId: Member + Parameter + MaxEncodedLen + Copy + AtLeast32BitUnsigned;

		type Item: NftConvertible<Self::KeyLimit, Self::ValueLimit>;

		/// The type used to identify a unique item within a collection.
		type ItemId: Member + Parameter + MaxEncodedLen + Copy;

		/// Type that holds the specific configurations for an item.
		type ItemConfig: Default + MaxEncodedLen + TypeInfo;

		type AssetManager: AssetManager<
			AccountId = AccountIdFor<Self>,
			AssetId = Self::ItemId,
			Asset = Self::Item,
		>;

		/// The maximum length of an attribute key.
		#[pallet::constant]
		type KeyLimit: Get<u32>;

		/// The maximum length of an attribute value.
		#[pallet::constant]
		type ValueLimit: Get<u32>;

		/// An NFT helper for the management of collections and items.
		type NftHelper: Inspect<Self::AccountId, CollectionId = Self::CollectionId, ItemId = Self::ItemId>
			+ Mutate<Self::AccountId, Self::ItemConfig>;

		type WeightInfo: WeightInfo;
	}

	#[pallet::storage]
	pub type CollectionId<T: Config> = StorageValue<_, T::CollectionId, OptionQuery>;

	#[pallet::storage]
	pub type ServiceAccount<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

	#[pallet::storage]
	pub type Preparation<T: Config> = StorageMap<_, Identity, T::ItemId, IpfsUrl, OptionQuery>;

	#[pallet::storage]
	pub type NftStatuses<T: Config> =
		StorageDoubleMap<_, Identity, T::CollectionId, Identity, T::ItemId, NftStatus, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A collection ID has been set.
		CollectionIdSet { collection_id: T::CollectionId },
		/// A service account has been set.
		ServiceAccountSet { service_account: T::AccountId },
		/// Avatar prepared.
		PreparedAvatar { asset_id: T::ItemId },
		/// Avatar unprepared.
		UnpreparedAvatar { asset_id: T::ItemId },
		/// IPFS URL prepared.
		PreparedIpfsUrl { url: IpfsUrl },
		/// Item has been stored as an NFT [collection_id, item_id, owner]
		ItemStored { collection_id: T::CollectionId, item_id: T::ItemId, owner: T::AccountId },
		/// Item has been restored back from its NFT representation [collection_id, item_id, owner]
		ItemRestored { collection_id: T::CollectionId, item_id: T::ItemId, owner: T::AccountId },
	}

	#[pallet::error]
	pub enum Error<T> {
		/// There is no collection ID set for NFT handler.
		CollectionIdNotSet,
		/// Tried to prepare an already prepared asset.
		AssetAlreadyPrepared,
		/// IPFS URL is not prepared yet.
		AssetUnprepared,
		/// IPFS URL must not be an empty string.
		EmptyIpfsUrl,
		/// Item code must be different to attribute codes.
		DuplicateItemCode,
		/// The given NFT item doesn't exist.
		UnknownItem,
		/// The given claim doesn't exist.
		UnknownClaim,
		/// No service account has been set.
		NoServiceAccount,
		/// The given NFT is not owned by the requester.
		NftNotOwned,
		/// The given NFT is currently outside of the chain, transfer it back before attempting a
		/// restore.
		NftOutsideOfChain,
		/// NFT transfer is not available at the moment.
		NftTransferClosed,
		/// The process of restoring an NFT into an item has failed.
		ItemRestoreFailure,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Set the collection ID to associate assets with.
		///
		/// Externally created collection ID for assets must be set in the `CollectionId` storage
		/// to serve as a lookup for locking and unlocking assets as NFTs.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::set_collection_id())]
		pub fn set_collection_id(
			origin: OriginFor<T>,
			collection_id: T::CollectionId,
		) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			T::AssetManager::ensure_organizer(&signer)?;
			CollectionId::<T>::put(&collection_id);
			Self::deposit_event(Event::CollectionIdSet { collection_id });
			Ok(())
		}

		/// Set a service account.
		///
		/// The origin of this call must be root. A service account has sufficient privilege to call
		/// the `prepare_ipfs` extrinsic.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::set_service_account())]
		pub fn set_service_account(
			origin: OriginFor<T>,
			service_account: T::AccountId,
		) -> DispatchResult {
			ensure_root(origin)?;
			ServiceAccount::<T>::put(&service_account);
			Self::deposit_event(Event::ServiceAccountSet { service_account });
			Ok(())
		}

		/// Locks an asset to be tokenized as an NFT.
		///
		/// The origin of this call must specify an asset, owned by the origin, to prevent it from
		/// forging, trading and transferring it to other players. When successful, the ownership of
		/// the asset is removed from the player.
		///
		/// Locking an asset allows for new
		/// ways of interacting with it currently under development.
		///
		/// Weight: `O(n)` where:
		/// - `n = max assets per player`
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::lock_asset())]
		pub fn store_prepared_as_nft(origin: OriginFor<T>, asset_id: T::ItemId) -> DispatchResult {
			let player = ensure_signed(origin)?;

			let asset = T::AssetManager::ensure_ownership(&player, &asset_id)?;

			let collection_id = CollectionId::<T>::get().ok_or(Error::<T>::CollectionIdNotSet)?;
			let url = Preparation::<T>::take(&asset_id).ok_or(Error::<T>::AssetUnprepared)?;

			Self::store_as_nft(player, collection_id, asset_id, asset, url)?;

			Ok(())
		}

		/// Unlocks an asset removing its NFT representation.
		///
		/// The origin of this call must specify an asset, owned and locked by the origin, to allow
		/// forging, trading and transferring it to other players. When successful, the ownership of
		/// the asset is transferred from the pallet's technical account back to the player and its
		/// existing NFT representation is destroyed.
		///
		/// Weight: `O(n)` where:
		/// - `n = max assets per player`
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::unlock_asset())]
		pub fn recover_asset_from_nft(origin: OriginFor<T>, asset_id: T::ItemId) -> DispatchResult {
			let player = ensure_signed(origin)?;

			let _asset = T::AssetManager::unlock_asset(player.clone(), asset_id)?;

			let collection_id = CollectionId::<T>::get().ok_or(Error::<T>::CollectionIdNotSet)?;

			// need to specify explicit Item type.
			let _ = <Self as NftHandler<_, _, _, _, <T as Config>::Item>>::recover_from_nft(
				player,
				collection_id,
				asset_id,
			)?;
			Ok(())
		}

		/// Prepare an asset to be uploaded to IPFS.
		///
		/// The origin of this call must specify an asset, owned by the origin, to display the
		/// intention of uploading it to an IPFS storage. When successful, the `PreparedAvatar`
		/// event is emitted to be picked up by our external service that interacts with the IPFS.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::prepare_asset())]
		pub fn prepare_asset(origin: OriginFor<T>, asset_id: T::ItemId) -> DispatchResult {
			let player = ensure_signed(origin)?;
			Self::ensure_unprepared(&asset_id)?;

			let asset = T::AssetManager::lock_asset(player.clone(), asset_id)?;

			let service_account = ServiceAccount::<T>::get().ok_or(Error::<T>::NoServiceAccount)?;
			T::AssetManager::handle_asset_fees(&asset, &player, &service_account)?;

			Preparation::<T>::insert(asset_id, IpfsUrl::default());
			Self::deposit_event(Event::PreparedAvatar { asset_id });
			Ok(())
		}

		/// Unprepare an asset to be detached from IPFS.
		///
		/// The origin of this call must specify an asset, owned by the origin, that is undergoing
		/// the IPFS upload process.
		///
		/// Weight: `O(1)`
		#[pallet::call_index(5)]
		#[pallet::weight(T::WeightInfo::unprepare_asset())]
		pub fn unprepare_asset(origin: OriginFor<T>, asset_id: T::ItemId) -> DispatchResult {
			let player = ensure_signed(origin)?;
			let _ = T::AssetManager::ensure_ownership(&player, &asset_id)?;
			ensure!(T::AssetManager::nft_transfer_open(), Error::<T>::NftTransferClosed);
			ensure!(Preparation::<T>::contains_key(asset_id), Error::<T>::AssetUnprepared);

			Preparation::<T>::remove(asset_id);
			Self::deposit_event(Event::UnpreparedAvatar { asset_id });
			Ok(())
		}

		/// Prepare IPFS for an asset.
		///
		/// The origin of this call must be signed by the service account to upload the given asset
		/// to an IPFS storage and stores its CID. A third-party service subscribes for the
		/// `PreparedAvatar` events which triggers preparing assets, their upload to IPFS and
		/// storing their CIDs.
		//
		/// Weight: `O(1)`
		#[pallet::call_index(6)]
		#[pallet::weight(T::WeightInfo::prepare_ipfs())]
		pub fn prepare_ipfs(
			origin: OriginFor<T>,
			asset_id: T::ItemId,
			url: IpfsUrl,
		) -> DispatchResult {
			let _ = Self::ensure_service_account(origin)?;

			ensure!(T::AssetManager::nft_transfer_open(), Error::<T>::NftTransferClosed);
			ensure!(Preparation::<T>::contains_key(asset_id), Error::<T>::AssetUnprepared);
			ensure!(!url.is_empty(), Error::<T>::EmptyIpfsUrl);
			Preparation::<T>::insert(asset_id, &url);
			Self::deposit_event(Event::PreparedIpfsUrl { url });
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		/// The account identifier to delegate NFT transfer operations.
		pub fn account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}

		pub(crate) fn ensure_service_account(
			origin: OriginFor<T>,
		) -> Result<T::AccountId, DispatchError> {
			let maybe_sa = ensure_signed(origin)?;
			let existing_sa = ServiceAccount::<T>::get().ok_or(Error::<T>::NoServiceAccount)?;
			ensure!(maybe_sa == existing_sa, DispatchError::BadOrigin);
			Ok(maybe_sa)
		}

		fn ensure_unprepared(asset_id: &T::ItemId) -> DispatchResult {
			ensure!(!Preparation::<T>::contains_key(asset_id), Error::<T>::AssetAlreadyPrepared);
			Ok(())
		}
	}

	impl<T: Config, Item: NftConvertible<T::KeyLimit, T::ValueLimit>>
		NftHandler<T::AccountId, T::ItemId, T::KeyLimit, T::ValueLimit, Item> for Pallet<T>
	{
		type CollectionId = T::CollectionId;

		fn store_as_nft(
			owner: T::AccountId,
			collection_id: Self::CollectionId,
			item_id: T::ItemId,
			item: Item,
			ipfs_url: IpfsUrl,
		) -> DispatchResult {
			let config = T::ItemConfig::default();
			T::NftHelper::mint_into(&collection_id, &item_id, &owner, &config, false)?;
			T::NftHelper::set_attribute(
				&collection_id,
				&item_id,
				Item::ITEM_CODE,
				item.encode().as_slice(),
			)?;

			ensure!(!ipfs_url.is_empty(), Error::<T>::EmptyIpfsUrl);
			T::NftHelper::set_attribute(
				&collection_id,
				&item_id,
				Item::IPFS_URL_CODE,
				ipfs_url.as_slice(),
			)?;

			item.get_encoded_attributes()
				.iter()
				.try_for_each(|(attribute_code, attribute)| {
					ensure!(
						attribute_code.as_slice() != Item::ITEM_CODE,
						Error::<T>::DuplicateItemCode
					);
					T::NftHelper::set_attribute(&collection_id, &item_id, attribute_code, attribute)
				})?;

			NftStatuses::<T>::insert(collection_id, item_id, NftStatus::Stored);

			Self::deposit_event(Event::<T>::ItemStored { collection_id, item_id, owner });
			Ok(())
		}

		fn recover_from_nft(
			owner: T::AccountId,
			collection_id: Self::CollectionId,
			item_id: T::ItemId,
		) -> Result<Item, DispatchError> {
			ensure!(
				NftStatuses::<T>::get(collection_id, item_id) == Some(NftStatus::Stored),
				Error::<T>::NftOutsideOfChain
			);

			let item =
				T::NftHelper::system_attribute(&collection_id, Some(&item_id), Item::ITEM_CODE)
					.ok_or(Error::<T>::UnknownItem)?;

			T::NftHelper::clear_attribute(&collection_id, &item_id, Item::ITEM_CODE)?;
			T::NftHelper::clear_attribute(&collection_id, &item_id, Item::IPFS_URL_CODE)?;
			for attribute_key in Item::get_attribute_codes() {
				T::NftHelper::clear_attribute(&collection_id, &item_id, &attribute_key)?;
			}

			NftStatuses::<T>::remove(collection_id, item_id);
			T::NftHelper::burn(&collection_id, &item_id, Some(&owner))?;

			Self::deposit_event(Event::<T>::ItemRestored { collection_id, item_id, owner });
			Item::decode(&mut item.as_slice()).map_err(|_| Error::<T>::ItemRestoreFailure.into())
		}
	}

	impl<T: Config> Locker<T::CollectionId, T::ItemId> for Pallet<T> {
		fn is_locked(collection_id: T::CollectionId, item_id: T::ItemId) -> bool {
			matches!(NftStatuses::<T>::get(collection_id, item_id), Some(NftStatus::Uploaded))
		}
	}
}
