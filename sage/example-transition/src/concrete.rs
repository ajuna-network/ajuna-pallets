//! An example transition that uses the concrete types of our Ajuna runtime.
//!
//! This is done to make the life of our downstream implementors a bit easier.
//!
//! However, this forces us to use the same types in our mock runtimes for tests.

use crate::types::{consume_asset, Asset, AssetId, ExampleTransitionId};
use core::marker::PhantomData;
use sage_api::{rules::ensure_asset_length, AccountId, Balance, SageApi, SageGameTransition};

pub struct ExampleTransition<SageApi> {
	_phantom: PhantomData<SageApi>,
}

// Constrain the generic Sage Api to our concrete types
pub trait ExampleTransitionSage:
	SageApi<Balance = Balance, AccountId = AccountId, AssetId = AssetId, Asset = Asset>
{
}

impl<Sage> SageGameTransition for ExampleTransition<Sage>
where
	Sage: ExampleTransitionSage,
{
	type AssetId = AssetId;
	type Asset = Asset;

	type SageApi = Sage;

	type TransitionId = ExampleTransitionId;
	type TransitionConfig = ();
	type Extra = ();

	fn verify_rule(
		transition_id: Self::TransitionId,
		account: &AccountId,
		asset_ids: &[Self::AssetId],
		_extra: &Self::Extra,
	) -> Result<(), sage_api::Error> {
		verify_transition_rule::<Self::SageApi>(transition_id, account, asset_ids)
	}

	fn do_transition(
		transition_id: Self::TransitionId,
		account: AccountId,
		asset_ids: Vec<Self::AssetId>,
		_extra: Self::Extra,
	) -> Result<(), sage_api::Error> {
		transition::<Self::SageApi>(transition_id, account, asset_ids)
	}
}

/// Verifies a transition rule with a given transition id.
pub fn verify_transition_rule<Sage: ExampleTransitionSage>(
	transition_id: ExampleTransitionId,
	account: &AccountId,
	assets: &[AssetId],
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
pub fn transition<Sage: ExampleTransitionSage>(
	transition_id: ExampleTransitionId,
	_account: AccountId,
	asset_ids: Vec<AssetId>,
) -> Result<(), sage_api::Error> {
	use ExampleTransitionId::*;
	match transition_id {
		UpgradeAsset => Sage::try_mutate_asset(&asset_ids[0], |asset| {
			asset.level = asset.level.upgrade()?;
			Ok(())
		}),
		ConsumeAsset => Sage::try_mutate_asset(&asset_ids[0], consume_asset),
	}
}
