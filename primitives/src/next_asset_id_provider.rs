use frame_support::{
	sp_runtime::traits::{CheckedAdd, One},
	traits::Randomness,
};
use parity_scale_codec::Encode;
use sp_std::marker::PhantomData;

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

impl<AssetId: Encode, BlockNumber, R: Randomness<AssetId, BlockNumber>> ProvideNextAssetId
	for RandomAssetIdProvider<AssetId, BlockNumber, R>
{
	type AssetId = AssetId;

	fn next_asset_id(current_asset_id: &Self::AssetId) -> Option<Self::AssetId> {
		// Important: The output of `R::random` should not be used to derive another random output.
		// This will weaken the safety. See the documentation of the randomness collective pallet
		// for more information.
		Some(R::random(current_asset_id.encode().as_slice()).0)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	struct MockRandomness;

	impl Randomness<u32, u32> for MockRandomness {
		fn random(subject: &[u8]) -> (u32, u32) {
			(2 * subject[0] as u32, 0)
		}
	}

	type MockRandomAssetIdProvider = RandomAssetIdProvider<u32, u32, MockRandomness>;

	#[test]
	fn incrementing_asset_id_provider_works() {
		assert_eq!(IncrementingAssetIdProvider::next_asset_id(&0u32), Some(1));
		assert_eq!(IncrementingAssetIdProvider::next_asset_id(&1u32), Some(2));

		assert_eq!(IncrementingAssetIdProvider::next_asset_id(&u32::MAX), None);
	}

	#[test]
	fn random_asset_id_provider_works() {
		// This test shows that the input is indeed used as a subject for creating the output.
		// But the safety and reasonable uniqueness guarantees depend on the actual
		// randomness source used.
		assert_eq!(MockRandomAssetIdProvider::next_asset_id(&10), Some(20));
	}
}
