use crate::asset;

use ajuna_primitives::{
	asset_manager::{AssetInspector, AssetManager},
	chain_inspector::ChainInspector,
};
use sage_api::{traits::TransitionOutput, SageGameTransition, TransitionError};

use frame_support::pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo};
use parity_scale_codec::Codec;
use sp_runtime::traits::{BlockNumber as BlockNumberT, Member};
use sp_std::{marker::PhantomData, vec::Vec};

pub mod hero_jam;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum TransitionIdentifier {
	HeroJam(hero_jam::HeroAction),
}

pub struct GameTransition<AccountId, BlockNumber, AssetHandler, ChainHandler> {
	_phantom: PhantomData<(AccountId, BlockNumber, AssetHandler, ChainHandler)>,
}

impl<AccountId, BlockNumber, AssetHandler, ChainHandler> SageGameTransition
	for GameTransition<AccountId, BlockNumber, AssetHandler, ChainHandler>
where
	AccountId: Member + Codec,
	BlockNumber: BlockNumberT,
	AssetHandler: AssetManager<
			AccountId = AccountId,
			AssetId = asset::AssetId,
			Asset = asset::Asset<BlockNumber>,
		> + AssetInspector<
			AccountId = AccountId,
			AssetId = asset::AssetId,
			Asset = asset::Asset<BlockNumber>,
		>,
	ChainHandler: ChainInspector<BlockNumber = BlockNumber>,
{
	type TransitionId = TransitionIdentifier;
	type TransitionConfig = ();
	type AccountId = AccountId;
	type AssetId = asset::AssetId;
	type Asset = asset::Asset<BlockNumber>;
	type Extra = ();

	fn do_transition(
		transition_id: &Self::TransitionId,
		account_id: &Self::AccountId,
		assets_ids: &[Self::AssetId],
		extra: &Self::Extra,
	) -> Result<Vec<TransitionOutput<Self::AssetId, Self::Asset>>, TransitionError> {
		match transition_id {
			TransitionIdentifier::HeroJam(hero_action) => hero_jam::HeroJamTransition::<
				AccountId,
				BlockNumber,
				AssetHandler,
				ChainHandler,
			>::do_transition(
				hero_action,
				account_id,
				assets_ids,
				extra,
			),
		}
	}
}
