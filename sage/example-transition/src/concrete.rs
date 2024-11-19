//! An example transition that uses the concrete types of our Ajuna runtime.
//!
//! This is done to make the life of our downstream implementors a bit easier.
//!
//! However, this forces us to use the same types in our mock runtimes for tests.

use crate::types::{consume_asset, Asset, AssetId, ExampleTransitionId};
use ajuna_primitives::asset_manager::{AssetInspector, AssetManager};
use sage_api::{
	rules::ensure_asset_length, traits::TransitionOutput, AccountId, SageGameTransition,
};
use std::marker::PhantomData;

pub struct ExampleTransition<AssetHandler> {
	_phantom: PhantomData<AssetHandler>,
}

impl<AssetHandler> ExampleTransition<AssetHandler>
where
	AssetHandler: AssetManager<AccountId = AccountId, AssetId = AssetId, Asset = Asset>
		+ AssetInspector<AssetId = AssetId, Asset = Asset>,
{
	/// Verifies a transition rule with a given transition id.
	pub fn verify_transition_rule(
		transition_id: &ExampleTransitionId,
		account: &AccountId,
		assets: &[AssetId],
	) -> Result<(), sage_api::Error> {
		use ExampleTransitionId::*;
		match *transition_id {
			// use our rule provided in the sage api
			UpgradeAsset => {
				ensure_asset_length(assets, 1)?;
				let _ = AssetHandler::ensure_ownership(account, &assets[0])
					.map_err(|_| sage_api::Error::InvalidTransitionId)?;
				Ok(())
			},
			ConsumeAsset => {
				ensure_asset_length(assets, 1)?;
				let _ = AssetHandler::ensure_ownership(account, &assets[0])
					.map_err(|_| sage_api::Error::InvalidTransitionId)?;
				Ok(())
			},
		}
	}

	/// Executes a transition with a given transition id.
	pub fn transition(
		transition_id: &ExampleTransitionId,
		_account: &AccountId,
		asset_ids: &[AssetId],
	) -> Result<Vec<TransitionOutput<AssetId, Asset>>, sage_api::Error> {
		use ExampleTransitionId::*;

		let mut asset = AssetHandler::get_asset(&asset_ids[0])
			.map_err(|_| sage_api::Error::Transition { error: 0 })?;

		match transition_id {
			UpgradeAsset => {
				asset.level = asset.level.upgrade()?;
			},
			ConsumeAsset => {
				consume_asset(&mut asset)?;
			},
		}

		Ok(vec![TransitionOutput::Mutated(asset_ids[0], asset)])
	}
}

impl<AssetHandler> SageGameTransition for ExampleTransition<AssetHandler>
where
	AssetHandler: AssetManager<AccountId = AccountId, AssetId = AssetId, Asset = Asset>
		+ AssetInspector<AssetId = AssetId, Asset = Asset>,
{
	type TransitionId = ExampleTransitionId;
	type AccountId = AccountId;

	type AssetId = AssetId;

	type Asset = Asset;
	type Extra = ();

	fn verify_rule(
		transition_id: &Self::TransitionId,
		account: &AccountId,
		asset_ids: &[Self::AssetId],
		_extra: &Self::Extra,
	) -> Result<(), sage_api::Error> {
		Self::verify_transition_rule(transition_id, account, asset_ids)
	}

	fn do_transition(
		transition_id: &Self::TransitionId,
		account: &AccountId,
		asset_ids: &[Self::AssetId],
		_extra: &Self::Extra,
	) -> Result<Vec<TransitionOutput<Self::AssetId, Self::Asset>>, sage_api::Error> {
		Self::transition(transition_id, account, asset_ids)
	}
}
