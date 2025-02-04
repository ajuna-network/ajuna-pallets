// where
// A: AssetFundsManager<
// AccountId = AccountIdOf<T>,
// AssetId = AssetIdOf<T, I>,
// FungiblesAssetId = FungiblesAssetIdOf<T, I>,
// Balance = BalanceOf<T, I>,
// >,
// C: ChainInspector<BlockNumber = BlockNumberFor<T>>,
// S: SeasonManager<
// AssetId = AssetIdOf<T, I>,
// Balance = BalanceOf<T, I>,
// SeasonId = SeasonIdOf<T, I>,
// >,

#[macro_export]
macro_rules! impl_sage_api {
	($impl_target:ident, $runtime:ident, $sage_instance:ident, $account_id:ident, $asset_id:ident, $asset:ident, $fungible_asset_id:ident, $balance:ident, $block_number:ident, $season_id:ident, $transition_config:ident) => {
		impl SageApi for $impl_target {
			type AccountId = $account_id;
			type AssetId = $asset_id;
			type Asset = $asset;
			type FungiblesAssetId = $fungible_asset_id;
			type Balance = $balance;
			type BlockNumber = $block_number;
			type SeasonId = $season_id;
			type TransitionConfig = $transition_config;

			fn get_transition_config() -> Self::TransitionConfig {
				TransitionConfigStore::<$runtime, $sage_instance>::get()
			}

			fn ensure_ownership(
				owner: &Self::AccountId,
				asset_id: &Self::AssetId,
			) -> Result<Self::Asset, DispatchError> {
				<Pallet<$runtime, $sage_instance> as AssetManager>::ensure_ownership(
					owner, asset_id,
				)
			}

			fn get_asset(asset_id: &Self::AssetId) -> Result<Self::Asset, DispatchError> {
				<Pallet<$runtime, $sage_instance> as AssetInspector>::get_asset(asset_id)
			}

			fn iter_assets_from(
				account_id: &Self::AccountId,
			) -> impl Iterator<Item = (Self::AssetId, Self::Asset)> {
				<Pallet<$runtime, $sage_instance> as AssetInspector>::iter_assets_from(account_id)
			}

			fn inspect_asset_funds(
				asset_id: &Self::AssetId,
				fungibles_asset_id: &Self::FungiblesAssetId,
			) -> Self::Balance {
				<Pallet<$runtime, $sage_instance> as AssetFundsManager>::inspect_asset_funds(
					asset_id,
					fungibles_asset_id,
				)
			}

			fn deposit_funds_to_asset(
				asset_id: &Self::AssetId,
				from: &Self::AccountId,
				fungibles_asset_id: Self::FungiblesAssetId,
				amount: Self::Balance,
			) -> Result<(), DispatchError> {
				<Pallet<$runtime, $sage_instance> as AssetFundsManager>::deposit_funds_to_asset(
					asset_id,
					from,
					fungibles_asset_id,
					amount,
				)
			}

			fn transfer_funds_from_asset(
				asset_id: &Self::AssetId,
				to: &Self::AccountId,
				fungibles_asset_id: Self::FungiblesAssetId,
				amount: Self::Balance,
			) -> Result<(), DispatchError> {
				<Pallet<$runtime, $sage_instance> as AssetFundsManager>::transfer_funds_from_asset(
					asset_id,
					to,
					fungibles_asset_id,
					amount,
				)
			}

			fn transfer_all_from_asset(
				asset_id: &Self::AssetId,
				to: &Self::AccountId,
				fungibles_asset_id: Self::FungiblesAssetId,
			) -> Result<(), DispatchError> {
				<Pallet<$runtime, $sage_instance> as AssetFundsManager>::transfer_all_from_asset(
					asset_id,
					to,
					fungibles_asset_id,
				)
			}

			fn get_current_block_number() -> Self::BlockNumber {
				frame_system::Pallet::<$runtime>::block_number()
			}

			fn get_season_id_for(
				asset_id: &Self::AssetId,
			) -> Result<Self::SeasonId, DispatchError> {
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
	};
}
