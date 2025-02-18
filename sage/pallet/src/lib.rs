// Ajuna Node
// Copyright (C) 2022 BlogaTech AG

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

//! Pallet-SAGE
//!
//! This pallet is the core entry point of the sage architecture that wires things together.

#![cfg_attr(not(feature = "std"), no_std)]

pub mod weights;

pub mod config;
mod pallet_impls;
mod trait_impls;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;
#[cfg(test)]
pub mod mock;
#[cfg(test)]
mod tests;

use ajuna_primitives::{
	account_manager::{AccountManager, WhitelistKey},
	asset_manager::{AssetFundsManager, AssetManager, Lock, LockIdentifier},
	payment_handler::{FeeHandler, TransferFungible},
	season_manager::{SeasonConfig, SeasonManager},
	trade_manager::{TradeManager, TransferManager},
};
use sage_api::{
	traits::{GetId, TransitionOutput},
	SageGameTransition, TransitionError,
};

use frame_support::{pallet_prelude::*, traits::fungible, PalletId};
use frame_system::pallet_prelude::*;
use sp_runtime::{
	traits::{AccountIdConversion, UniqueSaturatedInto},
	Saturating,
};
use sp_std::prelude::*;

use weights::WeightInfo;

pub use config::*;
pub use pallet::*;

pub const SAGE_LOCK_ID: &[u8; 8] = b"sagelock";

