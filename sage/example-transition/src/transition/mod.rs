use crate::asset;

use ajuna_primitives::asset_manager::{AssetInspector, AssetManager};
use frame_support::pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo};
use sage_api::{traits::TransitionOutput, Error, SageGameTransition};
use std::marker::PhantomData;

pub mod hero_jam;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum TransitionIdentifier {
	HeroJam(hero_jam::HeroAction),
}

pub struct GameTransition<AccountId, BlockNumber, AssetHandler> {
	_phantom: PhantomData<(AccountId, BlockNumber, AssetHandler)>,
}

impl<AccountId, BlockNumber, AssetHandler> SageGameTransition
	for GameTransition<AccountId, BlockNumber, AssetHandler>
{
	type TransitionId = TransitionIdentifier;
	type TransitionConfig = ();
	type AccountId = AccountId;
	type AssetId = asset::AssetId;
	type Asset = asset::Asset<BlockNumber>;
	type Extra = ();

	fn verify_rule(
		transition_id: &Self::TransitionId,
		account_id: &Self::AccountId,
		asset_ids: &[Self::AssetId],
		extra: &Self::Extra,
	) -> Result<(), Error> {
		match transition_id {
			TransitionIdentifier::HeroJam(hero_action) =>
				hero_jam::HeroJamTransition::verify_rule(hero_action, account_id, asset_ids, extra),
		}
	}

	fn do_transition(
		transition_id: &Self::TransitionId,
		account_id: &Self::AccountId,
		assets_ids: &[Self::AssetId],
		extra: &Self::Extra,
	) -> Result<Vec<TransitionOutput<Self::AssetId, Self::Asset>>, Error> {
		let outputs = match transition_id {
			TransitionIdentifier::HeroJam(hero_action) =>
				hero_jam::HeroJamTransition::do_transition(
					hero_action,
					account_id,
					assets_ids,
					extra,
				),
		};

		outputs.map(|outputs| outputs.into_iter().map(TransitionOutput::into).collect())
	}
}
