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
pub mod impls;
#[cfg(test)]
pub mod mock;

use ajuna_primitives::{
	account_manager::{AccountManager, WhitelistKey},
	asset_manager::{AssetManager, Lock, LockIdentifier},
	fee_handler::FeeHandler,
	season_manager::SeasonManager,
	treasury_manager::TreasuryManager,
};
use sage_api::{AsErrorCode, Error as SageApiError, SageApi, SageGameTransition};

use frame_support::{pallet_prelude::*, traits::Currency};
use frame_system::pallet_prelude::*;
use sp_std::prelude::*;

use weights::WeightInfo;

pub use config::*;
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::traits::{ExistenceRequirement::AllowDeath, WithdrawReasons};

	#[pallet::pallet]
	pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

	pub type AssetIdOf<T, I> =
		<<T as Config<I>>::SageGameTransition as SageGameTransition>::AssetId;
	pub type AssetOf<T, I> = <<T as Config<I>>::SageGameTransition as SageGameTransition>::Asset;
	pub type SeasonIdOf<T, I> = <<T as Config<I>>::SeasonHandle as SeasonManager>::SeasonId;
	pub type BalanceOf<T, I> = <<T as Config<I>>::Currency as Currency<AccountIdOf<T>>>::Balance;
	pub type TransitionIdOf<T, I> =
		<<T as Config<I>>::SageGameTransition as SageGameTransition>::TransitionId;
	pub type ExtraOf<T, I> = <<T as Config<I>>::SageGameTransition as SageGameTransition>::Extra;
	pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub type TransitionConfigOf<T, I> =
		<<T as Config<I>>::SageGameTransition as SageGameTransition>::TransitionConfig;
	pub type GeneralConfigOf<T, I> = GeneralConfig<TransitionConfigOf<T, I>>;

	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config {
		type SageGameTransition: SageGameTransition<SageApi = Self::SageApi>;

		// This associated type mostly exists to constraint the SageApi's associated types.
		type SageApi: SageApi<Balance = BalanceOf<Self, I>, AccountId = AccountIdOf<Self>>;

		type SeasonHandle: SeasonManager<AssetId = AssetIdOf<Self, I>>;

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
	pub type UnlockConfigs<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
		_,
		Identity,
		SeasonIdOf<T, I>,
		Identity,
		LockableFeature,
		UnlockConfig,
		ValueQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		/// An organizer has been set.
		OrganizerSet { organizer: T::AccountId },
		/// Global configuration updated.
		UpdatedGlobalConfig { updated_config: GeneralConfigOf<T, I> },
		/// Unlock configuration updated for feature.
		UpdatedUnlockConfig {
			season_id: SeasonIdOf<T, I>,
			feature: LockableFeature,
			updated_config: UnlockConfig,
		},
		/// A transition has been executed.
		TransitionExecuted {
			/// Account who initiated execution.
			account: T::AccountId,
			/// Transition ID that was executed.
			id: TransitionIdOf<T, I>,
		},
	}

	/// Error for the treasury pallet.
	#[pallet::error]
	pub enum Error<T, I = ()> {
		/// There is no account set as the organizer
		OrganizerNotSet,
		/// The rule for a given transition was not satisfied.
		RuleNotSatisfied { code: u8 },
		/// An error occurred during the state transition.
		Transition { code: u8 },
	}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		/// Set game organizer.
		#[pallet::call_index(0)]
		#[pallet::weight({10_000})]
		pub fn set_organizer(origin: OriginFor<T>, organizer: T::AccountId) -> DispatchResult {
			ensure_root(origin)?;
			Organizer::<T, I>::put(&organizer);
			Self::deposit_event(Event::OrganizerSet { organizer });
			Ok(())
		}

		/// Update general configuration.
		#[pallet::call_index(1)]
		#[pallet::weight({10_000})]
		pub fn update_general_config(
			origin: OriginFor<T>,
			new_config: GeneralConfigOf<T, I>,
		) -> DispatchResult {
			Self::ensure_organizer(origin)?;
			GeneralConfigStore::<T, I>::put(&new_config);
			Self::deposit_event(Event::UpdatedGlobalConfig { updated_config: new_config });
			Ok(())
		}

		#[pallet::call_index(2)]
		#[pallet::weight({1000})]
		pub fn update_unlock_rules_for(
			origin: OriginFor<T>,
			season_id: SeasonIdOf<T, I>,
			feature: LockableFeature,
			unlock_config: UnlockConfig,
		) -> DispatchResult {
			Self::ensure_organizer(origin)?;
			UnlockConfigs::<T, I>::mutate(&season_id, &feature, |config| {
				*config = unlock_config.clone();
			});
			Self::deposit_event(Event::UpdatedUnlockConfig {
				season_id,
				feature,
				updated_config: unlock_config,
			});
			Ok(())
		}

		/// Upgrade the asset inventory space.
		#[pallet::call_index(3)]
		#[pallet::weight({10_000})]
		pub fn upgrade_asset_inventory(
			origin: OriginFor<T>,
			beneficiary: Option<AccountIdOf<T>>,
			in_season: Option<SeasonIdOf<T, I>>,
		) -> DispatchResult {
			/*let caller = ensure_signed(origin)?;
			let (season_id, Season { fee, .. }) = {
				if let Some(season_id) = in_season {
					(season_id, Self::seasons(&season_id)?)
				} else {
					Self::current_season_with_id()?
				}
			};
			let account_to_upgrade = beneficiary.unwrap_or_else(|| caller.clone());

			let storage_tier =
				PlayerSeasonConfigs::<T>::get(&account_to_upgrade, season_id).storage_tier;
			ensure!(storage_tier != StorageTier::Max, Error::<T>::MaxStorageTierReached);

			let upgrade_fee = {
				let base_fee = fee.upgrade_storage;
				let GlobalConfig { affiliate_config, .. } = GlobalConfigs::<T>::get();

				if affiliate_config.mode == AffiliateMode::Open &&
					affiliate_config.enabled_in_upgrade
				{
					T::FeeHandler::try_propagate_chain_fee(
						base_fee,
						&caller,
						&AffiliateMethods::UpgradeStorage,
					)?
				} else {
					base_fee
				}
			};

			T::Currency::withdraw(&caller, upgrade_fee, WithdrawReasons::FEE, AllowDeath)?;
			Self::deposit_into_treasury(&season_id, upgrade_fee);

			PlayerSeasonConfigs::<T>::mutate(&account_to_upgrade, season_id, |account| {
				account.storage_tier = storage_tier.upgrade()
			});
			Self::deposit_event(Event::StorageTierUpgraded {
				account: account_to_upgrade,
				season_id,
			});*/
			Ok(())
		}

		/// Entry point for the custom state transition.
		#[pallet::weight(T::WeightInfo::state_transition())]
		#[pallet::call_index(10)]
		pub fn state_transition(
			origin: OriginFor<T>,
			transition_id: TransitionIdOf<T, I>,
			asset_ids: Vec<AssetIdOf<T, I>>,
			extra: ExtraOf<T, I>,
		) -> DispatchResult {
			let sender = ensure_signed(origin)?;

			T::SageGameTransition::verify_rule(transition_id.clone(), &sender, &asset_ids, &extra)
				.map_err(|e| Error::<T, I>::RuleNotSatisfied { code: e.as_error_code() })?;

			T::SageGameTransition::do_transition(
				transition_id.clone(),
				sender.clone(),
				asset_ids,
				extra,
			)
			.map_err(|e| Error::<T, I>::Transition { code: e.as_error_code() })?;

			Self::deposit_event(Event::TransitionExecuted { account: sender, id: transition_id });

			Ok(())
		}
	}

	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		/// Check if origin is the current organizer account
		pub(crate) fn ensure_organizer(
			origin: OriginFor<T>,
		) -> Result<T::AccountId, DispatchError> {
			let maybe_organizer = ensure_signed(origin)?;
			let existing_organizer =
				Organizer::<T, I>::get().ok_or(Error::<T, I>::OrganizerNotSet)?;
			ensure!(maybe_organizer == existing_organizer, DispatchError::BadOrigin);
			Ok(maybe_organizer)
		}
	}
}
