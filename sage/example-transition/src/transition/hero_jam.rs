use crate::{
	asset,
	asset::hero_jam::{AssetSubType, AssetType, StateType},
	rules,
	rules::hero_jam as hero_jam_rules,
};

use ajuna_primitives::{
	asset_manager::{AssetInspector, AssetManager},
	chain_inspector::ChainInspector,
};
use sage_api::{traits::TransitionOutput, Error, SageGameTransition};

use crate::{asset::AssetVariant, rules::RuleVerifier};
use frame_support::{
	pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo},
	sp_runtime,
};
use parity_scale_codec::Codec;
use sp_runtime::{
	traits::{BlockNumber as BlockNumberT, Member},
	SaturatedConversion,
};
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

impl From<u8> for WorkType {
	fn from(value: u8) -> Self {
		match value {
			1 => WorkType::Hunt,
			2 => WorkType::Gather,
			3 => WorkType::Build,
			4 => WorkType::Travel,
			5 => WorkType::Train,
			6 => WorkType::Fight,
			7 => WorkType::Rest,
			_ => WorkType::Hunt,
		}
	}
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum HeroAction {
	Create,
	Sleep(ActionTime),
	Work(ActionTime, WorkType),
	Travel,
	Claim,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Encode, Decode, MaxEncodedLen, TypeInfo)]
pub enum SleepType {
	None = 0,
	Normal = 1,
}

impl From<u8> for SleepType {
	fn from(value: u8) -> Self {
		match value {
			0 => SleepType::None,
			1 => SleepType::Normal,
			_ => SleepType::None,
		}
	}
}

pub(super) struct HeroJamTransition<AccountId, BlockNumber, AssetHandler, ChainHandler> {
	_phantom: PhantomData<(AccountId, BlockNumber, AssetHandler, ChainHandler)>,
}

type HeroJamRule<AccountId, AssetHandler, ChainHandler> =
	hero_jam_rules::Rule<AccountId, AssetHandler, ChainHandler>;
type HeroJamRuleset<AccountId, AssetHandler, ChainHandler> =
	rules::Rule<AccountId, HeroJamRule<AccountId, AssetHandler, ChainHandler>, AssetHandler>;

