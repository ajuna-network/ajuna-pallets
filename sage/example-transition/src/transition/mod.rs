use crate::asset;

use ajuna_primitives::{
	asset_manager::{AssetInspector, AssetManager},
	chain_inspector::ChainInspector,
};
use sage_api::{traits::TransitionOutput, SageGameTransition, TransitionError};

use crate::asset::AssetId;
use ajuna_primitives::{
	asset_manager::AssetFundsManager, payment_handler::NativeId, sage_api::SageApi,
};
use core::marker::PhantomData;
use frame_support::pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo};
use parity_scale_codec::Codec;
use sp_runtime::traits::{BlockNumber as BlockNumberT, Member};

pub mod hero_jam;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum TransitionIdentifier {
	HeroJam(hero_jam::HeroAction),
}

pub struct GameTransition<AccountId, BlockNumber, AssetHandler, ChainHandler, Sage> {
	_phantom: PhantomData<(AccountId, BlockNumber, AssetHandler, ChainHandler, Sage)>,
}

/// This is an example how a transition config custom to a game could look like.
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Debug, Default, Copy, Clone, PartialEq, Eq)]
pub struct GameTransitionConfig {
	pub game_fee: u64,
}

impl<AccountId, BlockNumber, AssetHandler, ChainHandler, Sage> SageGameTransition
	for GameTransition<AccountId, BlockNumber, AssetHandler, ChainHandler, Sage>
where
	AccountId: Member + Codec,
	BlockNumber: BlockNumberT,
	AssetHandler: AssetManager<AccountId = AccountId, AssetId = AssetId, Asset = asset::Asset<BlockNumber>>
		+ AssetInspector<AccountId = AccountId, AssetId = AssetId, Asset = asset::Asset<BlockNumber>>,
	ChainHandler: ChainInspector<BlockNumber = BlockNumber>,
	Sage: SageApi<TransitionConfig = GameTransitionConfig>
		+ AssetFundsManager<AccountId = AccountId, AssetId = AssetId, Balance = u64>,
	<Sage as AssetFundsManager>::FungiblesAssetId: NativeId,
{
	type TransitionId = TransitionIdentifier;
	type TransitionConfig = GameTransitionConfig;
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
			TransitionIdentifier::HeroJam(hero_action) =>
				hero_jam::HeroJamTransition::<
					AccountId,
					BlockNumber,
					AssetHandler,
					ChainHandler,
					Sage,
				>::do_transition(hero_action, account_id, assets_ids, extra),
		}
	}
}
