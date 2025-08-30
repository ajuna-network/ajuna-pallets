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
#[cfg(test)]
mod mock;
#[cfg(test)]
mod test_impls;
#[cfg(test)]
mod tests;

mod impls;
mod types;

use ajuna_primitives::{
	account_manager::AccountManager,
	season_manager::{SeasonConfig, SeasonManager},
};

use frame_support::{pallet_prelude::*, traits::Currency};
use frame_system::pallet_prelude::*;
use sp_runtime::{Saturating, traits::MaybeSerializeDeserialize};

pub use types::*;
use weights::WeightInfo;

const LOG_TARGET: &str = "runtime::ajuna-seasons";

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	#[pallet::pallet]
	pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

	pub type AccountIdOf<T> = <T as frame_system::Config>::AccountId;
	pub type SeasonIdOf<T, I> = <T as Config<I>>::SeasonId;
	pub type AssetIdOf<T, I> = <T as Config<I>>::AssetId;

	pub(crate) type SeasonStatusOf<T, I> = SeasonStatus<SeasonIdOf<T, I>>;
	pub(crate) type SeasonConfigOf<T, I> = SeasonConfig<BalanceOf<T, I>>;
	pub(crate) type SeasonScheduleOf<T> = SeasonSchedule<BlockNumberFor<T>>;
	pub(crate) type SeasonScheduledActionOf<T, I> =
		SeasonScheduledAction<<T as Config<I>>::SeasonId>;

	pub type BalanceOf<T, I> = <<T as Config<I>>::Currency as Currency<AccountIdOf<T>>>::Balance;

	#[cfg(feature = "runtime-benchmarks")]
	pub trait BenchmarkHelper<SeasonId> {
		fn create_season_id(id: u32) -> SeasonId;
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl<SeasonId: From<u32>> BenchmarkHelper<SeasonId> for () {
		fn create_season_id(id: u32) -> SeasonId {
			id.into()
		}
	}

	#[pallet::genesis_config]
	#[derive(frame_support::DefaultNoBound)]
	pub struct GenesisConfig<T: Config<I>, I: 'static = ()> {
		/// Genesis initial season + duration in blocks
		pub season: Option<(SeasonIdOf<T, I>, BlockNumberFor<T>)>,
	}
	#[pallet::genesis_build]
	impl<T: Config<I>, I: 'static> BuildGenesisConfig for GenesisConfig<T, I> {
		fn build(&self) {
			if let Some((ref season_id, ref duration)) = self.season {
				CurrentSeasonStatus::<T, I>::set(Ok(SeasonStatus {
					season_id: season_id.clone(),
					early: true,
					active: true,
					early_ended: false,
				}));

				Seasons::<T, I>::insert(
					season_id,
					SeasonConfig {
						fee: ajuna_primitives::season_manager::SeasonFeeConfig {
							transfer_asset: 100_u32.into(),
							buy_asset_min: 100_u32.into(),
							buy_percent: 10_u8,
							upgrade_asset_inventory: 100_u32.into(),
							unlock_trade_asset: 100_u32.into(),
							unlock_transfer_asset: 100_u32.into(),
							state_transition_base_fee: 100_u32.into(),
						},
					},
				);

				let early_start: BlockNumberFor<T> = 0_u32.into();
				let start: BlockNumberFor<T> = 5_u32.into();
				let end = duration.saturating_add(5_u32.into());
				SeasonSchedules::<T, I>::insert(
					season_id,
					SeasonSchedule { early_start, start, end: Some(end) },
				);

				SeasonScheduledActions::<T, I>::insert(
					early_start,
					SeasonScheduledAction::EarlyStart(season_id.clone()),
				);
				SeasonScheduledActions::<T, I>::insert(
					start,
					SeasonScheduledAction::Start(season_id.clone()),
				);
				SeasonScheduledActions::<T, I>::insert(
					end,
					SeasonScheduledAction::End(season_id.clone()),
				);
			}
		}
	}

	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config {
		type SeasonId: Member + Parameter + MaxEncodedLen + MaybeSerializeDeserialize;

		type AssetId: Member + Parameter + MaxEncodedLen + TypeInfo;

		type AccountHandler: AccountManager<AccountId = AccountIdOf<Self>>;

		type Currency: Currency<AccountIdOf<Self>>;

		type WeightInfo: WeightInfo;

		#[cfg(feature = "runtime-benchmarks")]
		type BenchmarkHelper: BenchmarkHelper<SeasonIdOf<Self, I>>;
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

	#[pallet::storage]
	pub type SeasonScheduledActions<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, BlockNumberFor<T>, SeasonScheduledActionOf<T, I>, OptionQuery>;

	/// Stores the assets season id registration.
	#[pallet::storage]
	pub type AssetSeasonRegister<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, AssetIdOf<T, I>, SeasonIdOf<T, I>, OptionQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		/// The season configuration for 'season_id' has been updated.
		UpdatedSeason {
			season_id: SeasonIdOf<T, I>,
			config: Option<SeasonConfigOf<T, I>>,
			metadata: Option<SeasonMetadata>,
			schedule: Option<SeasonScheduleOf<T>>,
		},
		/// The season was started during its 'early_start' by the scheduler.
		SeasonEarlyStarted { season_id: SeasonIdOf<T, I> },
		/// The season was started during its 'start' by the scheduler.
		SeasonStarted { season_id: SeasonIdOf<T, I> },
		/// The season was ended during its 'end' by the scheduler.
		SeasonEnded { season_id: SeasonIdOf<T, I> },
		/// The season was ended prematurely by the organizer.
		SeasonEarlyEnded { season_id: SeasonIdOf<T, I> },
	}

	#[pallet::error]
	#[derive(PartialEq)]
	pub enum Error<T, I = ()> {
		/// There is currently no active season
		NoActiveSeason,
		/// Cannot set season schedule without season config first.
		CannotScheduleSeasonWithoutConfig,
		/// The previous season has no end, making it so that no
		/// new seasons can be added after it.
		CannotScheduleSeasonIfPreviousSeasonIsInfinite,
		/// Cannot modify a season to be infinite if a season after it has already
		/// been scheduled.
		CannotScheduleInfiniteSeasonIfNextSeasonExists,
		/// The season's early start is before the current block.
		SeasonStartBeforeCurrentBlock,
		/// The season starts before the previous season starts.
		SeasonStartOverlapsPreviousSeason,
		/// The season starts after the next season starts.
		SeasonStartOverlapsNextSeason,
		/// The season's early start is earlier than its normal start.
		SeasonStartBeforeEarlyStart,
		/// The season's start block is greater than its end block.
		SeasonEndBeforeStart,
		/// The given asset was not registered in any season.
		AssetNotRegistered,
		/// The given season identifier has not been registered.
		InvalidSeason,
		/// The given season schedule update clashed with another season's schedule.
		ScheduleSlotAlreadyInUse,
	}

	#[pallet::hooks]
	impl<T: Config<I>, I: 'static> Hooks<BlockNumberFor<T>> for Pallet<T, I> {
		fn on_initialize(now: BlockNumberFor<T>) -> Weight {
			let mut weight = T::DbWeight::get().reads(1);

			if let Some(action) = SeasonScheduledActions::<T, I>::take(now) {
				let w = match action {
					SeasonScheduledAction::EarlyStart(season_id) =>
						CurrentSeasonStatus::<T, I>::mutate(|maybe_season_status| {
							if let Ok(current_season) = maybe_season_status {
								if !current_season.active {
									*current_season = SeasonStatus {
										season_id: season_id.clone(),
										early: true,
										active: true,
										early_ended: false,
									};
									Self::deposit_event(Event::SeasonEarlyStarted { season_id });
									T::DbWeight::get().reads_writes(1, 1)
								} else {
									log::info!(target: LOG_TARGET,
										"Next season [{:?}] could not be early started, previous season [{:?}] still active.",
										season_id,
										current_season.season_id
									);
									T::DbWeight::get().reads(1)
								}
							} else {
								*maybe_season_status = Ok(SeasonStatus {
									season_id: season_id.clone(),
									early: true,
									active: true,
									early_ended: false,
								});
								Self::deposit_event(Event::SeasonEarlyStarted { season_id });
								T::DbWeight::get().reads_writes(1, 1)
							}
						}),
					SeasonScheduledAction::Start(season_id) =>
						CurrentSeasonStatus::<T, I>::mutate(|maybe_season_status| {
							if let Ok(current_season) = maybe_season_status {
								if !current_season.active {
									*current_season = SeasonStatus {
										season_id: season_id.clone(),
										early: false,
										active: true,
										early_ended: false,
									};
									Self::deposit_event(Event::SeasonStarted { season_id });
									T::DbWeight::get().reads_writes(1, 1)
								} else if current_season.season_id != season_id {
									log::error!(target: LOG_TARGET,
										"Next season [{:?}] could not be started, previous season [{:?}] still active.",
										season_id,
										current_season.season_id
									);
									T::DbWeight::get().reads(1)
								} else {
									log::info!(target: LOG_TARGET,
										"Season [{:?}] start skipped, season already early started.",
										season_id,
									);
									T::DbWeight::get().reads(1)
								}
							} else {
								log::warn!(target: LOG_TARGET,
									"Next season [{:?}] started, no previous season found, so it should have been early started.",
									season_id,
								);
								*maybe_season_status = Ok(SeasonStatus {
									season_id: season_id.clone(),
									early: false,
									active: true,
									early_ended: false,
								});
								Self::deposit_event(Event::SeasonStarted { season_id });
								T::DbWeight::get().reads_writes(1, 1)
							}
						}),
					SeasonScheduledAction::End(season_id) =>
						CurrentSeasonStatus::<T, I>::mutate(|maybe_current_status| {
							if let Ok(current_status) = maybe_current_status {
								current_status.active = false;
								FinishedSeasons::<T, I>::insert(&season_id, ());
								Self::deposit_event(Event::SeasonEnded { season_id });
								T::DbWeight::get().reads_writes(1, 2)
							} else {
								log::error!(target: LOG_TARGET,
									"CurrentSeasonStatus didn't contain the expected entry for season [{:?}].",
									season_id
								);
								T::DbWeight::get().reads(1)
							}
						}),
				};
				weight.saturating_accrue(w);
			};

			weight
		}
	}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::update_season())]
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
					Seasons::<T, I>::insert(&season_id, config_update);
				}

				if !PrevSeasonChain::<T, I>::contains_key(&season_id) {
					// If the 'season_id' key is not already in 'PrevSeasonChain',
					// then that means that this is the first time we call this method with
					// 'season_id', in that case we build the chain link for the given
					// 'season_id'. Once the link has been built all subsequent calls will never
					// modify it.
					Self::insert_season_chains(&season_id);
				}

				if let Some(ref schedule_update) = schedule {
					ensure!(
						Seasons::<T, I>::contains_key(&season_id),
						Error::<T, I>::CannotScheduleSeasonWithoutConfig
					);

					Self::ensure_valid_schedule(&season_id, schedule_update)?;
					Self::update_season_scheduled_actions(&season_id, schedule_update)?;
				}
			} else if Self::is_active_season_and_infinite(&season_id) {
				// If the season we want to update is the current active season,
				// and it has infinite duration (no end block).
				// We only allow to add an end ot it once.
				if let Some(SeasonSchedule { end: Some(end), .. }) = schedule {
					Self::add_season_end_action_at(&season_id, end)?;
					SeasonSchedules::<T, I>::mutate(&season_id, |maybe_schedule| {
						if let Some(schedule) = maybe_schedule {
							schedule.end = Some(end);
						} else {
							log::error!(target: LOG_TARGET,
								"Updating schedule for active infinite season [{:?}], found no schedule data!",
								&season_id,
							);
						}
					});
				}
			}

			if let Some(ref meta_update) = metadata {
				SeasonMetadatas::<T, I>::insert(&season_id, meta_update);
			}

			Self::deposit_event(Event::UpdatedSeason { season_id, config, metadata, schedule });

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::interrupt_active_season())]
		pub fn interrupt_active_season(origin: OriginFor<T>) -> DispatchResult {
			let account = ensure_signed(origin)?;
			T::AccountHandler::is_organizer(&account)?;

			CurrentSeasonStatus::<T, I>::try_mutate(|maybe_current_season| {
				if let Ok(current_season) = maybe_current_season {
					current_season.active = false;
					current_season.early_ended = true;

					FinishedSeasons::<T, I>::insert(&current_season.season_id, ());

					let season_schedule = SeasonSchedules::<T, I>::get(&current_season.season_id)
						.ok_or(Error::<T, I>::InvalidSeason)?;
					SeasonScheduledActions::<T, I>::remove(season_schedule.early_start);
					SeasonScheduledActions::<T, I>::remove(season_schedule.start);

					if let Some(ref end) = season_schedule.end {
						SeasonScheduledActions::<T, I>::remove(end);
					}

					// TODO: Maybe we could try to early start the next season if
					// it exists and its early start was skipped

					Self::deposit_event(Event::SeasonEarlyEnded {
						season_id: current_season.season_id.clone(),
					});

					Ok(())
				} else {
					Err(Error::<T, I>::NoActiveSeason.into())
				}
			})
		}
	}

	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		fn insert_season_chains(new_season_id: &SeasonIdOf<T, I>) {
			LatestSeason::<T, I>::mutate(|maybe_prev_season_id| {
				match maybe_prev_season_id {
					Some(prev_season_id) if prev_season_id != new_season_id => {
						// If there was a previous season we can build the chain links
						PrevSeasonChain::<T, I>::insert(new_season_id, &*prev_season_id);
						NextSeasonChain::<T, I>::insert(&*prev_season_id, new_season_id);
						*prev_season_id = new_season_id.clone();
					},
					_ => {
						// If this is the first season ever created
						// we won't build the chain links
						// since we are missing another season to do so
						*maybe_prev_season_id = Some(new_season_id.clone());
					},
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

		fn is_active_season_and_infinite(season_id: &SeasonIdOf<T, I>) -> bool {
			if let Ok(current_season) = CurrentSeasonStatus::<T, I>::get() {
				if &current_season.season_id == season_id {
					if let Some(schedule) = SeasonSchedules::<T, I>::get(season_id) {
						return schedule.end.is_none();
					}
				}
			}

			false
		}

		fn ensure_valid_schedule(
			season_id: &SeasonIdOf<T, I>,
			schedule: &SeasonScheduleOf<T>,
		) -> DispatchResult {
			let current_block = <frame_system::Pallet<T>>::block_number();
			ensure!(
				current_block < schedule.early_start,
				Error::<T, I>::SeasonStartBeforeCurrentBlock
			);
			ensure!(
				schedule.early_start < schedule.start,
				Error::<T, I>::SeasonStartBeforeEarlyStart
			);

			if let Some(ref end) = schedule.end {
				ensure!(schedule.start < *end, Error::<T, I>::SeasonEndBeforeStart);
			}

			if let Some(prev_season_id) = PrevSeasonChain::<T, I>::get(season_id) {
				if let Some(prev_schedule) = SeasonSchedules::<T, I>::get(prev_season_id) {
					ensure!(
						prev_schedule.start < schedule.early_start,
						Error::<T, I>::SeasonStartOverlapsPreviousSeason
					);

					if prev_schedule.end.is_none() {
						return Err(
							Error::<T, I>::CannotScheduleSeasonIfPreviousSeasonIsInfinite.into()
						);
					}
				}
			}

			if let Some(next_season_id) = NextSeasonChain::<T, I>::get(season_id) {
				if let Some(next_schedule) = SeasonSchedules::<T, I>::get(next_season_id) {
					ensure!(
						schedule.start < next_schedule.early_start,
						Error::<T, I>::SeasonStartOverlapsNextSeason
					);

					if schedule.end.is_none() {
						return Err(
							Error::<T, I>::CannotScheduleInfiniteSeasonIfNextSeasonExists.into()
						);
					}
				}
			}

			Ok(())
		}

		fn update_season_scheduled_actions(
			season_id: &SeasonIdOf<T, I>,
			schedule: &SeasonScheduleOf<T>,
		) -> DispatchResult {
			SeasonSchedules::<T, I>::try_mutate(season_id, |maybe_prev_schedule| {
				if let Some(prev_schedule) = maybe_prev_schedule {
					SeasonScheduledActions::<T, I>::remove(prev_schedule.early_start);
					SeasonScheduledActions::<T, I>::remove(prev_schedule.start);
					if let Some(ref prev_end) = prev_schedule.end {
						SeasonScheduledActions::<T, I>::remove(prev_end);
					}
				}

				SeasonScheduledActions::<T, I>::try_mutate(schedule.early_start, |maybe_action| {
					ensure!(maybe_action.is_none(), Error::<T, I>::ScheduleSlotAlreadyInUse);

					*maybe_action = Some(SeasonScheduledAction::EarlyStart(season_id.clone()));

					Ok::<(), DispatchError>(())
				})?;
				SeasonScheduledActions::<T, I>::try_mutate(schedule.start, |maybe_action| {
					ensure!(maybe_action.is_none(), Error::<T, I>::ScheduleSlotAlreadyInUse);

					*maybe_action = Some(SeasonScheduledAction::Start(season_id.clone()));

					Ok::<(), DispatchError>(())
				})?;
				if let Some(end) = schedule.end {
					Self::add_season_end_action_at(season_id, end)?;
				}

				*maybe_prev_schedule = Some(schedule.clone());

				Ok::<(), DispatchError>(())
			})
		}

		fn add_season_end_action_at(
			season_id: &SeasonIdOf<T, I>,
			end: BlockNumberFor<T>,
		) -> DispatchResult {
			SeasonScheduledActions::<T, I>::try_mutate(end, |maybe_action| {
				ensure!(maybe_action.is_none(), Error::<T, I>::ScheduleSlotAlreadyInUse);
				*maybe_action = Some(SeasonScheduledAction::End(season_id.clone()));
				Ok::<(), DispatchError>(())
			})?;

			Ok(())
		}
	}
}
