use frame_support::pallet_prelude::TypeInfo;
use sp_core::{Decode, Encode, MaxEncodedLen};

/// Placeholder type, this was just a quick brain dump to get things going.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct Asset {
	pub collection_id: u32,

	pub asset_type: u32,

	pub asset_sub_type: u32,

	pub dna: [u8; 32],

	pub minted_at: u32,
}

pub enum Error {
	InvalidTransitionId,
	InvalidAssetLength,
	TransferError,
	FeeError,
}
