//! Some example transitions.
//!
//! These should be expanded to really showcase the power of the SageApi design.

use parity_scale_codec::{Decode, Encode, MaxEncodedLen};
use sage_api::{rules::ensure_asset_length, AssetT, SageApi, SageGameTransition};
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum ExampleTransitionId {
	UpgradeAsset,
	ConsumeAsset,
}

pub struct ExampleTransition<AccountId, Balance> {
	_phantom: PhantomData<(Balance, AccountId)>,
}

impl<AccountId, Balance> SageGameTransition for ExampleTransition<AccountId, Balance> {
	type Asset = Asset;
	type AccountId = AccountId;
	type Balance = Balance;

	type TransitionId = ExampleTransitionId;
	type Extra = ();

	fn verify_rule<
		Sage: SageApi<Asset = Self::Asset, AccountId = Self::AccountId, Balance = Self::Balance>,
	>(
		transition_id: Self::TransitionId,
		account: &Self::AccountId,
		assets: &[Self::Asset],
		_extra: &Self::Extra,
	) -> Result<(), sage_api::Error> {
		verify_transition_rule::<Sage>(transition_id, account, assets)
	}

	fn do_transition<
		Sage: SageApi<Asset = Self::Asset, AccountId = Self::AccountId, Balance = Self::Balance>,
	>(
		transition_id: Self::TransitionId,
		account: Self::AccountId,
		assets: Vec<Self::Asset>,
		_extra: Self::Extra,
	) -> Result<(), sage_api::Error> {
		transition::<Sage>(transition_id, account, assets)
	}
}

/// Verifies a transition rule with a given transition id.
pub fn verify_transition_rule<Sage: SageApi<Asset = Asset>>(
	transition_id: ExampleTransitionId,
	account: &Sage::AccountId,
	assets: &[Asset],
) -> Result<(), sage_api::Error> {
	use ExampleTransitionId::*;
	match transition_id {
		// use our rule provided in the sage api
		UpgradeAsset => {
			ensure_asset_length(assets, 1)?;
			Sage::ensure_ownership(account, &assets[0])
		},
		_ => Err(sage_api::Error::InvalidTransitionId),
	}
}

/// Executes a transition with a given transition id.
pub fn transition<Sage: SageApi<Asset = Asset>>(
	transition_id: ExampleTransitionId,
	_account: Sage::AccountId,
	assets: Vec<Asset>,
) -> Result<(), sage_api::Error> {
	use ExampleTransitionId::*;
	match transition_id {
		UpgradeAsset => upgrade_asset::<Sage>(assets),
		_ => Err(sage_api::Error::InvalidTransitionId),
	}
}

/// One specific transition that a game wants to execute.
pub fn upgrade_asset<Sage: SageApi<Asset = Asset>>(
	_assets: Vec<Asset>,
) -> Result<(), sage_api::Error> {
	todo!()
}
