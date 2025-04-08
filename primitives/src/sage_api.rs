use crate::season_manager::SeasonConfig;
use ajuna_payment_handler::{IdentifyVoucherOrAssetId, NativeId};
use frame_support::{
	pallet_prelude::{DispatchError, MaybeSerializeDeserialize, Member},
	Parameter,
};
use parity_scale_codec::{Codec, MaxEncodedLen};

pub trait SageApi {
	type AccountId: Member + Codec;

	type AssetId: Member + Codec;

	type Asset: Member + Codec;

	type FungiblesAssetId: Clone + NativeId + IdentifyVoucherOrAssetId;

	type Balance;

	type BlockNumber;

	type SeasonId: Member + Parameter + MaxEncodedLen + MaybeSerializeDeserialize;

	type TransitionConfig;
	type HashOutput;
	fn get_transition_config() -> Self::TransitionConfig;

	fn ensure_ownership(
		owner: &Self::AccountId,
		asset_id: &Self::AssetId,
	) -> Result<Self::Asset, DispatchError>;

	fn get_asset(asset_id: &Self::AssetId) -> Result<Self::Asset, DispatchError>;

	fn create_next_asset_id() -> Option<Self::AssetId>;

	fn iter_assets_from(
		account_id: &Self::AccountId,
	) -> impl Iterator<Item = (Self::AssetId, Self::Asset)>;

	fn inspect_asset_funds(
		asset_id: &Self::AssetId,
		fungibles_asset_id: &Self::FungiblesAssetId,
	) -> Self::Balance;

	fn deposit_funds_to_asset(
		asset_id: &Self::AssetId,
		from: &Self::AccountId,
		fungibles_asset_id: Self::FungiblesAssetId,
		amount: Self::Balance,
	) -> Result<(), DispatchError>;

	fn transfer_funds_from_asset(
		asset_id: &Self::AssetId,
		to: &Self::AccountId,
		fungibles_asset_id: Self::FungiblesAssetId,
		amount: Self::Balance,
	) -> Result<(), DispatchError>;

	fn transfer_all_from_asset(
		asset_id: &Self::AssetId,
		to: &Self::AccountId,
		fungibles_asset_id: Self::FungiblesAssetId,
	) -> Result<(), DispatchError>;

	fn get_current_block_number() -> Self::BlockNumber;

	fn get_season_id_for(asset_id: &Self::AssetId) -> Result<Self::SeasonId, DispatchError>;

	fn get_current_season_id() -> Result<Self::SeasonId, DispatchError>;

	fn is_valid_season(season_id: &Self::SeasonId) -> Result<(), DispatchError>;

	fn get_season_config_for(
		season_id: &Self::SeasonId,
	) -> Result<SeasonConfig<Self::Balance>, DispatchError>;

	fn register_asset_in(
		asset_id: &Self::AssetId,
		season_id: &Self::SeasonId,
	) -> Result<(), DispatchError>;

	fn random_hash(subject: &[u8]) -> Self::HashOutput;
}
#[cfg(test)]
pub mod mock {
	use std::{collections::BTreeMap, marker::PhantomData};

	// #[derive(Debug, Clone)]
	// pub struct MockSage<AccountId, AssetId, Asset, FungiblesAssetIt> {
	//
	// 	assets: BTreeMap<AssetId, (AccountId, Asset)>,
	//
	// 	_phantom: PhantomData<(FungiblesAssetIt, AccountId)>,
	// }
	//
	// impl<AccountId, AssetId, Asset, FungiblesAssetIt> MockSage<AccountId, AssetId, Asset,
	// FungiblesAssetIt> {
	//
	// 	pub fn new() -> Self {
	// 		Self {
	// 			assets: BTreeMap::new(),
	// 			_phantom: Default::default(),
	// 		}
	// 	}
	// }

	#[macro_export]
	macro_rules! mock_sage_api {
		(
		$account_id:ident,
		$asset_id:ident,
		$asset:ident,
		$fungible_asset_id:ident,
	) => {
			#[derive(Debug, Clone)]
			pub struct MockSage<$account_id, $asset_id, $asset, $fungible_asset_id> {
				assets: BTreeMap<AssetId, (AccountId, Asset)>,
				_phantom: PhantomData<(FungiblesAssetIt, AccountId)>,
			}

			thread_local! {
				static MOCK_SAGE: RefCell<Option<TestSage>> = RefCell::new(Some(TestSage::new()));
			}

			pub fn with_mock_sage<F: FnOnce(TestSage) -> R, R>(f: F) -> R {
				MOCK_SAGE.with(|s| f(s.borrow().clone().unwrap()))
			}
		};
	}
}

#[cfg(test)]
mod tests {
	use crate::sage_api::mock::MockSage;
	use std::cell::RefCell;

	type TestSage = MockSage<String, String, String, ()>;

	thread_local! {
			static MOCK_SAGE: RefCell<Option<TestSage>> = RefCell::new(Some(TestSage::new()));
	}

	pub fn with_mock_sage<F: FnOnce(TestSage) -> R, R>(f: F) -> R {
		MOCK_SAGE.with(|s| f(s.borrow().clone().unwrap()))
	}

	// #[test]
	// fn test_mock_sage() {
	// 	with_mock_sage(|s| {
	// 		s.
	//
	// 	})
	// }
}
