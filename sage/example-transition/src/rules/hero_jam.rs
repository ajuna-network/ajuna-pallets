use crate::asset::{
	hero_jam::{AssetType, HeroJamAsset, StateType},
	Asset, AssetId, AssetVariant,
};

use ajuna_primitives::{asset_manager::AssetInspector, chain_inspector::ChainInspector};
use sage_api::TransitionError;

use crate::asset::hero_jam::VariantType;
use sp_runtime::traits::BlockNumber as BlockNumberT;

pub const ASSETS_NOT_ALL_SAME_TYPE: u8 = 100;
pub const ASSETS_NOT_ALL_SAME_STATE: u8 = 101;
pub const ASSET_CANNOT_STATE_CHANGE: u8 = 102;
pub const ASSET_HERO_NOT_IN_ACCOUNT: u8 = 103;
pub const ASSET_HERO_ALREADY_IN_ACCOUNT: u8 = 104;

pub(crate) fn ensure_all_asset_type<BlockNumber>(
	assets: &[(AssetId, HeroJamAsset<BlockNumber>)],
	asset_type: VariantType,
) -> Result<(), TransitionError> {
	assets
		.iter()
		.all(|(_, hero_jam)| hero_jam.is_variant(asset_type))
		.then_some(())
		.ok_or(TransitionError::Transition { code: ASSETS_NOT_ALL_SAME_TYPE })
}

#[allow(dead_code)]
pub(crate) fn ensure_all_state_type<BlockNumber>(
	assets: &[(AssetId, HeroJamAsset<BlockNumber>)],
	state_type: StateType,
) -> Result<(), TransitionError>
where
{
	assets
		.iter()
		.all(|(_, hero_jam)| hero_jam.state_type == state_type)
		.then_some(())
		.ok_or(TransitionError::Transition { code: ASSETS_NOT_ALL_SAME_STATE })
}

pub(crate) fn ensure_can_state_change<BlockNumber, Chain>(
	asset: &HeroJamAsset<BlockNumber>,
) -> Result<(), TransitionError>
where
	BlockNumber: BlockNumberT,
	Chain: ChainInspector<BlockNumber = BlockNumber>,
{
	if asset.state_change_block_number < Chain::get_current_block_number() {
		Ok(())
	} else {
		Err(TransitionError::Transition { code: ASSET_CANNOT_STATE_CHANGE })
	}
}

#[inline]
fn account_has_asset_of_type<AccountId, BlockNumber, Inspector>(
	account_id: &AccountId,
	asset_type: VariantType,
) -> bool
where
	Inspector: AssetInspector<AccountId = AccountId, AssetId = AssetId, Asset = Asset<BlockNumber>>,
{
	Inspector::iter_assets_from(account_id).any(|(_, asset)| match asset.asset_variant {
		AssetVariant::HeroJam(hero_jam) => hero_jam.is_variant(asset_type),
	})
}

#[allow(dead_code)]
pub(crate) fn ensure_account_has_asset_of_type<AccountId, BlockNumber, Inspector>(
	account_id: &AccountId,
	asset_type: VariantType,
) -> Result<(), TransitionError>
where
	Inspector: AssetInspector<AccountId = AccountId, AssetId = AssetId, Asset = Asset<BlockNumber>>,
{
	if account_has_asset_of_type::<_, _, Inspector>(account_id, asset_type) {
		Ok(())
	} else {
		Err(TransitionError::Transition { code: ASSET_HERO_NOT_IN_ACCOUNT })
	}
}

pub(crate) fn ensure_account_has_not_asset_of_type<AccountId, BlockNumber, Inspector>(
	account_id: &AccountId,
	asset_type: VariantType,
) -> Result<(), TransitionError>
where
	Inspector: AssetInspector<AccountId = AccountId, AssetId = AssetId, Asset = Asset<BlockNumber>>,
{
	if account_has_asset_of_type::<_, _, Inspector>(account_id, asset_type) {
		Err(TransitionError::Transition { code: ASSET_HERO_ALREADY_IN_ACCOUNT })
	} else {
		Ok(())
	}
}
