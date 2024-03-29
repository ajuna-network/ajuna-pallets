use frame_support::Parameter;
use parity_scale_codec::{Codec, MaxEncodedLen};
use sp_runtime::{traits::AtLeast32BitUnsigned, BoundedVec, DispatchError, DispatchResult};
use sp_std::vec::Vec;

/// Type used to differentiate attribute codes for each item.
pub type NFTAttribute<N> = BoundedVec<u8, N>;

/// Type to denote an IPFS URL.
pub type IpfsUrl = Vec<u8>;

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
