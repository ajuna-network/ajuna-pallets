use crate::{
	AccountIdOf, AssetIdOf, AssetOf, BalanceOf, Config, FungiblesAssetIdOf, Pallet, SeasonIdOf,
	TransitionConfigOf, TransitionConfigStore,
};
use ajuna_primitives::{
	asset_manager::AssetFundsManager,
	chain_inspector::ChainInspector,
	sage_api::SageApi,
	season_manager::{SeasonConfig, SeasonManager},
};
use frame_system::pallet_prelude::BlockNumberFor;
use sp_runtime::DispatchError;
use std::marker::PhantomData;

pub struct SageEngine<Runtime, Instance, AssetFundsManager, ChainInspector, SeasonManager>(
	PhantomData<(Runtime, Instance, AssetFundsManager, ChainInspector, SeasonManager)>,
);

impl<T: Config<I>, I: 'static, A, C, S> SageApi for SageEngine<T, I, A, C, S>
where
	A: AssetFundsManager<
		AccountId = AccountIdOf<T>,
		AssetId = AssetIdOf<T, I>,
		FungiblesAssetId = FungiblesAssetIdOf<T, I>,
		Balance = BalanceOf<T, I>,
	>,
	C: ChainInspector<BlockNumber = BlockNumberFor<T>>,
	S: SeasonManager<
		AssetId = AssetIdOf<T, I>,
		Balance = BalanceOf<T, I>,
		SeasonId = SeasonIdOf<T, I>,
	>,
{
	type AccountId = AccountIdOf<T>;
	type AssetId = AssetIdOf<T, I>;
	type Asset = AssetOf<T, I>;
	type FungiblesAssetId = FungiblesAssetIdOf<T, I>;
	type Balance = BalanceOf<T, I>;
	type BlockNumber = BlockNumberFor<T>;
	type SeasonId = SeasonIdOf<T, I>;
	type TransitionConfig = TransitionConfigOf<T, I>;

	fn get_transition_config() -> Self::TransitionConfig {
		TransitionConfigStore::<T, I>::get()
	}

	fn ensure_ownership(
		owner: &Self::AccountId,
		asset_id: &Self::AssetId,
	) -> Result<Self::Asset, DispatchError> {
		todo!()
	}

	fn get_asset(asset_id: &Self::AssetId) -> Result<Self::Asset, DispatchError> {
		todo!()
	}

	fn iter_assets_from(
		account_id: &Self::AccountId,
	) -> impl Iterator<Item = (Self::AssetId, Self::Asset)> {
		vec![].into_iter()
	}

	fn inspect_asset_funds(
		asset_id: &Self::AssetId,
		fungibles_asset_id: &Self::FungiblesAssetId,
	) -> Self::Balance {
		todo!()
	}

	fn deposit_funds_to_asset(
		asset_id: &Self::AssetId,
		from: &Self::AccountId,
		fungibles_asset_id: Self::FungiblesAssetId,
		amount: Self::Balance,
	) -> Result<(), DispatchError> {
		todo!()
	}

	fn transfer_funds_from_asset(
		asset_id: &Self::AssetId,
		to: &Self::AccountId,
		fungibles_asset_id: Self::FungiblesAssetId,
		amount: Self::Balance,
	) -> Result<(), DispatchError> {
		todo!()
	}

	fn transfer_all_from_asset(
		asset_id: &Self::AssetId,
		to: &Self::AccountId,
		fungibles_asset_id: Self::FungiblesAssetId,
	) -> Result<(), DispatchError> {
		todo!()
	}

	fn get_current_block_number() -> Self::BlockNumber {
		todo!()
	}

	fn get_season_id_for(asset_id: &Self::AssetId) -> Result<Self::SeasonId, DispatchError> {
		todo!()
	}

	fn get_current_season_id() -> Result<Self::SeasonId, DispatchError> {
		todo!()
	}

	fn is_valid_season(season_id: &Self::SeasonId) -> Result<(), DispatchError> {
		todo!()
	}

	fn get_season_config_for(
		season_id: &Self::SeasonId,
	) -> Result<SeasonConfig<Self::Balance>, DispatchError> {
		todo!()
	}

	fn register_asset_in(
		asset_id: &Self::AssetId,
		season_id: &Self::SeasonId,
	) -> Result<(), DispatchError> {
		todo!()
	}
}
