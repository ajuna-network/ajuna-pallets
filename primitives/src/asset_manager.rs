use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

pub trait AssetManager {
	type AccountId;

	type AssetId;

	type Asset;

	fn lock_asset(owner: Self::AccountId, asset_id: Self::AssetId) -> Result<Self::Asset, Error>;

	fn unlock_asset(owner: Self::AccountId, asset_id: Self::AssetId) -> Result<(), Error>;

	fn is_locked(asset: &Self::AssetId) -> bool;
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Decode, Encode, MaxEncodedLen, TypeInfo)]
pub enum Error {
	Ownership,
	AssetInTrade,
	NftTransferClosed,
	MaxOwnershipReached,
	UnknownAsset,
	AssetIsUnprepared,
	AssetIsLocked,
	AssetIsUnlocked,
}
