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
