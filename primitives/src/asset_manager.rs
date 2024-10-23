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

use frame_support::pallet_prelude::{DispatchError, Member};
use parity_scale_codec::{Codec, Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;

pub type LockIdentifier = [u8; 8];

/// A lock that tracks the purpose of the lock via the `id` and
/// who was the `locker`.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, MaxEncodedLen, TypeInfo)]
pub struct Lock<AccountId> {
	/// An identifier for this lock. Only one lock may be in existence for each identifier.
	pub id: LockIdentifier,
	/// Account who locked the asset.
	///
	/// This is needed as the locked asset looses its ownership information.
	pub locker: AccountId,
}

impl<AccountId> Lock<AccountId> {
	pub fn new(id: LockIdentifier, locker: AccountId) -> Self {
		Self { id, locker }
	}
}

/// The asset manager trait that can be passed around to other pallets that want to do
/// something with the assets.
///
/// This trait is more powerful than it is supposed to be in the future, but simplification here
/// shall be done in the course of implementing sage.
pub trait AssetManager {
	type AccountId: Member + Codec;

	type AssetId: Member + Codec;

	type Asset: Member + Codec;

	fn ensure_ownership(
		owner: &Self::AccountId,
		asset_id: &Self::AssetId,
	) -> Result<Self::Asset, DispatchError>;

	fn lock_asset(
		lock_id: LockIdentifier,
		owner: Self::AccountId,
		asset_id: Self::AssetId,
	) -> Result<Self::Asset, DispatchError>;

	fn unlock_asset(
		lock_id: LockIdentifier,
		owner: Self::AccountId,
		asset_id: Self::AssetId,
	) -> Result<Self::Asset, DispatchError>;

	fn is_locked(asset: &Self::AssetId) -> Option<Lock<Self::AccountId>>;

	/// This should probably be moved from the global config into the nft-transfer-pallet?
	fn nft_transfer_open() -> bool;

	/// This should als be extracted to a separate fee handler component.
	fn handle_asset_prepare_fee(
		asset: &Self::Asset,
		from: &Self::AccountId,
		fees_recipient: &Self::AccountId,
	) -> Result<(), DispatchError>;

	#[cfg(feature = "runtime-benchmarks")]
	fn create_assets(owner: Self::AccountId, count: u32) -> Vec<Self::AssetId>;
}
