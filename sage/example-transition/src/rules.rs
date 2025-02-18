use crate::{
	asset::{Asset, AssetId, VariantType},
	error::*,
};

use ajuna_primitives::sage_api::SageApi;
use sage_api::TransitionError;

#[allow(dead_code)]
pub(crate) fn ensure_all_asset_type<BlockNumber>(
	assets: &[(AssetId, Asset<BlockNumber>)],
	variant_type: VariantType,
) -> Result<(), TransitionError> {
	assets
		.iter()
		.all(|(_, asset)| asset.variant.is_variant(variant_type))
		.then_some(())
		.ok_or(TransitionError::Transition { code: ASSETS_NOT_ALL_SAME_TYPE })
}

pub(crate) fn ensure_asset_type_at<BlockNumber>(
	assets: &[(AssetId, Asset<BlockNumber>)],
	variant_type: VariantType,
	asset_index: usize,
) -> Result<(), TransitionError> {
	if let Some((_, asset)) = assets.get(asset_index) {
		if asset.variant.is_variant(variant_type) {
			Ok(())
		} else {
			Err(TransitionError::Transition { code: ASSET_TYPE_NOT_VALID })
		}
	} else {
		Err(TransitionError::Transition { code: ASSET_NOT_FOUND })
	}
}

#[inline]
fn account_has_any_asset<AccountId, BlockNumber, Sage, F>(
	account_id: &AccountId,
	filter_fn: F,
) -> bool
where
	F: Fn((AssetId, Asset<BlockNumber>)) -> bool,
	Sage: SageApi<AccountId = AccountId, AssetId = AssetId, Asset = Asset<BlockNumber>>,
{
	Sage::iter_assets_from(account_id).any(filter_fn)
}

#[inline]
fn account_has_no_asset<AccountId, BlockNumber, Sage, F>(
	account_id: &AccountId,
	filter_fn: F,
) -> bool
where
	F: Fn((AssetId, Asset<BlockNumber>)) -> bool,
	Sage: SageApi<AccountId = AccountId, AssetId = AssetId, Asset = Asset<BlockNumber>>,
{
	Sage::iter_assets_from(account_id).all(|asset| !filter_fn(asset))
}

#[allow(dead_code)]
pub(crate) fn ensure_account_has_asset_of_type<AccountId, BlockNumber, Sage>(
	account_id: &AccountId,
	variant_type: VariantType,
) -> Result<(), TransitionError>
where
	Sage: SageApi<AccountId = AccountId, AssetId = AssetId, Asset = Asset<BlockNumber>>,
{
	let filter = |(_, asset): (AssetId, Asset<BlockNumber>)| asset.variant.is_variant(variant_type);

	if account_has_any_asset::<_, _, Sage, _>(account_id, filter) {
		Ok(())
	} else {
		Err(TransitionError::Transition { code: ASSET_TYPE_NOT_FOUND_IN_ACCOUNT })
	}
}

pub(crate) fn ensure_account_has_no_asset_of_type<AccountId, BlockNumber, Sage>(
	account_id: &AccountId,
	variant_type: VariantType,
) -> Result<(), TransitionError>
where
	Sage: SageApi<AccountId = AccountId, AssetId = AssetId, Asset = Asset<BlockNumber>>,
{
	let filter = |(_, asset): (AssetId, Asset<BlockNumber>)| asset.variant.is_variant(variant_type);

	if account_has_no_asset::<_, _, Sage, _>(account_id, filter) {
		Ok(())
	} else {
		Err(TransitionError::Transition { code: ASSET_TYPE_ALREADY_IN_ACCOUNT })
	}
}
