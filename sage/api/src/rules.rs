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

use crate::TransitionError;

use ajuna_primitives::sage_api::SageApi;
use frame_support::ensure;

pub fn ensure_asset_length<AssetId>(
	assets: &[AssetId],
	length: u32,
) -> Result<(), TransitionError> {
	ensure!(assets.len() as u32 == length, TransitionError::AssetLength);
	Ok(())
}

pub fn ensure_owner_of<AssetId, AccountId, Sage>(
	assets: &[AssetId],
	owner: &AccountId,
) -> Result<(), TransitionError>
where
	Sage: SageApi<AccountId = AccountId, AssetId = AssetId>,
{
	if assets.iter().all(|asset_id| Sage::ensure_ownership(owner, asset_id).is_ok()) {
		Ok(())
	} else {
		Err(TransitionError::AssetOwnership)
	}
}
