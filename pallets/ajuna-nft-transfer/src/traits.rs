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

use frame_support::{pallet_prelude::ConstU32, Parameter};
use parity_scale_codec::{Codec, MaxEncodedLen};
use sp_runtime::{traits::AtLeast32BitUnsigned, BoundedVec, DispatchError, DispatchResult};
use sp_std::vec::Vec;

/// Type used to differentiate attribute codes for each item.
pub type NFTAttribute<N> = BoundedVec<u8, N>;

/// Type to denote an IPFS URL.
pub type IpfsUrl = BoundedVec<u8, ConstU32<100>>;

/// Marker trait for items that can be converted back and forth into an NFT representation.
pub trait NftConvertible<KL, VL>: Codec {
	/// Numeric key used to identify this item as an NFT attribute.
	const ITEM_CODE: &'static [u8];
	/// Numeric key used to identify this item's IPFS URL as an NFT attribute.
	const IPFS_URL_CODE: &'static [u8];

	/// Returns the list of attribute codes associated with this type.
	fn get_attribute_codes() -> Vec<NFTAttribute<KL>>;

	/// Returns the list of pairs of attribute code and its encoded attribute.
	fn get_encoded_attributes(&self) -> Vec<(NFTAttribute<KL>, NFTAttribute<VL>)>;
}

/// Trait to define the transformation and bridging of NFT items.
pub trait NftHandler<Account, ItemId, KL, VL, Item: NftConvertible<KL, VL>> {
	type CollectionId: AtLeast32BitUnsigned + Codec + Parameter + MaxEncodedLen;

	/// Consumes the given `item` and its associated identifiers, and stores it as an NFT
	/// owned by `owner`.
	fn store_as_nft(
		owner: Account,
		collection_id: Self::CollectionId,
		item_id: ItemId,
		item: Item,
		ipfs_url: IpfsUrl,
	) -> DispatchResult;

	/// Recovers the NFT item indexed by `collection_id` and `item_id`.
	fn recover_from_nft(
		owner: Account,
		collection_id: Self::CollectionId,
		item_id: ItemId,
	) -> Result<Item, DispatchError>;

	/// Schedules the upload of a previously stored NFT item to be teleported out of the chain, into
	/// an external source. Once this process completes the item is locked until transported back
	/// from the external source into the chain.
	fn schedule_upload(
		_owner: Account,
		_collection_id: Self::CollectionId,
		_item_id: ItemId,
	) -> DispatchResult {
		todo!("will be implemented when bridge is ready")
	}
}
