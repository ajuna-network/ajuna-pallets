#[macro_export]
macro_rules! impl_sage_api {
	(
		$impl_target:ident,
		$runtime:ident,
		$sage_instance:ident,
		$season_manager:ident,
		$randomness:ident,
		$account_id:ident,
		$asset_id:ident,
		$asset:ident,
		$fungible_asset_id:ident,
		$balance:ident,
		$block_number:ident,
		$season_id:ident,
		$transition_config:ident,
		$hash_output:ident,
	) => {
		impl SageApi for $impl_target {
			type AccountId = $account_id;
			type AssetId = $asset_id;
			type Asset = $asset;
			type FungiblesAssetId = $fungible_asset_id;
			type Balance = $balance;
			type BlockNumber = $block_number;
			type SeasonId = $season_id;
			type TransitionConfig = $transition_config;
			type HashOutput = $hash_output;

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
				<$season_manager as SeasonManager>::get_season_id_for(asset_id)
			}

			fn get_current_season_id() -> Result<Self::SeasonId, DispatchError> {
				<$season_manager as SeasonManager>::get_current_season_id()
			}

			fn is_valid_season(season_id: &Self::SeasonId) -> Result<(), DispatchError> {
				<$season_manager as SeasonManager>::is_valid_season(season_id)
			}

			fn get_season_config_for(
				season_id: &Self::SeasonId,
			) -> Result<SeasonConfig<Self::Balance>, DispatchError> {
				<$season_manager as SeasonManager>::get_season_config_for(season_id)
			}

			fn register_asset_in(
				asset_id: &Self::AssetId,
				season_id: &Self::SeasonId,
			) -> Result<(), DispatchError> {
				<$season_manager as SeasonManager>::register_asset_in(asset_id, season_id)
			}

			fn random_hash(subject: &[u8]) -> Self::HashOutput {
				<$randomness as frame_support::traits::Randomness<
					Self::HashOutput,
					Self::BlockNumber,
				>>::random(subject)
				.0
			}
		}
	};
}
