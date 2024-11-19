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

#![cfg_attr(not(feature = "std"), no_std)]

pub mod weights;

pub mod config;
mod pallet_impls;
mod trait_impls;

#[cfg(test)]
pub mod mock;
#[cfg(test)]
mod tests;

use ajuna_primitives::{
	account_manager::{AccountManager, WhitelistKey},
	asset_manager::{AssetManager, Lock, LockIdentifier},
	fee_handler::FeeHandler,
	season_manager::{SeasonConfig, SeasonManager},
	trade_manager::TradeManager,
};
use sage_api::{AsErrorCode, SageApi, SageGameTransition};

use frame_support::{pallet_prelude::*, traits::Currency, PalletId};
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

	#[pallet::pallet]
	pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

	pub type AssetIdOf<T, I> =
		<<T as Config<I>>::SageGameTransition as SageGameTransition>::AssetId;
	pub type AssetOf<T, I> = <<T as Config<I>>::SageGameTransition as SageGameTransition>::Asset;
	pub type SeasonIdOf<T, I> =
		<<T as Config<I>>::SeasonHandler as SeasonManager<TransitionIdOf<T, I>>>::SeasonId;
	pub type BalanceOf<T, I> = <<T as Config<I>>::Currency as Currency<AccountIdOf<T>>>::Balance;
	pub type TransitionIdOf<T, I> =
		<<T as Config<I>>::SageGameTransition as SageGameTransition>::TransitionId;
	pub type ExtraOf<T, I> = <<T as Config<I>>::SageGameTransition as SageGameTransition>::Extra;
	pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub type TransitionConfigOf<T, I> =
		<<T as Config<I>>::SageGameTransition as SageGameTransition>::TransitionConfig;
	pub type GeneralConfigOf<T, I> = GeneralConfig<TransitionConfigOf<T, I>>;

	pub(crate) type PlayerStatsOf<T> = PlayerStats<BlockNumberFor<T>>;

	pub(crate) type BoundedAssetIdsOf<T, I> = BoundedVec<AssetIdOf<T, I>, MaxAssetsPerPlayer>;

	pub(crate) type SeasonConfigOf<T, I> = SeasonConfig<BalanceOf<T, I>, TransitionIdOf<T, I>>;

	pub(crate) type TradeFilterOf<T, I> =
		<<T as Config<I>>::TradeHandler as TradeManager>::TradeFilter;
	pub(crate) type AffiliateMethodsOf<T, I> = AffiliateMethods<TransitionIdOf<T, I>>;

	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config {
		#[pallet::constant]
		type PalletId: Get<PalletId>;
		type SageGameTransition: SageGameTransition<SageApi = Self::SageApi>;

		// This associated type mostly exists to constraint the SageApi's associated types.
		type SageApi: SageApi<Balance = BalanceOf<Self, I>, AccountId = AccountIdOf<Self>>;

		type SeasonHandler: SeasonManager<
			TransitionIdOf<Self, I>,
			AssetId = AssetIdOf<Self, I>,
			Balance = BalanceOf<Self, I>,
		>;

		type FeeHandler: FeeHandler<
			AccountId = AccountIdOf<Self>,
			FeeCurrency = BalanceOf<Self, I>,
			AffiliateFeeIdentifier = AffiliateMethodsOf<Self, I>,
			TournamentFeeIdentifier = SeasonIdOf<Self, I>,
			// TODO: Define if we want this as a configurable parameter or some different fixed
			// value
			TreasuryKey = SeasonIdOf<Self, I>,
		>;

		type TradeHandler: TradeManager<Asset = AssetOf<Self, I>>;

		type Currency: Currency<AccountIdOf<Self>>;

		/// The overarching event type.
		type RuntimeEvent: From<Event<Self, I>>
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The weight calculations
		type WeightInfo: WeightInfo;
	}

	#[pallet::storage]
	pub type Organizer<T: Config<I>, I: 'static = ()> =
		StorageValue<_, AccountIdOf<T>, OptionQuery>;

	#[pallet::storage]
	pub type GeneralConfigStore<T: Config<I>, I: 'static = ()> =
		StorageValue<_, GeneralConfigOf<T, I>, ValueQuery>;

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

	#[pallet::storage]
	pub type PlayerSeasonStats<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
		_,
		Identity,
		AccountIdOf<T>,
		Identity,
		SeasonIdOf<T, I>,
		PlayerStats<BlockNumberFor<T>>,
		ValueQuery,
	>;

	#[pallet::storage]
	pub type Assets<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, AssetIdOf<T, I>, (AccountIdOf<T>, AssetOf<T, I>)>;

	#[pallet::storage]
	pub type AssetOwners<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
		_,
		Identity,
		AccountIdOf<T>,
		Identity,
		SeasonIdOf<T, I>,
		BoundedAssetIdsOf<T, I>,
		ValueQuery,
	>;

	#[pallet::storage]
	pub type SeasonTradeFilters<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, SeasonIdOf<T, I>, TradeFilterOf<T, I>, ValueQuery>;

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

	#[pallet::storage]
	pub type LockedAssets<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, AssetIdOf<T, I>, Lock<AccountIdOf<T>>>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		/// An organizer has been set.
		OrganizerSet { organizer: AccountIdOf<T> },
		/// General configuration updated.
		UpdatedGeneralConfig { updated_config: GeneralConfigOf<T, I> },
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
		/// Asset transferred.
		AssetTransferred { from: AccountIdOf<T>, to: AccountIdOf<T>, asset_id: AssetIdOf<T, I> },
		/// Trade filter has been updated
		UpdatedTradeFilter { season_id: SeasonIdOf<T, I>, filter: TradeFilterOf<T, I> },
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

	/// Error for the treasury pallet.
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
		/// Tried transferring to his or her own account.
		CannotTransferToSelf,
		/// The feature is locked for the current player
		FeatureLocked,
		/// The feature trying to be unlocked is not available for the selected season
		FeatureLockedInSeason,
		/// The feature trying to be unlocked cannot be unlocked with payment
		FeatureLockedThroughPayment,
		/// The feature trying to be unlocked has missing requirements to be fulfilled by
		/// the account trying to unlock it
		UnlockCriteriaNotFulfilled,
		/// The rule for a given transition was not satisfied.
		RuleNotSatisfied { code: u8 },
		/// The amount of input assets in the transition is greater than 'MAX_ASSETS_IN_TRANSITION'
		TooManyAssetsInTransition,
		/// An error occurred during the state transition.
		Transition { code: u8 },
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
			new_config: GeneralConfigOf<T, I>,
		) -> DispatchResult {
			Self::ensure_organizer(origin)?;
			GeneralConfigStore::<T, I>::put(&new_config);
			Self::deposit_event(Event::UpdatedGeneralConfig { updated_config: new_config });
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::update_unlock_rule())]
		pub fn update_unlock_rule(
			origin: OriginFor<T>,
			season_id: SeasonIdOf<T, I>,
			feature: LockableFeature,
			unlock_rule: UnlockRule,
		) -> DispatchResult {
			Self::ensure_organizer(origin)?;
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
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::upgrade_asset_inventory())]
		pub fn upgrade_asset_inventory(
			origin: OriginFor<T>,
			beneficiary: Option<AccountIdOf<T>>,
			in_season: Option<SeasonIdOf<T, I>>,
		) -> DispatchResult {
			let caller = ensure_signed(origin)?;
			let (season_id, SeasonConfigOf::<T, I> { fee, .. }) = if let Some(season_id) = in_season
			{
				let season_config = T::SeasonHandler::get_season_config_for(&season_id)?;
				(season_id, season_config)
			} else {
				let current_season_id = T::SeasonHandler::get_current_season_id();
				let season_config = T::SeasonHandler::get_season_config_for(&current_season_id)?;
				(current_season_id, season_config)
			};

			let account_to_upgrade = beneficiary.unwrap_or_else(|| caller.clone());

			let inventory_tier =
				PlayerSeasonConfigs::<T, I>::get(&account_to_upgrade, &season_id).inventory_tier;
			ensure!(inventory_tier != InventoryTier::Max, Error::<T, I>::MaxStorageTierReached);

			let upgrade_fee = {
				let base_fee = fee.upgrade_asset_inventory;
				T::FeeHandler::try_propagate_chain_fee(
					base_fee,
					&caller,
					&AffiliateMethods::UpgradeAssetInventory,
				)?
			};
			T::FeeHandler::deposit_fee_into_treasury(&caller, &season_id, upgrade_fee)?;

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

		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::transfer_asset())]
		pub fn transfer_asset(
			origin: OriginFor<T>,
			to: AccountIdOf<T>,
			asset_id: AssetIdOf<T, I>,
		) -> DispatchResult {
			let GeneralConfig { transfer, .. } = GeneralConfigStore::<T, I>::get();
			let from = match Self::ensure_organizer(origin.clone()) {
				Ok(organizer) => organizer,
				_ => {
					ensure!(transfer.open, Error::<T, I>::TransferClosed);
					ensure_signed(origin)?
				},
			};
			ensure!(from != to, Error::<T, I>::CannotTransferToSelf);
			ensure!(
				Self::ensure_for_trade(&asset_id).is_err(),
				Error::<T, I>::CannotTransferAssetInTrade
			);
			Self::ensure_unlocked(&asset_id)?;

			Self::ensure_ownership(&from, &asset_id)?;
			let asset_season_id = T::SeasonHandler::get_season_id_for(&asset_id)?;
			ensure!(
				PlayerSeasonConfigs::<T, I>::get(&from, &asset_season_id).locks.asset_transfer,
				Error::<T, I>::FeatureLocked
			);

			let SeasonConfigOf::<T, I> { fee, .. } =
				T::SeasonHandler::get_season_config_for(&asset_season_id)?;
			T::FeeHandler::deposit_fee_into_treasury(&from, &asset_season_id, fee.transfer_asset)?;

			Self::do_transfer_asset(&from, &to, &asset_season_id, &asset_id)?;
			Self::deposit_event(Event::AssetTransferred { from, to, asset_id });
			Ok(())
		}

		#[pallet::call_index(5)]
		#[pallet::weight(T::WeightInfo::update_trade_filter())]
		pub fn update_trade_filter(
			origin: OriginFor<T>,
			season_id: SeasonIdOf<T, I>,
			trade_filter: TradeFilterOf<T, I>,
		) -> DispatchResult {
			Self::ensure_organizer(origin)?;

			SeasonTradeFilters::<T, I>::insert(&season_id, &trade_filter);

			Self::deposit_event(Event::UpdatedTradeFilter { season_id, filter: trade_filter });

			Ok(())
		}

		/// Set the price of a given asset, putting it on sale for others to buy.
		#[pallet::call_index(6)]
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
			let season_id = T::SeasonHandler::get_season_id_for(&asset_id)?;
			let config = PlayerSeasonConfigs::<T, I>::get(&seller, &season_id);
			ensure!(config.locks.asset_trade, Error::<T, I>::FeatureLocked);
			Self::ensure_unlocked(&asset_id)?;
			Self::ensure_can_be_set_for_trade(&asset_id, &asset)?;
			AssetTradePrices::<T, I>::insert(&season_id, &asset_id, price);
			Self::deposit_event(Event::AssetPriceSet { asset_id, price });
			Ok(())
		}

		/// Remove the price of an asset set on sale previously.
		#[pallet::call_index(7)]
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

		/// Attempt to buy the selected asset
		#[pallet::call_index(8)]
		#[pallet::weight(T::WeightInfo::buy_asset())]
		pub fn buy_asset(origin: OriginFor<T>, asset_id: AssetIdOf<T, I>) -> DispatchResult {
			let buyer = ensure_signed(origin)?;
			let GeneralConfig { trade, .. } = GeneralConfigStore::<T, I>::get();
			ensure!(trade.open, Error::<T, I>::TradeClosed);

			let (seller, price) = Self::ensure_for_trade(&asset_id)?;
			ensure!(buyer != seller, Error::<T, I>::AlreadyOwned);
			T::Currency::transfer(
				&buyer,
				&seller,
				price,
				frame_support::traits::ExistenceRequirement::KeepAlive,
			)?;

			let asset_season_id = T::SeasonHandler::get_season_id_for(&asset_id)?;
			let current_season_id = T::SeasonHandler::get_current_season_id();
			let SeasonConfigOf::<T, I> { fee, .. } =
				T::SeasonHandler::get_season_config_for(&current_season_id)?;

			let trade_fee = {
				let min_buy_fee = fee.buy_asset_min;
				let percentage_fee = price.saturating_mul(fee.buy_percent.unique_saturated_into()) /
					MAX_PERCENTAGE.unique_saturated_into();
				let base_fee = sp_std::cmp::max(min_buy_fee, percentage_fee);

				T::FeeHandler::try_propagate_chain_fee(
					base_fee,
					&buyer,
					&AffiliateMethods::TradeAsset,
				)?
			};
			T::FeeHandler::deposit_fee_into_treasury(&buyer, &asset_season_id, trade_fee)?;

			Self::do_transfer_asset(&seller, &buyer, &asset_season_id, &asset_id)?;
			AssetTradePrices::<T, I>::remove(&asset_season_id, &asset_id);

			let current_season_id = T::SeasonHandler::get_current_season_id();
			PlayerSeasonStats::<T, I>::mutate(&buyer, &current_season_id, |stats| {
				stats.bought_amount = stats.bought_amount.saturating_add(1);
			});
			PlayerSeasonStats::<T, I>::mutate(&seller, &current_season_id, |stats| {
				stats.sold_amount = stats.sold_amount.saturating_add(1);
			});

			Self::deposit_event(Event::AssetTraded { asset_id, from: seller, to: buyer, price });
			Ok(())
		}

		/// Locks an asset, making it unavailable for use.
		#[pallet::call_index(9)]
		#[pallet::weight(T::WeightInfo::lock_asset())]
		pub fn lock_asset(origin: OriginFor<T>, asset_id: AssetIdOf<T, I>) -> DispatchResult {
			let player = ensure_signed(origin)?;
			<Self as AssetManager>::lock_asset(*SAGE_LOCK_ID, player, asset_id)?;
			Ok(())
		}

		/// Unlocks an asset, making it available for use again.
		#[pallet::call_index(10)]
		#[pallet::weight(T::WeightInfo::unlock_asset())]
		pub fn unlock_asset(origin: OriginFor<T>, asset_id: AssetIdOf<T, I>) -> DispatchResult {
			let player = ensure_signed(origin)?;
			<Self as AssetManager>::unlock_asset(*SAGE_LOCK_ID, player, asset_id)?;
			Ok(())
		}

		/// Attempts to unlock the selected feature for the given player
		#[pallet::call_index(11)]
		#[pallet::weight(T::WeightInfo::unlock_feature())]
		pub fn unlock_feature(
			origin: OriginFor<T>,
			target: UnlockTarget<AccountIdOf<T>>,
			feature: LockableFeature,
			season_id: SeasonIdOf<T, I>,
		) -> DispatchResult {
			let account = ensure_signed(origin)?;
			T::SeasonHandler::is_valid_season(&season_id)?;

			match feature {
				LockableFeature::TradeAsset =>
					Self::unlock_asset_trading_for(account, target, season_id),
				LockableFeature::TransferAsset =>
					Self::unlock_asset_transfer_for(account, target, season_id),
			}
		}

		/// Entry point for the custom state transition.
		#[pallet::call_index(12)]
		#[pallet::weight(T::WeightInfo::state_transition(6))]
		pub fn state_transition(
			origin: OriginFor<T>,
			transition_id: TransitionIdOf<T, I>,
			asset_ids: Vec<AssetIdOf<T, I>>,
			extra: ExtraOf<T, I>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			ensure!(
				asset_ids.len() <= MAX_ASSETS_IN_TRANSITION,
				Error::<T, I>::TooManyAssetsInTransition
			);

			for asset_id in asset_ids.iter() {
				Self::ensure_ownership(&sender, asset_id)?;
				Self::ensure_unlocked(asset_id)?;
			}

			T::SageGameTransition::verify_rule(transition_id.clone(), &sender, &asset_ids, &extra)
				.map_err(|e| Error::<T, I>::RuleNotSatisfied { code: e.as_error_code() })?;

			T::SageGameTransition::do_transition(
				transition_id.clone(),
				sender.clone(),
				asset_ids,
				extra,
			)
			.map_err(|e| Error::<T, I>::Transition { code: e.as_error_code() })?;

			// TODO: Review the logic in this section
			let current_season_id = T::SeasonHandler::get_current_season_id();
			let transition_fee = {
				let SeasonConfigOf::<T, I> { fee, .. } =
					T::SeasonHandler::get_season_config_for(&current_season_id)?;

				let base_fee = fee.get_transition_fee_for(&transition_id);
				let updated_fee = T::FeeHandler::try_propagate_tournament_fee(
					base_fee,
					&sender,
					&current_season_id,
				)?;
				T::FeeHandler::try_propagate_chain_fee(
					updated_fee,
					&sender,
					&AffiliateMethodsOf::<T, I>::StateTransition(transition_id.clone()),
				)?
			};
			T::FeeHandler::deposit_fee_into_treasury(&sender, &current_season_id, transition_fee)?;

			Self::deposit_event(Event::TransitionExecuted { account: sender, id: transition_id });

			Ok(())
		}
	}

	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		pub fn technical_account_id() -> T::AccountId {
			T::PalletId::get().into_sub_account_truncating(b"technical")
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

		fn do_transfer_asset(
			from: &AccountIdOf<T>,
			to: &AccountIdOf<T>,
			season_id: &SeasonIdOf<T, I>,
			asset_id: &AssetIdOf<T, I>,
		) -> DispatchResult {
			let mut from_asset_ids = AssetOwners::<T, I>::get(from, season_id);
			from_asset_ids.retain(|owned_asset_id| owned_asset_id != asset_id);

			let mut to_asset_ids = AssetOwners::<T, I>::get(to, season_id);
			to_asset_ids
				.try_push(asset_id.clone())
				.map_err(|_| Error::<T, I>::MaxOwnershipReached)?;
			ensure!(
				to_asset_ids.len() <=
					PlayerSeasonConfigs::<T, I>::get(to, season_id).inventory_tier as usize,
				Error::<T, I>::MaxOwnershipReached
			);

			AssetOwners::<T, I>::mutate(from, season_id, |asset_ids| *asset_ids = from_asset_ids);
			AssetOwners::<T, I>::mutate(to, season_id, |asset_ids| *asset_ids = to_asset_ids);
			Assets::<T, I>::try_mutate(asset_id, |maybe_asset| -> DispatchResult {
				let (from_owner, _) = maybe_asset.as_mut().ok_or(Error::<T, I>::UnknownAsset)?;
				*from_owner = to.clone();
				Ok(())
			})
		}
	}
}
