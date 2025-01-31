use crate::{
	asset::{hero_jam::*, Asset, AssetId},
	rules::hero_jam::*,
};

use ajuna_primitives::{
	asset_manager::{AssetInspector, AssetManager},
	chain_inspector::ChainInspector,
};
use sage_api::{
	rules::{ensure_asset_length, ensure_owner_of},
	traits::TransitionOutput,
	SageGameTransition, TransitionError,
};

use crate::transition::GameTransitionConfig;
use ajuna_primitives::sage_api::SageApi;
use core::marker::PhantomData;
use frame_support::{
	pallet_prelude::{Decode, Encode, MaxEncodedLen, TypeInfo},
	sp_runtime,
};
use parity_scale_codec::Codec;
use sp_runtime::{
	traits::{BlockNumber as BlockNumberT, Member},
	SaturatedConversion,
};

const BLOCKS_PER_HOUR: u32 = 600;

const ASSET_NOT_FOUND: u8 = 200;
const ASSET_NOT_HERO_JAM: u8 = 201;

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
	Sleep(SleepType, ActionTime),
	Work(WorkType, ActionTime),
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

pub(super) struct HeroJamTransition<AccountId, BlockNumber, AssetHandler, ChainHandler, Sage> {
	_phantom: PhantomData<(AccountId, BlockNumber, AssetHandler, ChainHandler, Sage)>,
}

impl<AccountId, BlockNumber, AssetHandler, ChainHandler, Sage>
	HeroJamTransition<AccountId, BlockNumber, AssetHandler, ChainHandler, Sage>
where
	AccountId: Member + Codec,
	BlockNumber: BlockNumberT,
	AssetHandler: AssetManager<AccountId = AccountId, AssetId = AssetId, Asset = Asset<BlockNumber>>
		+ AssetInspector<AccountId = AccountId, AssetId = AssetId, Asset = Asset<BlockNumber>>,
	ChainHandler: ChainInspector<BlockNumber = BlockNumber>,
	Sage: SageApi<TransitionConfig = GameTransitionConfig>,
{
	fn try_get_hero_jam(asset_id: &AssetId) -> Result<HeroJamAsset<BlockNumber>, TransitionError> {
		let asset = AssetHandler::get_asset(asset_id)
			.map_err(|_| TransitionError::Transition { code: ASSET_NOT_FOUND })?;
		asset
			.try_into()
			.map_err(|_| TransitionError::Transition { code: ASSET_NOT_HERO_JAM })
	}

	fn try_get_hero_jam_assets(
		asset_ids: &[AssetId],
	) -> Result<Vec<(AssetId, HeroJamAsset<BlockNumber>)>, TransitionError> {
		asset_ids
			.iter()
			.copied()
			.map(|asset_id| match Self::try_get_hero_jam(&asset_id) {
				Ok(asset) => Ok((asset_id, asset)),
				Err(err) => Err(err),
			})
			.collect::<Result<Vec<_>, _>>()
	}

	fn verify_transition_rules(
		transition_id: &HeroAction,
		account_id: &AccountId,
		asset_ids: &[AssetId],
	) -> Result<Vec<(AssetId, HeroJamAsset<BlockNumber>)>, TransitionError> {
		let mut maybe_assets = None;

		match transition_id {
			HeroAction::Create => {
				ensure_asset_length(asset_ids, 0)?;
				ensure_account_has_not_asset_of_type::<_, _, AssetHandler>(
					account_id,
					AssetType::Hero,
				)?;
			},
			HeroAction::Sleep(_, _) | HeroAction::Work(_, _) => {
				let assets = Self::try_get_hero_jam_assets(asset_ids)?;

				ensure_asset_length(asset_ids, 1)?;
				ensure_owner_of::<_, _, AssetHandler>(asset_ids, account_id)?;
				ensure_all_asset_type(assets.as_slice(), AssetType::Hero)?;
				ensure_can_state_change::<_, ChainHandler>(&assets[0].1)?;

				maybe_assets = Some(assets);
			},
			_ => {},
		}

		if let Some(assets) = maybe_assets {
			Ok(assets)
		} else {
			Ok(Self::try_get_hero_jam_assets(asset_ids)?)
		}
	}

	fn transition_assets(
		transition_id: &HeroAction,
		_account_id: &AccountId,
		assets: Vec<(AssetId, HeroJamAsset<BlockNumber>)>,
	) -> Result<Vec<TransitionOutput<AssetId, Asset<BlockNumber>>>, TransitionError> {
		match transition_id {
			HeroAction::Create => {
				let asset_id = ChainHandler::get_current_block_number().saturated_into::<AssetId>();
				let asset = HeroJamAsset {
					id: asset_id,
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
				Ok(vec![TransitionOutput::Minted(Asset::from(asset))])
			},
			HeroAction::Sleep(_, sleep_time) => {
				let (asset_id, mut asset) = assets[0];

				asset.state_type = StateType::Sleep;
				asset.state_sub_type = SleepType::Normal as u8;
				asset.state_sub_value = *sleep_time as u8;
				asset.state_change_block_number = Self::get_block_time_for_action(sleep_time);

				Ok(vec![TransitionOutput::Mutated(asset_id, Asset::from(asset))])
			},
			HeroAction::Work(work_type, work_time) => {
				let (asset_id, mut asset) = assets[0];

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

				asset.fatigue = (asset.fatigue as i32).saturating_add(fatigue).clamp(0, 255) as u8;
				asset.energy = (asset.energy as i32).saturating_add(energy).clamp(0, 255) as u8;

				asset.state_type = StateType::Work;
				asset.state_sub_type = *work_type as u8;
				asset.state_sub_value = *work_time as u8;
				asset.state_change_block_number = Self::get_block_time_for_action(work_time);

				Ok(vec![TransitionOutput::Mutated(asset_id, Asset::from(asset))])
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
	AssetHandler: AssetManager<AccountId = AccountId, AssetId = AssetId, Asset = Asset<BlockNumber>>
		+ AssetInspector<AccountId = AccountId, AssetId = AssetId, Asset = Asset<BlockNumber>>,
	ChainHandler: ChainInspector<BlockNumber = BlockNumber>,
{
	type TransitionId = HeroAction;
	type TransitionConfig = ();
	type AccountId = AccountId;
	type AssetId = AssetId;
	type Asset = Asset<BlockNumber>;
	type Extra = ();

	fn do_transition(
		transition_id: &Self::TransitionId,
		account_id: &Self::AccountId,
		assets_ids: &[Self::AssetId],
		_: &Self::Extra,
	) -> Result<Vec<TransitionOutput<Self::AssetId, Self::Asset>>, TransitionError> {
		let assets = Self::verify_transition_rules(transition_id, account_id, assets_ids)?;
		Self::transition_assets(transition_id, account_id, assets)
	}
}
