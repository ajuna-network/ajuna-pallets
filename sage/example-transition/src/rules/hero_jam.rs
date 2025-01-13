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

fn try_get_hero_jam<AccountId, BlockNumber, Inspector>(
	asset_id: &AssetId,
) -> Result<HeroJamAsset<BlockNumber>, RuleError>
where
	Inspector: AssetInspector<AccountId = AccountId, AssetId = AssetId, Asset = Asset<BlockNumber>>,
{
	let asset =
		Inspector::get_asset(asset_id).map_err(|_| RuleError::Other { error: ASSET_NOT_FOUND })?;
	asset.try_into().map_err(|_| RuleError::Other { error: ASSET_NOT_HERO_JAM })
}

fn try_get_hero_jam_vec<AccountId, BlockNumber, Inspector>(
	assets: &[AssetId],
) -> Result<Vec<HeroJamAsset<BlockNumber>>, RuleError>
where
	Inspector: AssetInspector<AccountId = AccountId, AssetId = AssetId, Asset = Asset<BlockNumber>>,
{
	assets
		.iter()
		.map(try_get_hero_jam::<_, _, Inspector>)
		.collect::<Result<Vec<_>, _>>()
}

pub(crate) fn ensure_all_asset_type<AccountId, BlockNumber, Inspector>(
	assets: &[AssetId],
	asset_type: AssetType,
) -> Result<(), RuleError>
where
	Inspector: AssetInspector<AccountId = AccountId, AssetId = AssetId, Asset = Asset<BlockNumber>>,
{
	let hero_jam_vec = try_get_hero_jam_vec::<_, _, Inspector>(assets)?;

	hero_jam_vec
		.iter()
		.all(|hero_jam_asset| hero_jam_asset.asset_type == asset_type)
		.then_some(())
		.ok_or(RuleError::Other { error: ASSETS_NOT_ALL_SAME_TYPE })
}

#[allow(dead_code)]
pub(crate) fn ensure_all_state_type<AccountId, BlockNumber, Inspector>(
	assets: &[AssetId],
	state_type: StateType,
) -> Result<(), RuleError>
where
	Inspector: AssetInspector<AccountId = AccountId, AssetId = AssetId, Asset = Asset<BlockNumber>>,
{
	let hero_jam_vec = try_get_hero_jam_vec::<_, _, Inspector>(assets)?;

	hero_jam_vec
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
	let hero_jam = try_get_hero_jam::<_, _, Inspector>(asset)?;

	if hero_jam.state_change_block_number < Chain::get_current_block_number() {
		Ok(())
	} else {
		Err(RuleError::Other { error: ASSET_CANNOT_STATE_CHANGE })
	}
}

#[inline]
fn account_has_asset_of_type<AccountId, BlockNumber, Inspector>(
	account_id: &AccountId,
	asset_type: AssetType,
) -> bool
where
	Inspector: AssetInspector<AccountId = AccountId, AssetId = AssetId, Asset = Asset<BlockNumber>>,
{
	Inspector::iter_assets_from(account_id).any(|asset_id| {
		if let Ok(hero_jam) = try_get_hero_jam::<_, _, Inspector>(&asset_id) {
			hero_jam.asset_type == asset_type
		} else {
			false
		}
	})
}

#[allow(dead_code)]
pub(crate) fn ensure_account_has_asset_of_type<AccountId, BlockNumber, Inspector>(
	account_id: &AccountId,
	asset_type: AssetType,
) -> Result<(), RuleError>
where
	Inspector: AssetInspector<AccountId = AccountId, AssetId = AssetId, Asset = Asset<BlockNumber>>,
{
	if account_has_asset_of_type::<_, _, Inspector>(account_id, asset_type) {
		Ok(())
	} else {
		Err(RuleError::Other { error: ASSET_HERO_NOT_IN_ACCOUNT })
	}
}

pub(crate) fn ensure_account_has_not_asset_of_type<AccountId, BlockNumber, Inspector>(
	account_id: &AccountId,
	asset_type: AssetType,
) -> Result<(), RuleError>
where
	Inspector: AssetInspector<AccountId = AccountId, AssetId = AssetId, Asset = Asset<BlockNumber>>,
{
	if account_has_asset_of_type::<_, _, Inspector>(account_id, asset_type) {
		Err(RuleError::Other { error: ASSET_HERO_ALREADY_IN_ACCOUNT })
	} else {
		Ok(())
	}
}