impl<AccountId, BlockNumber, AssetHandler, ChainHandler>
	HeroJamTransition<AccountId, BlockNumber, AssetHandler, ChainHandler>
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
	fn verify(
		account_id: &AccountId,
		asset_ids: &[asset::AssetId],
		ruleset: &[HeroJamRuleset<AccountId, AssetHandler, ChainHandler>],
	) -> Result<(), Error> {
		ruleset.iter().try_for_each(|rule| rule.verify(account_id, asset_ids))
	}

	fn transition(
		transition_id: &HeroAction,
		_account_id: &AccountId,
		asset_ids: &[asset::AssetId],
	) -> Result<Vec<TransitionOutput<asset::AssetId, asset::Asset<BlockNumber>>>, Error> {
		match transition_id {
			HeroAction::Create => {
				let asset = asset::hero_jam::HeroJamAsset {
					asset_type: AssetType::None,
					asset_subtype: AssetSubType::None,
					energy: 100,
					fatigue: 0,
					state_type: StateType::None,
					state_sub_type: 0,
					state_sub_value: 0,
					state_change_block_number: 0_u32.saturated_into::<BlockNumber>(),
					balance: 10,
				};
				Ok(vec![TransitionOutput::Minted(asset::Asset::from(asset))])
			},
			HeroAction::Sleep(sleep_time) => {
				match AssetHandler::get_asset(&asset_ids[0])
					.map_err(|_| Error::Transition { error: 0 })?
					.asset_variant
				{
					AssetVariant::HeroJam(mut asset) => {
						asset.state_type = StateType::Sleep;
						asset.state_sub_type = SleepType::Normal as u8;
						asset.state_sub_value = *sleep_time as u8;
						asset.state_change_block_number =
							Self::get_block_time_for_action(sleep_time);

						Ok(vec![TransitionOutput::Mutated(asset_ids[0], asset::Asset::from(asset))])
					},
				}
			},
			HeroAction::Work(work_time, work_type) => {
				match AssetHandler::get_asset(&asset_ids[0])
					.map_err(|_| Error::Transition { error: 0 })?
					.asset_variant
				{
					AssetVariant::HeroJam(mut asset) => {
						let fatigue = Self::get_resource_fatigue(
							asset.state_type,
							asset.state_sub_type,
							asset.state_sub_value,
						);
						let energy = Self::get_resource_energy(
							asset.state_type,
							asset.state_sub_type,
							asset.state_sub_value,
						);

						if work_type == &WorkType::Hunt {
							asset.balance = asset.balance.saturating_add(10);
						}

						asset.fatigue =
							(asset.fatigue as i32).saturating_add(fatigue).clamp(0, 255) as u8;
						asset.energy =
							(asset.energy as i32).saturating_add(energy).clamp(0, 255) as u8;

						asset.state_type = StateType::Work;
						asset.state_sub_type = *work_type as u8;
						asset.state_sub_value = *work_time as u8;
						asset.state_change_block_number =
							Self::get_block_time_for_action(work_time);

						Ok(vec![TransitionOutput::Mutated(asset_ids[0], asset::Asset::from(asset))])
					},
				}
			},
			_ => Ok(vec![]),
		}
	}

	fn get_block_time_for_action(action_time: &ActionTime) -> BlockNumber {
		let current_block = ChainHandler::get_current_block_number();
		let block_time = action_time.get_block_time_from();

		current_block.saturating_add(block_time.saturated_into())
	}

	fn get_resource_fatigue(state_type: StateType, state_sub_type: u8, action_time: u8) -> i32 {
		match state_type {
			StateType::None => 0,
			StateType::Sleep => match SleepType::from(state_sub_type) {
				SleepType::Normal => -((action_time as i32) * (action_time as i32) * 11),
				_ => 0,
			},
			StateType::Work => (action_time as i32) * 8,
		}
	}

	fn get_resource_energy(state_type: StateType, state_sub_type: u8, action_time: u8) -> i32 {
		match state_type {
			StateType::None => 0,
			StateType::Sleep => match SleepType::from(state_sub_type) {
				SleepType::Normal => -(action_time as i32),
				_ => 0,
			},
			StateType::Work => match WorkType::from(state_sub_type) {
				WorkType::Hunt => -((action_time as i32) * 6),
				_ => 0,
			},
		}
	}
}

impl<AccountId, BlockNumber, AssetHandler, ChainHandler> SageGameTransition
	for HeroJamTransition<AccountId, BlockNumber, AssetHandler, ChainHandler>
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
	type TransitionId = HeroAction;
	type TransitionConfig = ();
	type AccountId = AccountId;
	type AssetId = asset::AssetId;
	type Asset = asset::Asset<BlockNumber>;
	type Extra = ();

	fn verify_rule(
		transition_id: &Self::TransitionId,
		account_id: &Self::AccountId,
		asset_ids: &[Self::AssetId],
		_: &Self::Extra,
	) -> Result<(), Error> {
		let ruleset = match transition_id {
			HeroAction::Create => vec![
				HeroJamRuleset::AssetCount(0),
				HeroJamRuleset::Not(Box::new(HeroJamRuleset::Transition(
					HeroJamRule::AccountHasAssetOfType(AssetType::Hero, PhantomData),
				))),
			],
			HeroAction::Sleep(_) | HeroAction::Work(_, _) => vec![
				HeroJamRuleset::AssetCount(1),
				HeroJamRuleset::IsOwnerOf(account_id.clone(), PhantomData),
				HeroJamRuleset::Transition(HeroJamRule::AllAssetType(AssetType::Hero)),
				HeroJamRuleset::Transition(HeroJamRule::CanStateChange(PhantomData)),
			],
			_ => vec![],
		};

		Self::verify(account_id, asset_ids, ruleset.as_slice())
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