pub const MAX_ASSETS_IN_TRANSITION: usize = 10;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use ajuna_primitives::payment_handler::{IdentifyVoucherOrAssetId, NativeId};
	use frame_support::traits::tokens::{AssetId, Preservation};

	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

	pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub type BalanceOf<T, I> =
		<<T as Config<I>>::Fungible as fungible::Inspect<AccountIdOf<T>>>::Balance;

	pub type AssetIdOf<T, I> =
		<<T as Config<I>>::SageGameTransition as SageGameTransition>::AssetId;
	pub type AssetOf<T, I> = <<T as Config<I>>::SageGameTransition as SageGameTransition>::Asset;
	pub type TransitionIdOf<T, I> =
		<<T as Config<I>>::SageGameTransition as SageGameTransition>::TransitionId;
	pub type ExtraOf<T, I> = <<T as Config<I>>::SageGameTransition as SageGameTransition>::Extra;
	pub type TransitionConfigOf<T, I> =
		<<T as Config<I>>::SageGameTransition as SageGameTransition>::TransitionConfig;
	pub(crate) type TransitionOutputOf<T, I> = TransitionOutput<AssetIdOf<T, I>, AssetOf<T, I>>;
	pub type PlayerStatsOf<T> = PlayerStats<BlockNumberFor<T>>;

	pub type SeasonConfigOf<T, I> = SeasonConfig<BalanceOf<T, I>>;
	pub type SeasonIdOf<T, I> = <<T as Config<I>>::SeasonHandler as SeasonManager>::SeasonId;

	pub type TradeFilterOf<T, I> = <<T as Config<I>>::FilterHandler as TradeManager>::TradeFilter;
	pub type TransferFilterOf<T, I> =
		<<T as Config<I>>::FilterHandler as TransferManager>::TransferFilter;
	pub type AssetFilterOf<T, I> = AssetFilter<TradeFilterOf<T, I>, TransferFilterOf<T, I>>;
	pub type AffiliateMethodsOf<T, I> = AffiliateMethods<TransitionIdOf<T, I>>;

	pub type FungiblesAssetIdOf<T, I> = <T as Config<I>>::FungiblesAssetId;

	#[pallet::genesis_config]
	#[derive(frame_support::DefaultNoBound)]
	pub struct GenesisConfig<T: Config<I>, I: 'static = ()> {
		/// Genesis organizer account
		pub organizer: Option<AccountIdOf<T>>,
		/// Genesis initial season
		pub season: Option<SeasonIdOf<T, I>>,
	}

	#[pallet::genesis_build]
	impl<T: Config<I>, I: 'static> BuildGenesisConfig for GenesisConfig<T, I> {
		fn build(&self) {
			if let Some(ref organizer) = self.organizer {
				Organizer::<T, I>::set(Some(organizer.clone()));

				GeneralConfigStore::<T, I>::set(GeneralConfig {
					transfer: TransferConfig { open: true },
					trade: TradeConfig { open: true },
				});

				if let Some(ref season_id) = self.season {
					PlayerSeasonConfigs::<T, I>::insert(
						organizer,
						season_id,
						PlayerConfig {
							inventory_tier: InventoryTier::One,
							locks: Locks::all_unlocked(),
						},
					);
				}
			}
		}
	}

	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config {
		/// This pallet's id.
		///
		/// It will be used as a lock identifier when locking assets.
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// The `SageGameTransition` that this pallet hosts, and whose state transition
		/// are executed as part of the `state_transition` extrinsic.
		type SageGameTransition: SageGameTransition<
			AccountId = AccountIdOf<Self>,
			PaymentFungible = Self::FungiblesAssetId,
		>;

		/// Retrieves information about past and ongoing seasons.
		type SeasonHandler: SeasonManager<
			AssetId = AssetIdOf<Self, I>,
			Balance = BalanceOf<Self, I>,
		>;

		/// Handles the extra fees that incur during executing the state transition, or other
		/// things like paying for an asset inventory upgrade.
		type FeeHandler: FeeHandler<
			AccountId = AccountIdOf<Self>,
			PaymentKind = FungiblesAssetIdOf<Self, I>,
			Balance = BalanceOf<Self, I>,
			AffiliateFeeIdentifier = AffiliateMethodsOf<Self, I>,
			TournamentFeeIdentifier = SeasonIdOf<Self, I>,
		>;

		type TransferFunds: TransferFungible<
			AccountId = AccountIdOf<Self>,
			AssetId = FungiblesAssetIdOf<Self, I>,
			Balance = BalanceOf<Self, I>,
		>;

		type FungiblesAssetId: AssetId + IdentifyVoucherOrAssetId + NativeId;

		/// Applies the filter that has been set in the `SeasonTraderFilters` or the
		/// `SeasonTransferFilters` storage.
		type FilterHandler: TradeManager<Asset = AssetOf<Self, I>>
			+ TransferManager<Asset = AssetOf<Self, I>>;

		/// Fungible implementation used by this pallet. This will most likely be the
		/// balances-pallet.
		type Fungible: fungible::Inspect<AccountIdOf<Self>> + fungible::Mutate<AccountIdOf<Self>>;

		/// The overarching event type.
		type RuntimeEvent: From<Event<Self, I>>
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The weight calculations
		type WeightInfo: WeightInfo;

		#[cfg(feature = "runtime-benchmarks")]
		type BenchmarkHelper: sage_api::benchmarks::SageBenchmarkHelper<
			AssetIdOf<Self, I>,
			AssetOf<Self, I>,
			TransitionIdOf<Self, I>,
			TradeFilterOf<Self, I>,
			TransferFilterOf<Self, I>,
			FungiblesAssetIdOf<Self, I>,
		>;
	}

	/// Organizer of the game. Essentially the administrator with certain privileges.
	#[pallet::storage]
	pub type Organizer<T: Config<I>, I: 'static = ()> =
		StorageValue<_, AccountIdOf<T>, OptionQuery>;

	/// Tracks global configuration values that can be changed by the organizer only.
	#[pallet::storage]
	pub type GeneralConfigStore<T: Config<I>, I: 'static = ()> =
		StorageValue<_, GeneralConfig, ValueQuery>;

	/// Configuration values specific to the transition being used.
	#[pallet::storage]
	pub type TransitionConfigStore<T: Config<I>, I: 'static = ()> =
		StorageValue<_, TransitionConfigOf<T, I>, ValueQuery>;

	/// Some features need to be unlocked fulfilling certain criteria.
	///
	/// This storage keeps track of the `UnlockRule` that needs to be satisfied to unlock the
	/// feature. If there is no unlock rule, the feature can't be unlocked in that season.
	#[pallet::storage]
	pub type SeasonUnlocks<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
		_,
		Identity,
		SeasonIdOf<T, I>,
		Identity,
		LockableFeature,
		UnlockRule,
		OptionQuery,
	>;

	/// Tracks player configs per season. This can be mutated by unlocking certain privileges, e.g.
	/// upgrading the storage inventory size.
	#[pallet::storage]
	pub type PlayerSeasonConfigs<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
		_,
		Identity,
		AccountIdOf<T>,
		Identity,
		SeasonIdOf<T, I>,
		PlayerConfig,
		ValueQuery,
	>;

	/// Tracks player stats per season.
	#[pallet::storage]
	pub type PlayerSeasonStats<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
		_,
		Identity,
		AccountIdOf<T>,
		Identity,
		SeasonIdOf<T, I>,
		PlayerStatsOf<T>,
		ValueQuery,
	>;

	/// Maps the `AssetId` to its owner and the asset.
	#[pallet::storage]
	pub type Assets<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, AssetIdOf<T, I>, (AccountIdOf<T>, AssetOf<T, I>)>;

	/// Keeps track of the assets owned by an account and in which season the asset was created.
	///
	/// We mostly do ownership checks on this in the runtime. Whereas the frontends want to display
	/// a list. This has to be queried with a `state.getKeysPaged` followed by a `state.getStorage`
	/// call. Maybe it makes sense to implement a runtime api call for this to reduce networking
	/// bandwidth.
	#[pallet::storage]
	pub type AssetOwners<T: Config<I>, I: 'static = ()> = StorageNMap<
		_,
		(
			NMapKey<Identity, AccountIdOf<T>>,
			NMapKey<Identity, SeasonIdOf<T, I>>,
			NMapKey<Identity, AssetIdOf<T, I>>,
		),
		(),
		ValueQuery,
	>;

	/// Keeps track of how many assets an account owns.
	#[pallet::storage]
	pub type AssetsOwnedCount<T: Config<I>, I: 'static = ()> =
		StorageDoubleMap<_, Identity, AccountIdOf<T>, Identity, SeasonIdOf<T, I>, u8, ValueQuery>;

	/// A filter that assets need to pass in order to be traded.
	///
	/// The filter can be changed by the organizer.
	#[pallet::storage]
	pub type SeasonTradeFilters<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, SeasonIdOf<T, I>, TradeFilterOf<T, I>, OptionQuery>;

	/// A filter that assets need to pass in order to be transfer.
	///
	/// The filter can be changed by the organizer.
	#[pallet::storage]
	pub type SeasonTransferFilters<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, SeasonIdOf<T, I>, TransferFilterOf<T, I>, OptionQuery>;

	/// Tracks assets that have been put on the market with a certain price.
	#[pallet::storage]
	pub type AssetTradePrices<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
		_,
		Identity,
		SeasonIdOf<T, I>,
		Identity,
		AssetIdOf<T, I>,
		BalanceOf<T, I>,
		OptionQuery,
	>;

	/// Tracks assets that have been locked either through the `lock_asset` extrinsic, or by
	/// other pallets via this pallet's `AssetManager` implementation.
	///
	/// A locked asset can't be transferred, traded, consumed or mutated.
	#[pallet::storage]
	pub type LockedAssets<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, AssetIdOf<T, I>, Lock<AccountIdOf<T>>>;

	/// Tracks how many funds assets have, which will be returned to the owner, once the
	/// asset is consumed
	#[pallet::storage]
	pub type AssetFunds<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
		_,
		Blake2_128Concat,
		AssetIdOf<T, I>,
		Blake2_128Concat,
		FungiblesAssetIdOf<T, I>,
		BalanceOf<T, I>,
		OptionQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		/// An organizer has been set.
		OrganizerSet { organizer: AccountIdOf<T> },
		/// General configuration updated.
		UpdatedGeneralConfig { new_config: GeneralConfig },
		/// Transition configuration updated.
		UpdatedTransitionConfig { new_config: TransitionConfigOf<T, I> },
		/// Unlock configuration updated for feature.
		UpdatedUnlockRule {
			season_id: SeasonIdOf<T, I>,
			feature: LockableFeature,
			updated_rule: UnlockRule,
		},
		/// Storage tier has been upgraded.
		InventoryTierUpgraded {
			account: AccountIdOf<T>,
			season_id: SeasonIdOf<T, I>,
			new_tier: InventoryTier,
		},
		/// Trade filter has been updated
		UpdatedTradeFilter { season_id: SeasonIdOf<T, I>, filter: TradeFilterOf<T, I> },
		/// Transfer filter has been updated
		UpdatedTransferFilter { season_id: SeasonIdOf<T, I>, filter: TransferFilterOf<T, I> },
		/// Asset transferred.
		AssetTransferred { from: AccountIdOf<T>, to: AccountIdOf<T>, asset_id: AssetIdOf<T, I> },
		/// Asset has price set for trade.
		AssetPriceSet { asset_id: AssetIdOf<T, I>, price: BalanceOf<T, I> },
		/// Asset has price removed for trade.
		AssetPriceUnset { asset_id: AssetIdOf<T, I> },
		/// Asset has been traded.
		AssetTraded {
			asset_id: AssetIdOf<T, I>,
			from: AccountIdOf<T>,
			to: AccountIdOf<T>,
			price: BalanceOf<T, I>,
		},
		/// Asset locked.
		AssetLocked { asset_id: AssetIdOf<T, I>, lock: Lock<AccountIdOf<T>> },
		/// Asset unlocked.
		AssetUnlocked { asset_id: AssetIdOf<T, I>, lock: Lock<AccountIdOf<T>> },
		/// A feature has been unlocked
		FeatureUnlocked {
			feature: LockableFeature,
			season_id: SeasonIdOf<T, I>,
			account: AccountIdOf<T>,
		},
		/// A transition has been executed.
		TransitionExecuted {
			/// Account who initiated execution.
			account: AccountIdOf<T>,
			/// Transition ID that was executed.
			id: TransitionIdOf<T, I>,
		},
	}

	/// Error for the pallet-sage.
	#[pallet::error]
	pub enum Error<T, I = ()> {
		/// There is no account set as the organizer
		OrganizerNotSet,
		/// The asset doesn't exist.
		UnknownAsset,
		/// Transfer is not available at the moment.
		TransferClosed,
		/// Trading is not available at the moment.
		TradeClosed,
		/// Max asset ownership reached.
		MaxOwnershipReached,
		/// Max asset storage tier reached.
		MaxStorageTierReached,
		/// Asset belongs to someone else.
		AssetNotOwned,
		/// Attempt to buy already owned asset.
		AlreadyOwned,
		/// This asset cannot be used in transfer.
		AssetCannotBeTransfered,
		/// This asset cannot be used in trade.
		AssetCannotBeTraded,
		/// An asset selected for buying is not actually in sale.
		AssetNotInTrade,
		/// An asset listed for trade cannot be transferred to another account.
		CannotTransferAssetInTrade,
		/// An asset in trade cannot be locked.
		CannotLockAssetInTrade,
		/// The asset is currently locked and cannot be used.
		AssetLocked,
		/// The asset is locked by another application.
		AssetLockedByOtherApplication,
		/// The asset is not currently locked and cannot be unlocked.
		AssetNotLocked,
		/// The asset does not own enough funds for the operation..
		AssetsFundsTooLow,
		/// Tried transferring to his or her own account.
		CannotTransferToSelf,
		/// The feature is locked for the current player
		FeatureLocked,
		/// The feature trying to be unlocked is not available for the selected season
		FeatureUnavailableInSeason,
		/// The feature trying to be unlocked cannot be unlocked with payment
		FeatureLockedThroughPayment,
		/// The feature trying to be unlocked has missing requirements to be fulfilled by
		/// the account trying to unlock it
		UnlockCriteriaNotFulfilled,
		/// The amount of input assets in the transition is greater than 'MAX_ASSETS_IN_TRANSITION'
		TooManyAssetsInTransition,
		/// The rule for a given transition was not satisfied.
		TransitionRuleNotSatisfied,
		/// A transfer error occurred inside the transition.
		TransferError,
		/// An error occurred during the fee payment of the ransition.
		FeeError,
		/// Invalid number of assets for this transition.
		AssetLength,
		/// Asset Ownership error.
		AssetOwnership,
		/// Voucher is not allowed for that transition.
		VoucherNotAllowed,
		/// An error occurred during the state transition.
		Transition { code: u8 },
	}

	impl<T, I> From<TransitionError> for Error<T, I> {
		fn from(e: TransitionError) -> Self {
			match e {
				TransitionError::TransferError => Error::<T, I>::TransferError,
				TransitionError::FeeError => Error::<T, I>::FeeError,
				TransitionError::AssetLength => Error::<T, I>::AssetLength,
				TransitionError::AssetOwnership => Error::<T, I>::AssetOwnership,
				TransitionError::VoucherNotAllowed => Error::<T, I>::VoucherNotAllowed,
				TransitionError::Transition { code } => Error::<T, I>::Transition { code },
			}
		}
	}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		/// Set game organizer.
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::set_organizer())]
		pub fn set_organizer(origin: OriginFor<T>, organizer: AccountIdOf<T>) -> DispatchResult {
			ensure_root(origin)?;
			Organizer::<T, I>::put(&organizer);
			Self::deposit_event(Event::OrganizerSet { organizer });
			Ok(())
		}

		/// Update general configuration.
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::update_general_config())]
		pub fn update_general_config(
			origin: OriginFor<T>,
			new_config: GeneralConfig,
		) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			Self::ensure_organizer(&signer)?;

			GeneralConfigStore::<T, I>::put(&new_config);
			Self::deposit_event(Event::UpdatedGeneralConfig { new_config });
			Ok(())
		}

		/// Update general configuration.
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::update_general_config())]
		pub fn update_transition_config(
			origin: OriginFor<T>,
			new_config: TransitionConfigOf<T, I>,
		) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			Self::ensure_organizer(&signer)?;

			TransitionConfigStore::<T, I>::put(&new_config);
			Self::deposit_event(Event::UpdatedTransitionConfig { new_config });
			Ok(())
		}

		/// Updates an unlock rule for the given season.
		///
		/// It doesn't affect already unlocked features.
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::update_unlock_rule())]
		pub fn update_unlock_rule(
			origin: OriginFor<T>,
			season_id: SeasonIdOf<T, I>,
			feature: LockableFeature,
			unlock_rule: UnlockRule,
		) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			Self::ensure_organizer(&signer)?;
			SeasonUnlocks::<T, I>::mutate(&season_id, feature, |config| {
				*config = Some(unlock_rule);
			});
			Self::deposit_event(Event::UpdatedUnlockRule {
				season_id,
				feature,
				updated_rule: unlock_rule,
			});
			Ok(())
		}

		/// Upgrade the asset inventory space.
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::upgrade_asset_inventory())]
		pub fn upgrade_asset_inventory(
			origin: OriginFor<T>,
			beneficiary: Option<AccountIdOf<T>>,
			in_season: Option<SeasonIdOf<T, I>>,
			payment: Option<FungiblesAssetIdOf<T, I>>,
		) -> DispatchResult {
			let caller = ensure_signed(origin)?;

			let season_id = if let Some(season_id) = in_season {
				season_id
			} else {
				T::SeasonHandler::get_current_season_id()?
			};
			let fee = T::SeasonHandler::get_season_config_for(&season_id)?.fee;

			let base_fee = fee.upgrade_asset_inventory;
			T::FeeHandler::withdraw_and_pay_fees(
				&caller,
				payment.unwrap_or_else(FungiblesAssetIdOf::<T, I>::get_native_id),
				base_fee,
				&season_id,
				&AffiliateMethods::UpgradeAssetInventory,
				&Self::treasury_account_id(),
			)?;

			let account_to_upgrade = beneficiary.unwrap_or(caller);

			let inventory_tier =
				PlayerSeasonConfigs::<T, I>::get(&account_to_upgrade, &season_id).inventory_tier;
			ensure!(inventory_tier != InventoryTier::Max, Error::<T, I>::MaxStorageTierReached);

			let upgraded_tier =
				PlayerSeasonConfigs::<T, I>::mutate(&account_to_upgrade, &season_id, |account| {
					let upgraded_tier = inventory_tier.upgrade();
					account.inventory_tier = upgraded_tier;
					upgraded_tier
				});
			Self::deposit_event(Event::InventoryTierUpgraded {
				account: account_to_upgrade,
				season_id,
				new_tier: upgraded_tier,
			});
			Ok(())
		}

		/// Updates the filter that assets need to pass for certain actions.
		#[pallet::call_index(5)]
		#[pallet::weight(
			T::WeightInfo::update_asset_trade_filter()
				.max(T::WeightInfo::update_asset_transfer_filter())
        )]
		pub fn update_asset_filter(
			origin: OriginFor<T>,
			season_id: SeasonIdOf<T, I>,
			filter: AssetFilterOf<T, I>,
		) -> DispatchResult {
			let signer = ensure_signed(origin)?;
			Self::ensure_organizer(&signer)?;

			match filter {
				AssetFilter::Trade(filter) => {
					SeasonTradeFilters::<T, I>::insert(&season_id, &filter);
					Self::deposit_event(Event::UpdatedTradeFilter { season_id, filter });
				},
				AssetFilter::Transfer(filter) => {
					SeasonTransferFilters::<T, I>::insert(&season_id, &filter);
					Self::deposit_event(Event::UpdatedTransferFilter { season_id, filter });
				},
			}

			Ok(())
		}

		/// Transfers the asset with `asset_id` from the `origin` to `to`.
		///
		/// It will fail if the asset transfer is disabled, the asset doesn't pass the filter
		/// or if the asset is on the market.
		#[pallet::call_index(6)]
		#[pallet::weight(T::WeightInfo::transfer_asset())]
		pub fn transfer_asset(
			origin: OriginFor<T>,
			to: AccountIdOf<T>,
			asset_id: AssetIdOf<T, I>,
			payment: Option<FungiblesAssetIdOf<T, I>>,
		) -> DispatchResult {
			let from = ensure_signed(origin)?;

			let GeneralConfig { transfer, .. } = GeneralConfigStore::<T, I>::get();
			ensure!(
				transfer.open || Self::ensure_organizer(&from).is_ok(),
				Error::<T, I>::TransferClosed
			);

			ensure!(from != to, Error::<T, I>::CannotTransferToSelf);
			ensure!(
				Self::ensure_for_trade(&asset_id).is_err(),
				Error::<T, I>::CannotTransferAssetInTrade
			);
			Self::ensure_unlocked(&asset_id)?;

			let asset = Self::ensure_ownership(&from, &asset_id)?;
			let asset_season_id = T::SeasonHandler::get_season_id_for(&asset_id)?;
			ensure!(
				PlayerSeasonConfigs::<T, I>::get(&from, &asset_season_id).locks.asset_transfer,
				Error::<T, I>::FeatureLocked
			);

			if let Some(transfer_filter) = SeasonTransferFilters::<T, I>::get(&asset_season_id) {
				ensure!(
					T::FilterHandler::can_be_transferred_using(&asset, &transfer_filter),
					Error::<T, I>::AssetCannotBeTransfered
				);
			}

			let fee = T::SeasonHandler::get_season_config_for(&asset_season_id)?.fee;
			T::FeeHandler::withdraw_and_deposit_into(
				&from,
				payment.unwrap_or_else(FungiblesAssetIdOf::<T, I>::get_native_id),
				&Self::treasury_account_id(),
				fee.transfer_asset,
			)?;

			Self::do_transfer_asset(&from, &to, &asset_season_id, &asset_id)?;
			Self::deposit_event(Event::AssetTransferred { from, to, asset_id });
			Ok(())
		}

		/// Set the price of a given asset, putting it on sale for others to buy.
		#[pallet::call_index(7)]
		#[pallet::weight(T::WeightInfo::set_asset_price())]
		pub fn set_asset_price(
			origin: OriginFor<T>,
			asset_id: AssetIdOf<T, I>,
			price: BalanceOf<T, I>,
		) -> DispatchResult {
			let seller = ensure_signed(origin)?;
			ensure!(GeneralConfigStore::<T, I>::get().trade.open, Error::<T, I>::TradeClosed);

			let (owner, asset) = Self::asset_with_owner(&asset_id)?;
			ensure!(owner == seller, Error::<T, I>::AssetNotOwned);

			let asset_season_id = T::SeasonHandler::get_season_id_for(&asset_id)?;
			let config = PlayerSeasonConfigs::<T, I>::get(&seller, &asset_season_id);
			ensure!(config.locks.asset_trade, Error::<T, I>::FeatureLocked);

			Self::ensure_unlocked(&asset_id)?;

			if let Some(trade_filter) = SeasonTradeFilters::<T, I>::get(&asset_season_id) {
				ensure!(
					T::FilterHandler::can_be_traded_using(&asset, &trade_filter),
					Error::<T, I>::AssetCannotBeTraded
				);
			}

			AssetTradePrices::<T, I>::insert(&asset_season_id, &asset_id, price);
			Self::deposit_event(Event::AssetPriceSet { asset_id, price });
			Ok(())
		}

		/// Remove the price of an asset, and thereby remove it from the market.
		#[pallet::call_index(8)]
		#[pallet::weight(T::WeightInfo::remove_asset_price())]
		pub fn remove_asset_price(
			origin: OriginFor<T>,
			asset_id: AssetIdOf<T, I>,
		) -> DispatchResult {
			let seller = ensure_signed(origin)?;
			ensure!(GeneralConfigStore::<T, I>::get().trade.open, Error::<T, I>::TradeClosed);
			Self::ensure_for_trade(&asset_id)?;
			Self::ensure_ownership(&seller, &asset_id)?;
			let season_id = T::SeasonHandler::get_season_id_for(&asset_id)?;
			AssetTradePrices::<T, I>::remove(&season_id, &asset_id);
			Self::deposit_event(Event::AssetPriceUnset { asset_id });
			Ok(())
		}

		/// Attempt to buy the selected asset.
		#[pallet::call_index(9)]
		#[pallet::weight(T::WeightInfo::buy_asset())]
		pub fn buy_asset(
			origin: OriginFor<T>,
			asset_id: AssetIdOf<T, I>,
			payment: Option<FungiblesAssetIdOf<T, I>>,
		) -> DispatchResult {
			let buyer = ensure_signed(origin)?;
			let GeneralConfig { trade, .. } = GeneralConfigStore::<T, I>::get();
			ensure!(trade.open, Error::<T, I>::TradeClosed);

			let (seller, price) = Self::ensure_for_trade(&asset_id)?;
			ensure!(buyer != seller, Error::<T, I>::AlreadyOwned);
			<T::Fungible as fungible::Mutate<_>>::transfer(
				&buyer,
				&seller,
				price,
				Preservation::Protect,
			)?;

			let asset_season_id = T::SeasonHandler::get_season_id_for(&asset_id)?;
			let current_season_id = T::SeasonHandler::get_current_season_id()?;
			let fee = T::SeasonHandler::get_season_config_for(&current_season_id)?.fee;

			let trade_fee = {
				let min_buy_fee = fee.buy_asset_min;
				let percentage_fee = price.saturating_mul(fee.buy_percent.unique_saturated_into()) /
					MAX_PERCENTAGE.unique_saturated_into();
				sp_std::cmp::max(min_buy_fee, percentage_fee)
			};

			T::FeeHandler::withdraw_and_pay_fees(
				&buyer,
				payment.unwrap_or_else(FungiblesAssetIdOf::<T, I>::get_native_id),
				trade_fee,
				&asset_season_id,
				&AffiliateMethods::TradeAsset,
				&Self::treasury_account_id(),
			)?;

			Self::do_transfer_asset(&seller, &buyer, &asset_season_id, &asset_id)?;
			AssetTradePrices::<T, I>::remove(&asset_season_id, &asset_id);

			PlayerSeasonStats::<T, I>::mutate(&buyer, &current_season_id, |stats| {
				stats.bought_amount.saturating_inc();
			});
			PlayerSeasonStats::<T, I>::mutate(&seller, &current_season_id, |stats| {
				stats.sold_amount.saturating_inc();
			});

			Self::deposit_event(Event::AssetTraded { asset_id, from: seller, to: buyer, price });
			Ok(())
		}

		/// Locks an asset, making it unavailable for use.
		#[pallet::call_index(10)]
		#[pallet::weight(T::WeightInfo::lock_asset())]
		pub fn lock_asset(origin: OriginFor<T>, asset_id: AssetIdOf<T, I>) -> DispatchResult {
			let player = ensure_signed(origin)?;
			<Self as AssetManager>::lock_asset(*SAGE_LOCK_ID, player, asset_id)?;
			Ok(())
		}

		/// Unlocks an asset, making it available for use again.
		#[pallet::call_index(11)]
		#[pallet::weight(T::WeightInfo::unlock_asset())]
		pub fn unlock_asset(origin: OriginFor<T>, asset_id: AssetIdOf<T, I>) -> DispatchResult {
			let player = ensure_signed(origin)?;
			<Self as AssetManager>::unlock_asset(*SAGE_LOCK_ID, player, asset_id)?;
			Ok(())
		}

		/// Attempts to unlock the selected feature for the `target`.
		#[pallet::call_index(12)]
		#[pallet::weight(
			T::WeightInfo::unlock_trade_asset_feature()
				.max(T::WeightInfo::unlock_transfer_asset_feature())
        )]
		pub fn unlock_feature(
			origin: OriginFor<T>,
			target: UnlockTarget<AccountIdOf<T>>,
			feature: LockableFeature,
			season_id: SeasonIdOf<T, I>,
			payment: Option<FungiblesAssetIdOf<T, I>>,
		) -> DispatchResult {
			let account = ensure_signed(origin)?;
			T::SeasonHandler::is_valid_season(&season_id)?;
			let payment = payment.unwrap_or_else(FungiblesAssetIdOf::<T, I>::get_native_id);

			match feature {
				LockableFeature::TradeAsset =>
					Self::unlock_asset_trading_for(account, target, season_id, payment),
				LockableFeature::TransferAsset =>
					Self::unlock_asset_transfer_for(account, target, season_id, payment),
			}
		}

		/// Entry point for the custom state transition.
		#[pallet::call_index(13)]
		#[pallet::weight(T::WeightInfo::state_transition(6))]
		pub fn state_transition(
			origin: OriginFor<T>,
			transition_id: TransitionIdOf<T, I>,
			asset_ids: Vec<AssetIdOf<T, I>>,
			extra: ExtraOf<T, I>,
			payment_kind: Option<FungiblesAssetIdOf<T, I>>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(
				asset_ids.len() <= MAX_ASSETS_IN_TRANSITION,
				Error::<T, I>::TooManyAssetsInTransition
			);

			for asset_id in asset_ids.iter() {
				Self::ensure_unlocked(asset_id)?;
			}
			let transition_results = T::SageGameTransition::do_transition(
				&transition_id,
				&sender,
				&asset_ids,
				&extra,
				payment_kind.clone(),
			)
			.map_err(<Error<T, I>>::from)?;
			let current_season_id = T::SeasonHandler::get_current_season_id()?;
			let payment = payment_kind.unwrap_or_else(FungiblesAssetIdOf::<T, I>::get_native_id);
			Self::process_transition_results(
				&sender,
				&current_season_id,
				transition_results,
				payment.clone(),
			)?;

			let transition_fee = {
				let SeasonConfigOf::<T, I> { fee, .. } =
					T::SeasonHandler::get_season_config_for(&current_season_id)?;
				fee.state_transition_base_fee
			};

			T::FeeHandler::withdraw_and_pay_fees(
				&sender,
				payment,
				transition_fee,
				&current_season_id,
				&AffiliateMethodsOf::<T, I>::StateTransition(transition_id.clone()),
				&Self::treasury_account_id(),
			)?;

			Self::deposit_event(Event::TransitionExecuted { account: sender, id: transition_id });

			Ok(())
		}
	}

	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		pub fn treasury_account_id() -> T::AccountId {
			T::PalletId::get().into_account_truncating()
		}

		pub fn technical_account_id() -> T::AccountId {
			T::PalletId::get().into_sub_account_truncating(b"technical")
		}

		pub fn assets_funds_pot() -> T::AccountId {
			T::PalletId::get().into_sub_account_truncating(b"assets_funds")
		}

		pub(crate) fn asset_with_owner(
			asset_id: &AssetIdOf<T, I>,
		) -> Result<(AccountIdOf<T>, AssetOf<T, I>), DispatchError> {
			let (owner, asset) =
				Assets::<T, I>::get(asset_id).ok_or(Error::<T, I>::UnknownAsset)?;
			Ok((owner, asset))
		}

		pub(crate) fn is_locked(asset_id: &AssetIdOf<T, I>) -> Option<Lock<AccountIdOf<T>>> {
			LockedAssets::<T, I>::get(asset_id)
		}

		pub(crate) fn do_transfer_asset(
			from: &AccountIdOf<T>,
			to: &AccountIdOf<T>,
			asset_season_id: &SeasonIdOf<T, I>,
			asset_id: &AssetIdOf<T, I>,
		) -> DispatchResult {
			let technical_account = Self::technical_account_id();

			if from != &technical_account {
				// The technical account doesn't keep track of the assets transferred to it
				// so these storage entries are only populated if the asset is being
				// transferred from a player account
				AssetOwners::<T, I>::remove((from, asset_season_id, asset_id));
				AssetsOwnedCount::<T, I>::mutate(from, asset_season_id, |owned_count| {
					owned_count.saturating_dec()
				});
			}

			if to != &Self::technical_account_id() {
				// The technical account doesn't keep track of the assets transferred to it
				// so these storage entries only need to be populated if the destination
				// to which the asset is transferred to is a player account
				AssetOwners::<T, I>::insert((to, asset_season_id, asset_id), ());
				AssetsOwnedCount::<T, I>::try_mutate(to, asset_season_id, |owned_count| {
					owned_count.saturating_inc();
					ensure!(
						*owned_count <=
							PlayerSeasonConfigs::<T, I>::get(to, asset_season_id)
								.inventory_tier
								.get_asset_slots(),
						Error::<T, I>::MaxOwnershipReached
					);
					Ok::<_, DispatchError>(())
				})?;
			}

			Assets::<T, I>::try_mutate(asset_id, |maybe_asset| -> DispatchResult {
				let (from_owner, _) = maybe_asset.as_mut().ok_or(Error::<T, I>::UnknownAsset)?;
				*from_owner = to.clone();
				Ok(())
			})
		}

		fn process_transition_results(
			player: &AccountIdOf<T>,
			season_id: &SeasonIdOf<T, I>,
			transition_results: Vec<TransitionOutputOf<T, I>>,
			payment_kind: FungiblesAssetIdOf<T, I>,
		) -> DispatchResult {
			let mut minted_amount = 0 as Stat;
			let mut mutated_amount = 0 as Stat;

			let mut player_asset_count = AssetsOwnedCount::<T, I>::get(player, season_id);
			let player_inventory_slots = PlayerSeasonConfigs::<T, I>::get(player, season_id)
				.inventory_tier
				.get_asset_slots();

			for output in transition_results {
				match output {
					TransitionOutput::Minted(asset) => {
						minted_amount.saturating_inc();
						player_asset_count.saturating_inc();
						ensure!(
							player_asset_count <= player_inventory_slots,
							Error::<T, I>::MaxOwnershipReached
						);
						let asset_id = asset.get_id();

						T::SeasonHandler::register_asset_in(&asset_id, season_id)?;
						Assets::<T, I>::insert(&asset_id, (player, asset));
						AssetOwners::<T, I>::insert((player, season_id, &asset_id), ());
					},
					TransitionOutput::Mutated(asset_id, asset) => {
						mutated_amount.saturating_inc();
						Assets::<T, I>::mutate(asset_id, |maybe_asset| {
							if let Some((_, old_asset)) = maybe_asset {
								*old_asset = asset;
							}
						});
					},
					TransitionOutput::Consumed(asset_id) => {
						if let Some((owner, _)) = Assets::<T, I>::take(&asset_id) {
							let asset_season_id = T::SeasonHandler::get_season_id_for(&asset_id)?;
							AssetOwners::<T, I>::remove((&owner, &asset_season_id, &asset_id));
							AssetsOwnedCount::<T, I>::mutate(
								&owner,
								&asset_season_id,
								|asset_count| {
									asset_count.saturating_dec();
								},
							);

							// If the asset has some funds, we transfer all to the owner.
							// Todo: shall this be made configurable, like partly flowing into a
							// treasury?
							if Self::inspect_asset_funds(&asset_id, &payment_kind) >
								Default::default()
							{
								Self::transfer_all_from_asset(
									&asset_id,
									&owner,
									payment_kind.clone(),
								)?;
							}
						}
					},
				}
			}

			if minted_amount > 0 {
				AssetsOwnedCount::<T, I>::mutate(player, season_id, |asset_count| {
					*asset_count = asset_count.saturating_add(minted_amount as u8);
				});
			}

			PlayerSeasonStats::<T, I>::mutate(player, season_id, |stats| {
				stats.minted_amount = stats.minted_amount.saturating_add(minted_amount);
				stats.forged_amount = stats.forged_amount.saturating_add(mutated_amount);

				if stats.first_mint.is_none() && minted_amount > 0 {
					stats.first_mint = Some(<frame_system::Pallet<T>>::block_number())
				}

				if stats.first_forge.is_none() && mutated_amount > 0 {
					stats.first_forge = Some(<frame_system::Pallet<T>>::block_number())
				}
			});

			Ok(())
		}
	}
}
