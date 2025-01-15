use crate::TransitionError;

use ajuna_primitives::asset_manager::AssetManager;

use frame_support::ensure;

pub fn ensure_asset_length<AssetId>(
	assets: &[AssetId],
	length: u32,
) -> Result<(), TransitionError> {
	ensure!(assets.len() as u32 == length, TransitionError::AssetLength);
	Ok(())
}

pub fn ensure_owner_of<AssetId, AccountId, Manager>(
	assets: &[AssetId],
	owner: &AccountId,
) -> Result<(), TransitionError>
where
	Manager: AssetManager<AccountId = AccountId, AssetId = AssetId>,
{
	if assets.iter().all(|asset_id| Manager::ensure_ownership(owner, asset_id).is_ok()) {
		Ok(())
	} else {
		Err(TransitionError::AssetOwnership)
	}
}
