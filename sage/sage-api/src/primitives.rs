use frame_support::pallet_prelude::TypeInfo;
use sp_core::{Decode, Encode, MaxEncodedLen};

pub trait AssetT {
	fn collection_id(&self) -> u32;
	fn asset_type(&self) -> u32;
	fn dna(&self) -> [u8; 32];
	fn minted_at(&self) -> u32;
}

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
