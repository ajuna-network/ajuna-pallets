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

	// Example of a game's custom field.
	pub level: Level,

	// Example of a game's custom field.
	pub consumed: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum Level {
	One,
	Two,
	Three,
	Max,
}

const MAX_LEVEL_REACHED_ERROR: u8 = 100;
const ASSET_ALREADY_CONSUMED: u8 = 101;

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
		ConsumeAsset => {
			ensure_asset_length(assets, 1)?;
			Sage::ensure_ownership(account, &assets[0])
		},
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
		// Todo: this is not persisted anywhere. We have to figure out how to persist
		// mutated assets.
		UpgradeAsset => {
			let _upgraded = assets[0].level.upgrade();
			// todo what to do with mutated asset.
			// We probably want to have a `Sage::store_asset()`
			Ok(())
		},
		_ => Err(sage_api::Error::InvalidTransitionId),
	}
}

/// One specific transition that a game wants to execute.
pub fn consume_asset<Sage: SageApi<Asset = Asset>>(
	assets: Vec<Asset>,
) -> Result<(), sage_api::Error> {
	// make mut
	let mut assets = assets;

	if assets[0].consumed {
		Err(sage_api::Error::Transition { error: ASSET_ALREADY_CONSUMED })
	} else {
		assets[0].consumed = true;
		Ok(())
	}

	// todo what to do with mutated asset.
	// We probably want to have a `Sage::store_asset()`
}
