use crate::{
	asset,
	asset::hero_jam::{AssetSubType, AssetType, StateType},
	rules,
	rules::{hero_jam as hero_jam_rules, RuleVerifier},
};

use ajuna_primitives::{
	asset_manager::{AssetInspector, AssetManager},
	chain_inspector::ChainInspector,
};
use sage_api::{traits::TransitionOutput, Error, SageGameTransition};

use frame_support::{
	pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo},
	sp_runtime,
};
use sp_runtime::SaturatedConversion;
use std::marker::PhantomData;

pub const BLOCKS_PER_HOUR: u32 = 600;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum ActionTime {
	Short = 1,
	Medium = 6,
	Long = 12,
}

impl ActionTime {
	pub fn get_block_time_from(&self) -> u32 {
		match self {
			ActionTime::Short => BLOCKS_PER_HOUR,
			ActionTime::Medium => 6 * BLOCKS_PER_HOUR,
			ActionTime::Long => 12 * BLOCKS_PER_HOUR,
		}
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum WorkType {
	Hunt = 1,
	Gather = 2,
	Build = 3,
	Travel = 4,
	Train = 5,
	Fight = 6,
	Rest = 7,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum HeroAction {
	Create,
	Sleep(ActionTime),
	Work(ActionTime, WorkType),
	Travel,
	Claim,
}

pub type BlockAmount = u8;

pub(super) struct HeroJamTransition<AccountId, BlockNumber, AssetHandler, ChainHandler> {
	_phantom: PhantomData<(AccountId, BlockNumber, AssetHandler, ChainHandler)>,
}

type HeroJamRule<AssetHandler, ChainHandler> = hero_jam_rules::Rule<AssetHandler, ChainHandler>;
type HeroJamRuleset<AccountId, AssetHandler, ChainHandler> =
	rules::Rule<AccountId, HeroJamRule<AssetHandler, ChainHandler>>;

impl<AccountId, BlockNumber, AssetHandler, ChainHandler>
	HeroJamTransition<AccountId, BlockNumber, AssetHandler, ChainHandler>
where
	AssetHandler: AssetManager<
			AccountId = AccountId,
			AssetId = asset::AssetId,
			Asset = asset::hero_jam::HeroJamAsset<BlockNumber>,
		> + AssetInspector<AssetId = asset::AssetId, Asset = asset::hero_jam::HeroJamAsset<BlockNumber>>,
	ChainHandler: ChainInspector<BlockNumber = BlockNumber>,
	BlockNumber: sp_runtime::traits::BlockNumber,
{
	fn verify(
		account_id: &AccountId,
		asset_ids: &[asset::AssetId],
		ruleset: &[HeroJamRuleset<AccountId, AssetHandler, ChainHandler>],
	) -> Result<(), Error> {
		ruleset
			.iter()
			.map(|rule| rule.verify(account_id, asset_ids))
			.collect::<Result<(), _>>()
	}

	fn transition(
		transition_id: &HeroAction,
		_account_id: &AccountId,
		asset_ids: &[asset::AssetId],
	) -> Result<
		Vec<TransitionOutput<asset::AssetId, asset::hero_jam::HeroJamAsset<BlockNumber>>>,
		Error,
	> {
		match transition_id {
			HeroAction::Create => {
				let asset = asset::hero_jam::HeroJamAsset {
					asset_type: AssetType::None,
					asset_subtype: AssetSubType::None,
					energy: 100,
					state_type: StateType::Idle,
					state_value: 0,
					state_change_block_number: 0,
				};
				Ok(vec![TransitionOutput::Minted(asset)])
			},
			HeroAction::Sleep(sleep_time) => {
				let mut asset = AssetHandler::get_asset(&asset_ids[0])
					.map_err(|_| Error::Transition { error: 0 })?;

				asset.state_type = StateType::Sleep;
				asset.state_value = *sleep_time as u8;
				asset.state_change_block_number = Self::get_block_time_for_action(sleep_time);

				Ok(vec![TransitionOutput::Mutated(asset_ids[0], asset)])
			},
			HeroAction::Work(work_time, work_type) => {
				let mut asset = AssetHandler::get_asset(&asset_ids[0])
					.map_err(|_| Error::Transition { error: 0 })?;

				asset.state_type = StateType::Sleep;
				asset.state_value = *work_type as u8;
				asset.state_change_block_number = Self::get_block_time_for_action(work_time);

				Ok(vec![TransitionOutput::Mutated(asset_ids[0], asset)])
			},
			_ => Ok(vec![]),
		}
	}

	fn get_block_time_for_action(action_time: &ActionTime) -> BlockNumber {
		let current_block = ChainHandler::get_current_block_number();
		let block_time = action_time.get_block_time_from();

		current_block.saturating_add(block_time.saturated_into())
	}
}

impl<AccountId, BlockNumber, AssetHandler, ChainHandler> SageGameTransition
	for HeroJamTransition<AccountId, BlockNumber, AssetHandler, ChainHandler>
where
	AssetHandler: AssetManager<
			AccountId = AccountId,
			AssetId = asset::AssetId,
			Asset = asset::hero_jam::HeroJamAsset<BlockNumber>,
		> + AssetInspector<AssetId = asset::AssetId, Asset = asset::hero_jam::HeroJamAsset<BlockNumber>>,
	ChainHandler: ChainInspector<BlockNumber = BlockNumber>,
{
	type TransitionId = HeroAction;
	type TransitionConfig = ();
	type AccountId = AccountId;
	type AssetId = asset::AssetId;
	type Asset = asset::hero_jam::HeroJamAsset<BlockNumber>;
	type Extra = ();

	fn verify_rule(
		transition_id: &Self::TransitionId,
		account_id: &Self::AccountId,
		asset_ids: &[Self::AssetId],
		_: &Self::Extra,
	) -> Result<(), Error> {
		let ruleset = match transition_id {
			HeroAction::Create => [
				HeroJamRuleset::AssetCount(0),
				HeroJamRuleset::Not(HeroJamRuleset::Transition(
					HeroJamRule::AccountHasAssetOfType(AssetType::Hero),
				)),
			]
			.as_slice(),
			HeroAction::Sleep(_) | HeroAction::Work(_, _) => [
				HeroJamRuleset::AssetCount(1),
				HeroJamRuleset::IsOwnerOf(account_id.clone()),
				HeroJamRuleset::Transition(HeroJamRule::AllAssetType(AssetType::Hero)),
				HeroJamRuleset::Transition(HeroJamRule::AllStateType(StateType::Idle)),
				HeroJamRuleset::Transition(HeroJamRule::CanStateChange),
			]
			.as_slice(),
			_ => [].as_slice(),
		};

		Self::verify(account_id, asset_ids, &ruleset)
	}

	fn do_transition(
		transition_id: &Self::TransitionId,
		account_id: &Self::AccountId,
		assets_ids: &[Self::AssetId],
		_: &Self::Extra,
	) -> Result<Vec<TransitionOutput<Self::AssetId, Self::Asset>>, Error> {
		Self::transition(transition_id, account_id, assets_ids)
	}
}
