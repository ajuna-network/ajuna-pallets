use crate::asset;
use ajuna_primitives::asset_manager::{AssetInspector, AssetManager};
use frame_support::{
	ensure,
	pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo},
};
use sage_api::Error;

pub mod hero_jam;

pub trait RuleVerifier {
	type AccountId;
	type Asset;
	fn verify(&self, account_id: &Self::AccountId, assets: &[Self::Asset]) -> Result<(), Error>;
}

#[derive(Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum Rule<AccountId, TransitionRuleset> {
	Not(Box<Self>),
	Or(Box<Self>, Box<Self>),
	And(Box<Self>, Box<Self>),
	AssetCount(u8),
	IsOwnerOf(AccountId),
	Transition(TransitionRuleset),
}

impl<AccountId, TransitionRuleset, BlockNumber, AssetHandler> RuleVerifier
	for Rule<AccountId, TransitionRuleset>
where
	TransitionRuleset: RuleVerifier<AccountId = AccountId, Asset = asset::Asset<BlockNumber>>,
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
	type Asset = asset::Asset<BlockNumber>;

	fn verify(&self, account_id: &Self::AccountId, assets: &[Self::Asset]) -> Result<(), Error> {
		match self {
			Rule::Not(rule) => match rule.verify(account_id, assets) {
				Ok(()) => Err(Error::FeeError),
				Err(_) => Ok(()),
			},
			Rule::Or(rule_1, rule_2) =>
				rule_1.verify(account_id, assets).or_else(|_| rule_2.verify(account_id, assets)),
			Rule::And(rule_1, rule_2) => rule_1
				.verify(account_id, assets)
				.and_then(|_| rule_2.verify(account_id, assets)),
			Rule::AssetCount(expected_count) => {
				ensure!(assets.len() as u8 == *expected_count, Error::InvalidAssetLength);
				Ok(())
			},
			Rule::IsOwnerOf(owner_id) => assets
				.iter()
				.all(|asset_id| AssetHandler::ensure_ownership(owner_id, asset_id).is_ok())
				.then_some(())
				.ok_or(Error::Transition { error: 0 }),
			Rule::Transition(transition_ruleset) => transition_ruleset.verify(account_id, assets),
		}
	}
}
