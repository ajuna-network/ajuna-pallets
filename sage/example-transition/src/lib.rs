//! Some example transitions.
//!
//! These should be expanded to really showcase the power of the SageApi design.

use frame_support::ensure;
use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use sage_api::{AsErrorCode, AssetT, SageApi, SageGameTransition};
use scale_info::TypeInfo;
use std::marker::PhantomData;

/// Placeholder type, this was just a quick brain dump to get things going.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub struct Asset {
	pub collection_id: u32,

	pub asset_type: u32,

	pub asset_sub_type: u32,

	pub dna: [u8; 32],

	pub minted_at: u32,
}

impl AssetT for Asset {
	fn collection_id(&self) -> u32 {
		self.collection_id
	}

	fn asset_type(&self) -> u32 {
		self.asset_type
	}

	fn dna(&self) -> [u8; 32] {
		self.dna
	}

	fn minted_at(&self) -> u32 {
		self.minted_at
	}
}

pub struct ExampleTransition<AccountId, Balance> {
	_phantom: PhantomData<(Balance, AccountId)>,
}

impl<AccountId, Balance> SageGameTransition for ExampleTransition<AccountId, Balance> {
	type Asset = Asset;
	type AccountId = AccountId;
	type Balance = Balance;
	type Extra = ();
	type Error = u8;

	fn verify_rule<Sage: SageApi<Asset = Self::Asset>>(
		transition_id: u32,
		assets: &[Self::Asset],
		_extra: &Self::Extra,
	) -> Result<(), Self::Error> {
		verify_transition_rule::<Sage>(transition_id, assets).map_err(|e| e.as_error_code())
	}

	fn do_transition<Sage: SageApi<Asset = Self::Asset>>(
		transition_id: u32,
		assets: Vec<Self::Asset>,
		_extra: Self::Extra,
	) -> Result<(), Self::Error> {
		transition::<Sage>(transition_id, assets).map_err(|e| e.as_error_code())
	}
}

/// Verifies a transition rule with a given transition id.
pub fn verify_transition_rule<Sage: SageApi<Asset = Asset>>(
	transition_id: u32,
	assets: &[Asset],
) -> Result<(), sage_api::Error> {
	match transition_id {
		1 => rule_asset_length_1::<Sage>(assets),
		_ => Err(sage_api::Error::InvalidTransitionId),
	}
}

/// Executes a transition with a given transition id.
pub fn transition<Sage: SageApi<Asset = Asset>>(
	transition_id: u32,
	assets: Vec<Asset>,
) -> Result<(), sage_api::Error> {
	match transition_id {
		1 => transition_one::<Sage>(assets),
		_ => Err(sage_api::Error::InvalidTransitionId),
	}
}

/// One specific transition that a game wants to execute.
pub fn transition_one<Sage: SageApi<Asset = Asset>>(
	_assets: Vec<Asset>,
) -> Result<(), sage_api::Error> {
	todo!()
}

/// A rule that maybe many different transitions want to fulfill.
pub fn rule_asset_length_1<Sage: SageApi<Asset = Asset>>(
	assets: &[Asset],
) -> Result<(), sage_api::Error> {
	ensure!(assets.len() == 1, sage_api::Error::InvalidAssetLength);
	Ok(())
}
