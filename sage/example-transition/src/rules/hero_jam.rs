use crate::asset::{
	hero_jam::{AssetType, HeroJamAsset, StateType},
	Asset, AssetId,
};

use ajuna_primitives::{asset_manager::AssetInspector, chain_inspector::ChainInspector};
use sage_api::RuleError;
use sp_runtime::traits::BlockNumber as BlockNumberT;

pub const ASSET_NOT_FOUND: u8 = 200;
pub const ASSET_NOT_HERO_JAM: u8 = 201;

pub const ASSETS_NOT_ALL_SAME_TYPE: u8 = 100;
pub const ASSETS_NOT_ALL_SAME_STATE: u8 = 101;
pub const ASSET_CANNOT_STATE_CHANGE: u8 = 102;
pub const ASSET_HERO_NOT_IN_ACCOUNT: u8 = 103;
pub const ASSET_HERO_ALREADY_IN_ACCOUNT: u8 = 104;

fn try_cast_to_hero_jam_asset<AccountId, BlockNumber, Inspector>(
	asset_id: &AssetId,
) -> Result<HeroJamAsset<BlockNumber>, RuleError>
where
	Inspector: AssetInspector<AccountId = AccountId, AssetId = AssetId, Asset = Asset<BlockNumber>>,
{
	if let Ok(asset) = Inspector::get_asset(asset_id) {
		if let Ok(hero_jam_asset) = HeroJamAsset::try_from(asset) {
			Ok(hero_jam_asset)
		} else {
			Err(RuleError::Other { error: ASSET_NOT_HERO_JAM })
		}
	} else {
		Err(RuleError::Other { error: ASSET_NOT_FOUND })
	}
}

fn try_cast_to_hero_jam_assets<AccountId, BlockNumber, Inspector>(
	assets: &[AssetId],
) -> Result<Vec<HeroJamAsset<BlockNumber>>, RuleError>
where
	Inspector: AssetInspector<AccountId = AccountId, AssetId = AssetId, Asset = Asset<BlockNumber>>,
{
	assets
		.iter()
		.map(try_cast_to_hero_jam_asset::<_, _, Inspector>)
		.collect::<Result<Vec<_>, _>>()
}

pub(crate) fn ensure_all_asset_type<AccountId, BlockNumber, Inspector>(
	assets: &[AssetId],
	asset_type: AssetType,
) -> Result<(), RuleError>
where
	Inspector: AssetInspector<AccountId = AccountId, AssetId = AssetId, Asset = Asset<BlockNumber>>,
{
	let hero_jam_assets = try_cast_to_hero_jam_assets::<_, _, Inspector>(assets)?;

	hero_jam_assets
		.iter()
		.all(|hero_jam_asset| hero_jam_asset.asset_type == asset_type)
		.then_some(())
		.ok_or(RuleError::Other { error: ASSETS_NOT_ALL_SAME_TYPE })
}

pub(crate) fn ensure_all_state_type<AccountId, BlockNumber, Inspector>(
	assets: &[AssetId],
	state_type: StateType,
) -> Result<(), RuleError>
where
	Inspector: AssetInspector<AccountId = AccountId, AssetId = AssetId, Asset = Asset<BlockNumber>>,
{
	let hero_jam_assets = try_cast_to_hero_jam_assets::<_, _, Inspector>(assets)?;

	hero_jam_assets
		.iter()
		.all(|hero_jam_asset| hero_jam_asset.state_type == state_type)
		.then_some(())
		.ok_or(RuleError::Other { error: ASSETS_NOT_ALL_SAME_STATE })
}

pub(crate) fn ensure_can_state_change<AccountId, BlockNumber, Inspector, Chain>(
	asset: &AssetId,
) -> Result<(), RuleError>
where
	BlockNumber: BlockNumberT,
	Inspector: AssetInspector<AccountId = AccountId, AssetId = AssetId, Asset = Asset<BlockNumber>>,
	Chain: ChainInspector<BlockNumber = BlockNumber>,
{
	let hero_jam_asset = try_cast_to_hero_jam_asset::<_, _, Inspector>(asset)?;

	if hero_jam_asset.state_change_block_number < Chain::get_current_block_number() {
		Ok(())
	} else {
		Err(RuleError::Other { error: ASSET_CANNOT_STATE_CHANGE })
	}
}

pub(crate) fn ensure_account_has_asset_of_type<AccountId, BlockNumber, Inspector>(
	account_id: &AccountId,
	asset_type: AssetType,
) -> Result<(), RuleError>
where
	Inspector: AssetInspector<AccountId = AccountId, AssetId = AssetId, Asset = Asset<BlockNumber>>,
{
	if Inspector::iter_assets_from(account_id)
		.filter(|asset_id| {
			if let Ok(asset) = try_cast_to_hero_jam_asset::<_, _, Inspector>(asset_id) {
				asset.asset_type == asset_type
			} else {
				false
			}
		})
		.count()
		.eq(&1)
	{
		Ok(())
	} else {
		Err(RuleError::Other { error: ASSET_HERO_NOT_IN_ACCOUNT })
	}
}
