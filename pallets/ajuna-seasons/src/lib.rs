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
	use sp_runtime::{traits::AtLeast32BitUnsigned, ArithmeticError};

	#[pallet::pallet]
	pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

	pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub type SeasonIdOf<T, I> = <T as Config<I>>::SeasonId;
	pub type AssetIdOf<T, I> = <T as Config<I>>::AssetId;
	pub type TransitionIdOf<T, I> = <T as Config<I>>::TransitionId;

	pub(crate) type SeasonOf<T, I> = Season<BlockNumberFor<T>, BalanceOf<T, I>>;
	pub(crate) type SeasonStatusOf<T, I> = SeasonStatus<SeasonIdOf<T, I>>;
	pub(crate) type SeasonConfigOf<T, I> = SeasonConfig<BalanceOf<T, I>, TransitionIdOf<T, I>>;
	pub(crate) type SeasonScheduleOf<T> = SeasonSchedule<BlockNumberFor<T>>;

	pub type BalanceOf<T, I> = <<T as Config<I>>::Currency as Currency<AccountIdOf<T>>>::Balance;

	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self, I>>
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type SeasonId: Member + Parameter + MaxEncodedLen;

		type AssetId: Member + Parameter + MaxEncodedLen + TypeInfo;

		type TransitionId: Member + Parameter + Ord + PartialOrd + MaxEncodedLen + TypeInfo;

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

	/// Use to represent a linked list of SeasonId. All entries will have a
	/// value indicating the previous season id to them except the first
	/// created season which will not have a value for it.
	#[pallet::storage]
	pub type SeasonChains<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, SeasonIdOf<T, I>, SeasonIdOf<T, I>, OptionQuery>;

	/// Storage for the seasons.
	#[pallet::storage]
	pub type Seasons<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, SeasonIdOf<T, I>, SeasonOf<T, I>, OptionQuery>;

	/// Storage for the season's metadata.
	#[pallet::storage]
	pub type SeasonMetadatas<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, SeasonIdOf<T, I>, SeasonMetadata, OptionQuery>;

	/// Storage for the season's schedules.
	#[pallet::storage]
	pub type SeasonSchedules<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, SeasonIdOf<T, I>, SeasonScheduleOf<T>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		/// The season configuration for {season_id} has been updated.
		UpdatedSeason {
			season_id: SeasonIdOf<T, I>,
			season: Option<SeasonOf<T, I>>,
			metadata: Option<SeasonMetadata>,
			schedule: Option<SeasonScheduleOf<T>>,
		},
	}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		/// There is currently no active season
		NoActiveSeason,
		/// The season starts before the previous season has ended.
		EarlyStartTooEarly,
		/// The season's early start is earlier than its normal start.
		EarlyStartTooLate,
		/// The season's start block is greater than its end block.
		SeasonStartTooLate,
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
			season: Option<SeasonOf<T, I>>,
			metadata: Option<SeasonMetadata>,
			schedule: Option<SeasonScheduleOf<T>>,
		) -> DispatchResult {
			let account = ensure_signed(origin)?;
			T::AccountHandler::is_organizer(&account)?;

			// TODO: If season already started or is finished
			// don't allow any change except metadata

			if let Some(ref season_update) = season {
				Self::ensure_valid_season(season_update)?;
				Seasons::<T, I>::insert(&season_id, season_update);
			}

			if let Some(ref meta_update) = metadata {
				SeasonMetadatas::<T, I>::insert(&season_id, meta_update);
			}

			if let Some(ref schedule_update) = schedule {
				Self::ensure_valid_schedule(&season_id, schedule_update)?;
				SeasonSchedules::<T, I>::insert(&season_id, schedule_update);
			}

			Self::update_season_chain(&season_id);

			Self::deposit_event(Event::UpdatedSeason { season_id, season, metadata, schedule });

			Ok(())
		}
	}

	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		fn update_season_chain(new_season_id: &SeasonIdOf<T, I>) {
			if SeasonChains::<T, I>::contains_key(new_season_id) {
				// If the new_season_id is already in the SeasonChains, then
				// that means that this is not the first time we call this method,
				// in that case there is nothing to do.
				// Since the chain link is established the first time the method is called.
				return;
			}

			let maybe_prev_season_id = LatestSeason::<T, I>::mutate(|maybe_season_id| {
				if let Some(season_id) = maybe_season_id {
					let prev_season_id = season_id.clone();
					*season_id = new_season_id.clone();
					Some(prev_season_id)
				} else {
					*maybe_season_id = Some(new_season_id.clone());
					None
				}
			});

			SeasonChains::<T, I>::insert(new_season_id, maybe_prev_season_id);
		}

		pub(crate) fn ensure_valid_season(season: &SeasonOf<T, I>) -> DispatchResult {
			// TODO
			Ok(())
		}

		pub(crate) fn ensure_valid_schedule(
			season_id: &SeasonIdOf<T, I>,
			schedule: &SeasonScheduleOf<T>,
		) -> DispatchResult {
			ensure!(schedule.early_start < schedule.start, Error::<T, I>::EarlyStartTooLate);
			ensure!(schedule.start < schedule.end, Error::<T, I>::SeasonStartTooLate);

			/*let prev_season_id = season_id.checked_sub(1).ok_or(ArithmeticError::Underflow)?;
			let next_season_id = season_id.checked_add(1).ok_or(ArithmeticError::Overflow)?;

			if prev_season_id > 0 {
				let prev_schedule = SeasonSchedules::<T>::get(prev_season_id)
					.ok_or(Error::<T>::NonSequentialSeasonId)?;
				ensure!(
					prev_schedule.end < season_schedule.early_start,
					Error::<T>::EarlyStartTooEarly
				);
			}
			if let Some(next_schedule) = SeasonSchedules::<T>::get(next_season_id) {
				ensure!(
					season_schedule.end < next_schedule.early_start,
					Error::<T>::SeasonEndTooLate
				);
			}*/

			Ok(())
		}
	}
}
