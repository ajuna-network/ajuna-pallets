//! Generic example transition.
//!
//! The pattern here follows the generic style of frame, and is the easiest to integrate
//! within frame. However, it is a bit harder to understand for downstream implementors.

use crate::types::{consume_asset, Asset, AssetId, ExampleTransitionId};
use sage_api::{rules::ensure_asset_length, traits::AccountIdOf, SageApi, SageGameTransition};
use std::marker::PhantomData;

pub struct ExampleTransitionGeneric<Balance, AccountId, SageApi> {
	phantom_data: PhantomData<(Balance, AccountId, SageApi)>,
}

impl<Balance, AccountId, Sage> SageGameTransition
	for ExampleTransitionGeneric<Balance, AccountId, Sage>
where
	Sage: SageApi<AssetId = AssetId, Asset = Asset, Balance = Balance, AccountId = AccountId>,
{
	type AssetId = AssetId;
	type Asset = Asset;

	type SageApi = Sage;

	type TransitionId = ExampleTransitionId;
	type Extra = ();

	fn verify_rule(
		transition_id: Self::TransitionId,
		account: &AccountIdOf<Self>,
		asset_ids: &[Self::AssetId],
		_extra: &Self::Extra,
	) -> Result<(), sage_api::Error> {
		verify_transition_rule::<Self::SageApi>(transition_id, account, asset_ids)
	}

	fn do_transition(
		transition_id: Self::TransitionId,
		account: AccountIdOf<Self>,
		asset_ids: Vec<Self::AssetId>,
		_extra: Self::Extra,
	) -> Result<(), sage_api::Error> {
		transition::<Self::SageApi>(transition_id, account, asset_ids)
	}
}

/// Verifies a transition rule with a given transition id.
pub fn verify_transition_rule<Sage: SageApi<AssetId = AssetId, Asset = Asset>>(
	transition_id: ExampleTransitionId,
	account: &Sage::AccountId,
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
pub fn transition<Sage: SageApi<AssetId = AssetId, Asset = Asset>>(
	transition_id: ExampleTransitionId,
	_account: Sage::AccountId,
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
