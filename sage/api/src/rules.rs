use crate::{AssetT, Error};
use frame_support::ensure;

pub fn ensure_asset_length<Asset: AssetT>(assets: &[Asset], length: u32) -> Result<(), Error> {
	ensure!(assets.len() as u32 == length, Error::InvalidAssetLength);
	Ok(())
}
