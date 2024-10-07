use frame_support::{__private::DispatchError, pallet_prelude::Member};
use parity_scale_codec::Codec;

pub trait AssetManager {
	type AccountId: Member + Codec;

	type AssetId: Member + Codec;

	type Asset: Member + Codec;

	fn ensure_organizer(account: &Self::AccountId) -> Result<(), DispatchError>;

	fn ensure_ownership(
		owner: &Self::AccountId,
		asset_id: &Self::AssetId,
	) -> Result<Self::Asset, DispatchError>;

	fn lock_asset(
		owner: Self::AccountId,
		asset_id: Self::AssetId,
	) -> Result<Self::Asset, DispatchError>;

	fn unlock_asset(
		owner: Self::AccountId,
		asset_id: Self::AssetId,
	) -> Result<Self::Asset, DispatchError>;

	fn is_locked(asset: &Self::AssetId) -> bool;

	fn nft_transfer_open() -> bool;

	fn handle_asset_fees(
		asset: &Self::Asset,
		from: &Self::AccountId,
		fees_recipient: &Self::AccountId,
	) -> Result<(), DispatchError>;
}
