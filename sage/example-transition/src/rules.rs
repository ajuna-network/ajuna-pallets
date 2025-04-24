// Ajuna Node
// Copyright (C) 2022 BlogaTech AG
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use crate::{
	asset::{Asset, AssetId, VariantType},
	error::*,
};

use ajuna_primitives::sage_api::SageApi;
use sage_api::TransitionError;

pub(crate) fn ensure_asset_type_at<BlockNumber>(
	assets: &[(AssetId, Asset<BlockNumber>)],
	variant_type: VariantType,
	asset_index: usize,
) -> Result<(), TransitionError> {
	let (_, asset) = assets
		.get(asset_index)
		.ok_or(TransitionError::Transition { code: ASSET_NOT_FOUND })?;

	if asset.variant.is_variant(variant_type) {
		Ok(())
	} else {
		Err(TransitionError::Transition { code: ASSET_TYPE_NOT_VALID })
	}
}

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
