//! Some example transitions.
//!
//! These should be expanded to really showcase the power of the SageApi design.

use crate::types::{consume_asset, Asset, AssetId, ExampleTransitionId};
use sage_api::{
	rules::ensure_asset_length, traits::AjunaSageCore, AccountId, SageApi, SageGameTransition,
};

pub struct ExampleTransition;
pub type ExampleTransitionSageCore = AjunaSageCore<AssetId, Asset>;

impl SageGameTransition for ExampleTransition {
	type AssetId = AssetId;
	type Asset = Asset;
	type SageApi = ExampleTransitionSageCore;

	type TransitionId = ExampleTransitionId;
	type Extra = ();

	fn verify_rule(
		transition_id: Self::TransitionId,
		account: &AccountId,
		asset_ids: &[Self::AssetId],
		_extra: &Self::Extra,
	) -> Result<(), sage_api::Error> {
		verify_transition_rule(transition_id, account, asset_ids)
	}

	fn do_transition(
		transition_id: Self::TransitionId,
		account: AccountId,
		asset_ids: Vec<Self::AssetId>,
		_extra: Self::Extra,
	) -> Result<(), sage_api::Error> {
		transition(transition_id, account, asset_ids)
	}
}

/// Verifies a transition rule with a given transition id.
pub fn verify_transition_rule(
	transition_id: ExampleTransitionId,
	account: &AccountId,
	assets: &[AssetId],
) -> Result<(), sage_api::Error> {
	use ExampleTransitionId::*;
	match transition_id {
		// use our rule provided in the sage api
		UpgradeAsset => {
			ensure_asset_length(assets, 1)?;
			ExampleTransitionSageCore::ensure_ownership(account, &assets[0])
		},
		ConsumeAsset => {
			ensure_asset_length(assets, 1)?;
			ExampleTransitionSageCore::ensure_ownership(account, &assets[0])
		},
	}
}

/// Executes a transition with a given transition id.
pub fn transition(
	transition_id: ExampleTransitionId,
	_account: AccountId,
	asset_ids: Vec<AssetId>,
) -> Result<(), sage_api::Error> {
	use ExampleTransitionId::*;
	match transition_id {
		UpgradeAsset => ExampleTransitionSageCore::try_mutate_asset(&asset_ids[0], |asset| {
			asset.level = asset.level.upgrade()?;
			Ok(())
		}),
		ConsumeAsset => ExampleTransitionSageCore::try_mutate_asset(&asset_ids[0], consume_asset),
	}
}
