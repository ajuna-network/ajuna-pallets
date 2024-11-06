use crate::Error;
use frame_support::ensure;

pub fn ensure_asset_length<AssetId>(assets: &[AssetId], length: u32) -> Result<(), Error> {
	ensure!(assets.len() as u32 == length, Error::InvalidAssetLength);
	Ok(())
}
