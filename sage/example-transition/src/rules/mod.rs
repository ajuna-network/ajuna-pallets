use crate::asset;

use ajuna_primitives::asset_manager::{AssetInspector, AssetManager};
use sage_api::Error;

use frame_support::{
	ensure,
	pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo},
};
use parity_scale_codec::Codec;
use sp_runtime::traits::{BlockNumber as BlockNumberT, Member};
use std::marker::PhantomData;

pub mod hero_jam;

pub trait RuleVerifier {
	type AccountId;
	type AssetId;
	fn verify(
		&self,
		account_id: &Self::AccountId,
		asset_ids: &[Self::AssetId],
	) -> Result<(), Error>;
}

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum Rule<AccountId, TransitionRuleset, AssetHandler> {
	Not(Box<Self>),
	Or(Box<Self>, Box<Self>),
	And(Box<Self>, Box<Self>),
	AssetCount(u8),
	IsOwnerOf(AccountId, PhantomData<AssetHandler>),
	Transition(TransitionRuleset),
}

impl<AccountId, TransitionRuleset, BlockNumber, AssetHandler> RuleVerifier
	for Rule<AccountId, TransitionRuleset, AssetHandler>
where
	AccountId: Member + Codec,
	BlockNumber: BlockNumberT,
	TransitionRuleset: RuleVerifier<AccountId = AccountId, AssetId = asset::AssetId>,
	AssetHandler: AssetManager<
			AccountId = AccountId,
			AssetId = asset::AssetId,
			Asset = asset::Asset<BlockNumber>,
		> + AssetInspector<
			AccountId = AccountId,
			AssetId = asset::AssetId,
			Asset = asset::Asset<BlockNumber>,
		>,
{
	type AccountId = AccountId;
	type AssetId = asset::AssetId;

	fn verify(
		&self,
		account_id: &Self::AccountId,
		asset_ids: &[Self::AssetId],
	) -> Result<(), Error> {
		match self {
			Rule::<AccountId, TransitionRuleset, AssetHandler>::Not(rule) =>
				match rule.verify(account_id, asset_ids) {
					Ok(()) => Err(Error::FeeError),
					Err(_) => Ok(()),
				},
			Rule::<AccountId, TransitionRuleset, AssetHandler>::Or(rule_1, rule_2) => rule_1
				.verify(account_id, asset_ids)
				.or_else(|_| rule_2.verify(account_id, asset_ids)),
			Rule::<AccountId, TransitionRuleset, AssetHandler>::And(rule_1, rule_2) => rule_1
				.verify(account_id, asset_ids)
				.and_then(|_| rule_2.verify(account_id, asset_ids)),
			Rule::<AccountId, TransitionRuleset, AssetHandler>::AssetCount(expected_count) => {
				ensure!(asset_ids.len() as u8 == *expected_count, Error::InvalidAssetLength);
				Ok(())
			},
			Rule::<AccountId, TransitionRuleset, AssetHandler>::IsOwnerOf(owner_id, _) => asset_ids
				.iter()
				.all(|asset_id| AssetHandler::ensure_ownership(owner_id, asset_id).is_ok())
				.then_some(())
				.ok_or(Error::Transition { error: 0 }),
			Rule::<AccountId, TransitionRuleset, AssetHandler>::Transition(transition_ruleset) =>
				transition_ruleset.verify(account_id, asset_ids),
		}
	}
}
