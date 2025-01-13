use crate::RuleError;

use ajuna_primitives::asset_manager::AssetManager;

use frame_support::ensure;

pub fn ensure_asset_length<AssetId>(assets: &[AssetId], length: u32) -> Result<(), RuleError> {
	ensure!(assets.len() as u32 == length, RuleError::AssetLength);
	Ok(())
}

pub fn ensure_owner_of<AssetId, AccountId, Manager>(
	assets: &[AssetId],
	owner: &AccountId,
) -> Result<(), RuleError>
where
	Manager: AssetManager<AccountId = AccountId, AssetId = AssetId>,
{
	assets
		.iter()
		.all(|asset_id| Manager::ensure_ownership(owner, asset_id).is_ok())
		.then_some(())
		.ok_or(RuleError::AssetOwnership)
}
