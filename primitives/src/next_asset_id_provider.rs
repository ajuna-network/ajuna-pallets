use frame_support::{sp_runtime::traits::CheckedAdd, traits::Randomness};
use std::marker::PhantomData;
use frame_support::sp_runtime::traits::One;

pub trait ProvideNextAssetId {
	type AssetId;

	fn next_asset_id(current_asset_id: &Self::AssetId) -> Option<Self::AssetId>;
}

pub struct IncrementingAssetIdProvider<AssetId>(PhantomData<AssetId>);

impl<AssetId: CheckedAdd + One> ProvideNextAssetId for IncrementingAssetIdProvider<AssetId> {
	type AssetId = AssetId;

	fn next_asset_id(current_asset_id: &Self::AssetId) -> Option<Self::AssetId> {
		current_asset_id.checked_add(&One::one())
	}
}

pub struct RandomAssetIdProvider<AssetId, BlockNumber, RandomnessSource>(
	PhantomData<(AssetId, BlockNumber, RandomnessSource)>,
);

impl<AssetId: AsRef<[u8]>, BlockNumber, R: Randomness<AssetId, BlockNumber>> ProvideNextAssetId
	for RandomAssetIdProvider<AssetId, BlockNumber, R>
{
	type AssetId = AssetId;

	fn next_asset_id(current_asset_id: &Self::AssetId) -> Option<Self::AssetId> {
		Some(R::random(current_asset_id.as_ref()).0)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn incrementing_asset_id_provider_works() {
		assert_eq!(IncrementingAssetIdProvider::next_asset_id(&0u32), Some(1));
		assert_eq!(IncrementingAssetIdProvider::next_asset_id(&1u32), Some(2));

		assert_eq!(IncrementingAssetIdProvider::next_asset_id(&u32::MAX), None);
	}
}
