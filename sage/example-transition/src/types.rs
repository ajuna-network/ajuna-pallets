//! Some example transitions.
//!
//! These should be expanded to really showcase the power of the SageApi design.

use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use sage_api::traits::Identifiable;
use scale_info::TypeInfo;
use sp_core::H256;

pub type AssetId = H256;

/// Placeholder type, this was just a quick brain dump to get things going.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct Asset {
	asset_id: H256,

	pub collection_id: u32,

	pub asset_type: u32,

	pub asset_sub_type: u32,

	pub dna: [u8; 32],

	pub minted_at: u32,

	// Example of a game's custom field.
	pub level: Level,

	// Example of a game's custom field.
	pub consumed: bool,
}

impl Asset {
	pub fn create(
		asset_id: H256,
		collection_id: u32,
		asset_type: u32,
		asset_sub_type: u32,
		dna: [u8; 32],
		minted_at: u32,
		level: Level,
	) -> Self {
		Self {
			asset_id,
			collection_id,
			asset_type,
			asset_sub_type,
			dna,
			minted_at,
			level,
			consumed: false,
		}
	}
}

impl Identifiable<AssetId> for Asset {
	fn get_id(&self) -> AssetId {
		self.asset_id
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum Level {
	One,
	Two,
	Three,
	Max,
}

pub const MAX_LEVEL_REACHED_ERROR: u8 = 100;
pub const ASSET_ALREADY_CONSUMED: u8 = 101;

impl Level {
	pub fn upgrade(self) -> Result<Self, sage_api::Error> {
		use Level::*;
		match self {
			One => Ok(Two),
			Two => Ok(Three),
			Three => Ok(Three),
			Max => Err(sage_api::Error::Transition { error: MAX_LEVEL_REACHED_ERROR }),
		}
	}
}

#[derive(
	Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, MaxEncodedLen, TypeInfo,
)]
pub enum ExampleTransitionId {
	UpgradeAsset,
	ConsumeAsset,
}

/// One specific transition that a game wants to execute.
pub fn consume_asset(asset: &mut Asset) -> Result<(), sage_api::Error> {
	if asset.consumed {
		Err(sage_api::Error::Transition { error: ASSET_ALREADY_CONSUMED })
	} else {
		asset.consumed = true;
		Ok(())
	}
}
