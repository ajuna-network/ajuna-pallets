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
	($impl_target:ident, $system:ident, $sage:ident, $block_number:ident) => {
		impl SageApi for $impl_target {
			type AccountId = <$system as frame_system::Config>::AccountId;
			type AssetId =
				<<$sage as crate::Config>::SageGameTransition as SageGameTransition>::AssetId;
			type Asset =
				<<$sage as crate::Config>::SageGameTransition as SageGameTransition>::Asset;
			type FungiblesAssetId = <$sage as crate::Config>::FungiblesAssetId;
			type Balance = <<$sage as crate::Config>::Fungible as fungible::Inspect<
				<$system as frame_system::Config>::AccountId,
			>>::Balance;
			type BlockNumber = $block_number;
			type SeasonId = <<$sage as Config>::SeasonHandler as SeasonManager>::SeasonId;
			type TransitionConfig =
				<<$sage as Config>::SageGameTransition as SageGameTransition>::TransitionConfig;

			fn get_transition_config() -> Self::TransitionConfig {
				// TransitionConfigStore::<$runtime, $sage_instance>::get()
				todo!()
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

			fn iter_assets_from(account_id: &Self::AccountId) -> Vec<(Self::AssetId, Self::Asset)> {
				vec![]
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
