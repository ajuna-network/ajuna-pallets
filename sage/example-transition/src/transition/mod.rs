use crate::asset;

use sage_api::{traits::TransitionOutput, SageGameTransition, TransitionError};

use crate::asset::{Asset, AssetId};
use ajuna_primitives::sage_api::SageApi;
use core::marker::PhantomData;
use frame_support::{
	pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo},
	traits::tokens::Balance as BalanceT,
};
use parity_scale_codec::Codec;
use sp_runtime::traits::{BlockNumber as BlockNumberT, Member};

pub mod hero_jam;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum TransitionIdentifier {
	HeroJam(hero_jam::HeroAction),
}

pub struct GameTransition<AccountId, BlockNumber, Balance, Sage> {
	_phantom: PhantomData<(AccountId, BlockNumber, Balance, Sage)>,
}

/// This is an example how a transition config custom to a game could look like.
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Debug, Default, Clone, PartialEq, Eq)]
pub struct GameTransitionConfig<Balance> {
	pub hunting_reward: Balance,
}

impl<AccountId, BlockNumber, Balance, Sage> SageGameTransition
	for GameTransition<AccountId, BlockNumber, Balance, Sage>
where
	AccountId: Member + Codec,
	BlockNumber: BlockNumberT,
	Balance: BalanceT,
	Sage: SageApi<
		TransitionConfig = GameTransitionConfig<Balance>,
		AccountId = AccountId,
		AssetId = AssetId,
		Balance = Balance,
		Asset = Asset<BlockNumber, Balance>,
		BlockNumber = BlockNumber,
	>,
{
	type TransitionId = TransitionIdentifier;
	type TransitionConfig = GameTransitionConfig<Balance>;
	type AccountId = AccountId;
	type AssetId = asset::AssetId;
	type Asset = asset::Asset<BlockNumber, Balance>;
	type Extra = ();

	fn do_transition(
		transition_id: &Self::TransitionId,
		account_id: &Self::AccountId,
		assets_ids: &[Self::AssetId],
		extra: Option<Self::Extra>,
	) -> Result<Vec<TransitionOutput<Self::AssetId, Self::Asset>>, TransitionError> {
		match transition_id {
			TransitionIdentifier::HeroJam(hero_action) =>
				hero_jam::HeroJamTransition::<AccountId, BlockNumber, Sage>::do_transition(
					hero_action,
					account_id,
					assets_ids,
					extra,
				),
		}
	}
}
