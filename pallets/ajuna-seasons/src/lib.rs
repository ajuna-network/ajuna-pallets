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

pub use pallet::*;

pub mod weights;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
mod impls;
#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
mod types;

use ajuna_primitives::{
	account_manager::AccountManager,
	season_manager::{SeasonConfig, SeasonManager},
};

use frame_support::{pallet_prelude::*, traits::Currency};
use frame_system::pallet_prelude::*;

pub use types::*;
use weights::WeightInfo;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use ajuna_primitives::season_manager::Validate;
	use sp_runtime::{traits::AtLeast32BitUnsigned, ArithmeticError};

	#[pallet::pallet]
	pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

	pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub type SeasonIdOf<T, I> = <T as Config<I>>::SeasonId;
	pub type SeasonDataOf<T, I> = <T as Config<I>>::SeasonData;
	pub type AssetIdOf<T, I> = <T as Config<I>>::AssetId;

	pub(crate) type SeasonStatusOf<T, I> = SeasonStatus<SeasonIdOf<T, I>>;
	pub(crate) type SeasonConfigOf<T, I> = SeasonConfig<BalanceOf<T, I>, SeasonDataOf<T, I>>;
	pub(crate) type SeasonScheduleOf<T> = SeasonSchedule<BlockNumberFor<T>>;

	pub type BalanceOf<T, I> = <<T as Config<I>>::Currency as Currency<AccountIdOf<T>>>::Balance;

	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self, I>>
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type SeasonId: Member + Parameter + MaxEncodedLen;
		type SeasonData: Member + Parameter + MaxEncodedLen + Validate;

		type AssetId: Member + Parameter + MaxEncodedLen + TypeInfo;

		type AccountHandler: AccountManager<AccountId = AccountIdOf<Self>>;

		type Currency: Currency<AccountIdOf<Self>>;

		type WeightInfo: WeightInfo;
	}

	#[pallet::storage]
	pub type CurrentSeasonStatus<T: Config<I>, I: 'static = ()> =
		StorageValue<_, SeasonStatusOf<T, I>, ResultQuery<Error<T, I>::NoActiveSeason>>;

	/// Latest SeasonId created through 'update_season'
	#[pallet::storage]
	pub type LatestSeason<T: Config<I>, I: 'static = ()> =
		StorageValue<_, SeasonIdOf<T, I>, OptionQuery>;

	#[pallet::storage]
	pub type FinishedSeasons<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, SeasonIdOf<T, I>, (), ValueQuery>;

	/// Use to represent a linked list of SeasonId. All entries will have a
	/// value indicating the next season id to them except the latest season added
	/// which will not have a value for it.
	#[pallet::storage]
	pub type NextSeasonChain<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, SeasonIdOf<T, I>, SeasonIdOf<T, I>, OptionQuery>;

	/// Use to represent a linked list of SeasonId. All entries will have a
	/// value indicating the previous season id to them except the firsts season added
	/// which will not have a value for it.
	#[pallet::storage]
	pub type PrevSeasonChain<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, SeasonIdOf<T, I>, SeasonIdOf<T, I>, OptionQuery>;

	/// Storage for the seasons.
	#[pallet::storage]
	pub type Seasons<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, SeasonIdOf<T, I>, SeasonConfigOf<T, I>, OptionQuery>;

	/// Storage for the season's metadata.
	#[pallet::storage]
	pub type SeasonMetadatas<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, SeasonIdOf<T, I>, SeasonMetadata, OptionQuery>;

	/// Storage for the season's schedules.
	#[pallet::storage]
	pub type SeasonSchedules<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, SeasonIdOf<T, I>, SeasonScheduleOf<T>, OptionQuery>;

	/// Stores the assets season id registration.
	#[pallet::storage]
	pub type AssetSeasonRegister<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, AssetIdOf<T, I>, SeasonIdOf<T, I>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		/// The season configuration for {season_id} has been updated.
		UpdatedSeason {
			season_id: SeasonIdOf<T, I>,
			config: Option<SeasonConfigOf<T, I>>,
			metadata: Option<SeasonMetadata>,
			schedule: Option<SeasonScheduleOf<T>>,
		},
	}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		/// The season's data could not be validated.
		SeasonDataNotValid,
		/// There is currently no active season
		NoActiveSeason,
		/// The season starts before the previous season has ended.
		EarlyStartTooEarly,
		/// The season's early start is earlier than its normal start.
		EarlyStartTooLate,
		/// The season's start block is greater than its end block.
		SeasonStartTooLate,
		/// The schedule for a given season id could not be found.
		SeasonScheduleNotSet,
		/// The given asset was not registered in any season.
		AssetNotRegistered,
		/// The given season identifier has not be registered.
		InvalidSeason,
	}

	#[pallet::hooks]
	impl<T: Config<I>, I: 'static> Hooks<BlockNumberFor<T>> for Pallet<T, I> {}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		#[pallet::call_index(0)]
		#[pallet::weight({10_000})]
		pub fn update_season(
			origin: OriginFor<T>,
			season_id: SeasonIdOf<T, I>,
			config: Option<SeasonConfigOf<T, I>>,
			metadata: Option<SeasonMetadata>,
			schedule: Option<SeasonScheduleOf<T>>,
		) -> DispatchResult {
			let account = ensure_signed(origin)?;
			T::AccountHandler::is_organizer(&account)?;

			// If season already started or is finished
			// don't allow any changes except metadata
			if Self::is_season_modifiable(&season_id) {
				if let Some(ref config_update) = config {
					Self::ensure_valid_config(config_update)?;
					Seasons::<T, I>::insert(&season_id, config_update);
				}

				if let Some(ref schedule_update) = schedule {
					Self::ensure_valid_schedule(&season_id, schedule_update)?;
					SeasonSchedules::<T, I>::insert(&season_id, schedule_update);
				}
			}

			if let Some(ref meta_update) = metadata {
				SeasonMetadatas::<T, I>::insert(&season_id, meta_update);
			}

			if !PrevSeasonChain::<T, I>::contains_key(&season_id) {
				// If the 'season_id' key is not already in 'PrevSeasonChain',
				// then that means that this is the first time we call this method with
				// 'season_id', in that case we build the chain link for the given 'season_id'.
				// Once the link has been built all subsequent calls will never modify it.
				Self::insert_season_chains(&season_id);
			}

			Self::deposit_event(Event::UpdatedSeason { season_id, config, metadata, schedule });

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight({10_000})]
		pub fn interrupt_active_season(origin: OriginFor<T>) -> DispatchResult {
			let account = ensure_signed(origin)?;
			T::AccountHandler::is_organizer(&account)?;

			// TODO

			Ok(())
		}
	}

	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		fn insert_season_chains(new_season_id: &SeasonIdOf<T, I>) {
			LatestSeason::<T, I>::mutate(|maybe_prev_season_id| {
				if let Some(prev_season_id) = maybe_prev_season_id {
					// If there was a previous season we can build the chain links
					PrevSeasonChain::<T, I>::insert(new_season_id, &*prev_season_id);
					NextSeasonChain::<T, I>::insert(&*prev_season_id, new_season_id);

					*prev_season_id = new_season_id.clone();
				} else {
					// If this is the first season ever created
					// we won't build the chain links
					// since we are missing another season to do so
					*maybe_prev_season_id = Some(new_season_id.clone());
				}
			});
		}

		fn is_season_modifiable(season_id: &SeasonIdOf<T, I>) -> bool {
			if let Ok(current_season) = CurrentSeasonStatus::<T, I>::get() {
				(&current_season.season_id != season_id) &&
					!FinishedSeasons::<T, I>::contains_key(season_id)
			} else {
				true
			}
		}

		fn ensure_valid_config(config: &SeasonConfigOf<T, I>) -> DispatchResult {
			ensure!(config.validate_data(), Error::<T, I>::SeasonDataNotValid);
			Ok(())
		}

		fn ensure_valid_schedule(
			season_id: &SeasonIdOf<T, I>,
			schedule: &SeasonScheduleOf<T>,
		) -> DispatchResult {
			ensure!(schedule.early_start < schedule.start, Error::<T, I>::EarlyStartTooLate);
			ensure!(schedule.start < schedule.end, Error::<T, I>::SeasonStartTooLate);

			if let Some(prev_season_id) = PrevSeasonChain::<T, I>::get(season_id) {
				let prev_schedule = SeasonSchedules::<T, I>::get(prev_season_id)
					.ok_or(Error::<T, I>::SeasonScheduleNotSet)?;
				ensure!(
					prev_schedule.end < schedule.early_start,
					Error::<T, I>::EarlyStartTooEarly
				);
			}

			if let Some(next_season_id) = NextSeasonChain::<T, I>::get(season_id) {
				let next_schedule = SeasonSchedules::<T, I>::get(next_season_id)
					.ok_or(Error::<T, I>::SeasonScheduleNotSet)?;
				ensure!(
					schedule.end < next_schedule.early_start,
					Error::<T, I>::EarlyStartTooEarly
				);
			}

			Ok(())
		}
	}
}
