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
pub use types::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;
mod types;

#[frame_support::pallet]
pub mod pallet {
	use super::*;

	use core::num::NonZeroU32;
	use frame_support::{pallet_prelude::*, Hashable};
	use frame_system::pallet_prelude::*;
	use sp_runtime::{SaturatedConversion, Saturating};
	use sp_std::prelude::*;

	pub(crate) type AccountIdFor<T> = <T as frame_system::Config>::AccountId;

	pub(crate) type BattleConfigFor<T> = BattleConfig<BlockNumberFor<T>>;
	pub(crate) type BattleStateFor<T> = BattleState<BlockNumberFor<T>>;

	pub(crate) type OccupancyStateFor<T> = OccupancyState<AccountIdFor<T>>;

	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self, I>>
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;

		/// The duration of the Input phase
		#[pallet::constant]
		type InputPhaseDuration: Get<NonZeroU32>;

		/// The duration of the Reveal phase
		#[pallet::constant]
		type RevealPhaseDuration: Get<NonZeroU32>;

		/// The duration of the Execution phase
		#[pallet::constant]
		type ExecutionPhaseDuration: Get<NonZeroU32>;

		/// The duration of the Shrink phase
		#[pallet::constant]
		type ShrinkPhaseDuration: Get<NonZeroU32>;

		/// The duration of the Verification phase
		#[pallet::constant]
		type VerificationPhaseDuration: Get<NonZeroU32>;

		/// The duration of the Idle phase
		#[pallet::constant]
		type IdlePhaseDuration: Get<NonZeroU32>;
	}

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

	#[pallet::storage]
	pub type BattleSchedules<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, BlockNumberFor<T>, SchedulerAction, OptionQuery>;

	#[pallet::storage]
	pub type BattleStateStore<T: Config<I>, I: 'static = ()> =
		StorageValue<_, BattleStateFor<T>, ValueQuery>;

	#[pallet::storage]
	pub type PlayerDetails<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, AccountIdFor<T>, PlayerData, OptionQuery>;

	#[pallet::storage]
	pub type GridOccupancy<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, Coordinates, OccupancyStateFor<T>, ValueQuery>;

	#[pallet::storage]
	pub(crate) type GridOccupancyBitmap<T: Config<I>, I: 'static = ()> =
		StorageValue<_, GridBits, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		BattleStarted,
		BattleFinished,
		BattlePhaseChanged { phase: BattlePhase },
		PlayerQueued { player: AccountIdFor<T> },
		PlayerPerformedAction { player: AccountIdFor<T> },
		PlayerRevealedAction { player: AccountIdFor<T> },
		PlayerDefeated { player: AccountIdFor<T> },
	}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		BattleAlreadyStarted,
		BattleConfigDurationTooLow,
		BattleConfigTooFewPlayers,
		BattleConfigTooManyPlayers,
		BattleConfigGridSizeInvalid,
		PlayerAlreadyQueued,
		PlayerQueueFull,
		CouldNotQueuePlayer,
		BattleNotInQueueingPhase,
		BattleNotInInputPhase,
		BattleNotInPlayablePhases,
		BattleNotInFinishedPhase,
		BattleIsInactive,
		PlayerNotFound,
		PlayerCannotPerformAction,
		PlayerActionCouldNotBeDecoded,
		PlayerRevealDoesntMatchOriginalAction,
		PlayerDoesntHaveOriginalActionToReveal,
		PlayerDefeated,
	}

	#[pallet::hooks]
	impl<T: Config<I>, I: 'static> Hooks<BlockNumberFor<T>> for Pallet<T, I> {
		fn on_initialize(now: BlockNumberFor<T>) -> Weight {
			let weight = T::DbWeight::get().reads(1);

			let computed_weight = if let Some(scheduled_action) = BattleSchedules::<T, I>::take(now)
			{
				match scheduled_action {
					SchedulerAction::Input => Self::switch_to_phase(BattlePhase::Input),
					SchedulerAction::Reveal => Self::switch_to_phase(BattlePhase::Reveal),
					SchedulerAction::Execution =>
						Self::switch_to_phase(BattlePhase::Execution) + Self::resolve_battles(),
					SchedulerAction::Shrink =>
						Self::switch_to_phase(BattlePhase::Shrink) + Self::resolve_wall_shrinkage(),
					SchedulerAction::Verify(count) =>
						Self::switch_to_phase(BattlePhase::Verification) +
							Self::resolve_game_state(now) +
							Self::insert_next_schedule(now, count),
					SchedulerAction::Idle => Self::switch_to_phase(BattlePhase::Idle),
				}
			} else {
				Weight::from_parts(0, 0)
			};

			weight.saturating_add(computed_weight)
		}
	}

	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		pub(crate) fn switch_to_phase(new_phase: BattlePhase) -> Weight {
			BattleStateStore::<T, I>::mutate(|state| {
				state.switch_to_phase(new_phase);
			});

			Self::deposit_event(Event::<T, I>::BattlePhaseChanged { phase: new_phase });

			T::DbWeight::get().reads_writes(1, 1)
		}

		pub(crate) fn insert_next_schedule(now: BlockNumberFor<T>, phase_count: u8) -> Weight {
			if matches!(
				BattleStateStore::<T, I>::get(),
				BattleStateFor::<T>::Active { phase: BattlePhase::Finished, .. }
			) {
				return T::DbWeight::get().reads(1)
			}

			let shrink_frequency = if let BattleStateFor::<T>::Active {
				config: BattleConfigFor::<T> { shrink_frequency, .. },
				..
			} = BattleStateStore::<T, I>::get()
			{
				shrink_frequency
			} else {
				DEFAULT_SHRINK_PHASE_FREQUENCY
			};

			let verification_phase_duration = u32::from(T::VerificationPhaseDuration::get());
			let mut time = now.saturating_add(verification_phase_duration.into());
			BattleSchedules::<T, I>::insert(time, SchedulerAction::Idle);

			let idle_phase_duration = u32::from(T::IdlePhaseDuration::get());
			time = time.saturating_add(idle_phase_duration.into());
			BattleSchedules::<T, I>::insert(time, SchedulerAction::Input);

			let input_phase_duration = u32::from(T::InputPhaseDuration::get());
			time = time.saturating_add(input_phase_duration.into());
			BattleSchedules::<T, I>::insert(time, SchedulerAction::Reveal);

			let reveal_phase_duration = u32::from(T::RevealPhaseDuration::get());
			time = time.saturating_add(reveal_phase_duration.into());
			BattleSchedules::<T, I>::insert(time, SchedulerAction::Execution);

			let execution_phase_duration = u32::from(T::ExecutionPhaseDuration::get());
			let new_phase_count = {
				let new_phase_count = phase_count.saturating_add(1);

				if new_phase_count > shrink_frequency {
					0
				} else {
					new_phase_count
				}
			};
			let should_shrink = phase_count == shrink_frequency;

			if should_shrink {
				time = time.saturating_add(execution_phase_duration.into());
				BattleSchedules::<T, I>::insert(time, SchedulerAction::Shrink);

				let shrink_phase_duration = u32::from(T::ShrinkPhaseDuration::get());
				time = time.saturating_add(shrink_phase_duration.into());
				BattleSchedules::<T, I>::insert(time, SchedulerAction::Verify(new_phase_count));
			} else {
				time = time.saturating_add(execution_phase_duration.into());
				BattleSchedules::<T, I>::insert(time, SchedulerAction::Verify(new_phase_count));
			}

			if should_shrink {
				T::DbWeight::get().reads_writes(1, 5)
			} else {
				T::DbWeight::get().reads_writes(1, 4)
			}
		}

		pub(crate) fn mark_account_as_defeated(account: &AccountIdFor<T>) -> (u64, u64) {
			let mut r = 0;
			let mut w = 0;

			PlayerDetails::<T, I>::mutate(account, |maybe_details| {
				if let Some(details) = maybe_details {
					if !matches!(details.state, PlayerState::Defeated) {
						details.state = PlayerState::Defeated;

						GridOccupancy::<T, I>::mutate(details.position, |occupancy_state| {
							if let OccupancyStateFor::<T>::Open(account_set) = occupancy_state {
								let prev_length = account_set.len();
								account_set.remove(account);
								let new_length = account_set.len();

								if prev_length > 0 && new_length == 0 {
									GridOccupancyBitmap::<T, I>::mutate(|bitmap| {
										bitmap.mark_player_cell_as(&details.position, false);
									});
								}
							} else {
								panic!("Tried to remove account from Blocked position!")
							}
						});

						r = 2;
						w = 2;

						Self::deposit_event(Event::<T, I>::PlayerDefeated {
							player: account.clone(),
						})
					} else {
						r = 1;
					}
				} else {
					panic!("Expected account to mark as defeated to exist!")
				}
			});

			(r, w)
		}

		fn reset_player_states() -> (u64, u64) {
			let non_defeated_accounts = PlayerDetails::<T, I>::iter()
				.filter(|(_, details)| details.state != PlayerState::Defeated)
				.map(|(account_id, _)| account_id)
				.collect::<Vec<_>>();

			for account_id in non_defeated_accounts.iter() {
				PlayerDetails::<T, I>::mutate(account_id, |maybe_details| {
					if let Some(details) = maybe_details {
						details.state = PlayerState::Inactive;
					} else {
						panic!("Expected account to reset state to exist!")
					}
				});
			}

			let affected_accounts = non_defeated_accounts.len() as u64;

			(affected_accounts, affected_accounts)
		}

		fn resolve_battles() -> Weight {
			let mut r: u64 = 0;
			let mut w: u64 = 0;

			let battle_cells = GridOccupancyBitmap::<T, I>::get()
				.get_occupied_cells()
				.into_iter()
				.filter_map(|cell| match GridOccupancy::<T, I>::get(cell) {
					OccupancyStateFor::<T>::Open(account_set) if account_set.len() > 1 =>
						Some(account_set.into_iter().collect::<Vec<_>>()),
					_ => None,
				})
				.collect::<Vec<_>>();

			let boundaries = match BattleStateStore::<T, I>::get() {
				BattleStateFor::<T>::Inactive =>
					panic!("Resolve battle called with Inactive BattleState"),
				BattleStateFor::<T>::Active { boundaries, .. } => boundaries,
			};
			let current_block = <frame_system::Pallet<T>>::block_number();

			r = r.saturating_add(battle_cells.len() as u64 + 1);

			for mut accounts in battle_cells.into_iter() {
				r = r.saturating_add(accounts.len() as u64);

				while accounts.len() > 1 {
					let acc1 = accounts.pop().expect("Obtained AccountId from stack!");
					let acc2 = accounts.pop().expect("Obtained AccountId from stack!");

					let acc1_data = PlayerDetails::<T, I>::get(&acc1)
						.expect("Obtained Account data from stack!");
					let acc2_data = PlayerDetails::<T, I>::get(&acc2)
						.expect("Obtained Account data from stack!");

					r = r.saturating_add(2);

					if acc1_data.state != PlayerState::RevealedAction {
						let (r1, w1) = Self::mark_account_as_defeated(&acc1);
						r = r.saturating_add(r1);
						w = w.saturating_add(w1);
						accounts.push(acc2);
					} else if acc2_data.state != PlayerState::RevealedAction {
						let (r1, w1) = Self::mark_account_as_defeated(&acc2);
						r = r.saturating_add(r1);
						w = w.saturating_add(w1);
						accounts.push(acc1);
					} else {
						let acc1_weapon = acc1_data.weapon;
						let acc2_weapon = acc2_data.weapon;

						match acc1_weapon.battle_against(acc2_weapon) {
							BattleResult::Win => {
								let (r1, w1) = Self::mark_account_as_defeated(&acc2);
								r = r.saturating_add(r1);
								w = w.saturating_add(w1);
								accounts.push(acc1);
							},
							BattleResult::Loss => {
								let (r1, w1) = Self::mark_account_as_defeated(&acc1);
								r = r.saturating_add(r1);
								w = w.saturating_add(w1);
								accounts.push(acc2);
							},
							BattleResult::Draw => {
								if let Some(cell) = Self::try_get_random_unoccupied_cell_for(
									&acc1,
									&boundaries,
									current_block,
								) {
									Self::update_player_positions_storage(
										&acc1,
										acc1_data.position,
										cell,
									);
								}

								if let Some(cell) = Self::try_get_random_unoccupied_cell_for(
									&acc2,
									&boundaries,
									current_block,
								) {
									Self::update_player_positions_storage(
										&acc2,
										acc2_data.position,
										cell,
									);
								}

								r = r.saturating_add(4);
								w = w.saturating_add(2);
							},
						}
					};
				}
			}

			T::DbWeight::get().reads_writes(r, w)
		}

		fn resolve_wall_shrinkage() -> Weight {
			let mut r = 0;
			let mut w = 0;

			BattleStateStore::<T, I>::mutate(|state| {
				r = r.saturating_add(1);

				if let BattleStateFor::<T>::Active { boundaries, .. } = state {
					boundaries.shrink();

					GridOccupancyBitmap::<T, I>::mutate(|bitmap| {
						for cell in bitmap
							.get_occupied_cells()
							.iter()
							.filter(|cell| !boundaries.is_in_boundaries(cell))
						{
							GridOccupancy::<T, I>::mutate(cell, |state| {
								if let OccupancyStateFor::<T>::Open(account_set) = state {
									for account in account_set.iter() {
										PlayerDetails::<T, I>::mutate(account, |maybe_details| {
											if let Some(details) = maybe_details {
												if !matches!(details.state, PlayerState::Defeated) {
													details.state = PlayerState::Defeated;
												}

												Self::deposit_event(
													Event::<T, I>::PlayerDefeated {
														player: account.clone(),
													},
												);

												r = r.saturating_add(1);
												w = w.saturating_add(1);
											}
										});
									}
								}

								r = r.saturating_add(1);
								w = w.saturating_add(1);

								*state = OccupancyStateFor::<T>::Blocked;
							});

							bitmap.mark_player_cell_as(cell, false);
							bitmap.mark_blocked_cell_as(cell, true);

							r = r.saturating_add(1);
							w = w.saturating_add(1);
						}
					});
				} else {
					panic!("Expected Active battle state for wall shrinkage phase!");
				}
			});

			T::DbWeight::get().reads_writes(r, w)
		}

		fn resolve_game_state(now: BlockNumberFor<T>) -> Weight {
			let mut r: u64 = 0;
			let mut w: u64 = 0;

			BattleStateStore::<T, I>::mutate(|state| {
				r = r.saturating_add(1);

				if let BattleStateFor::<T>::Active { ref config, phase, .. } = state {
					if config.run_until <= now {
						// The game finishes
						*phase = BattlePhase::Finished;

						w = w.saturating_add(
							1 + BattleSchedules::<T, I>::clear(10, None).unique as u64,
						);

						Self::deposit_event(Event::<T, I>::BattleFinished);
					} else {
						let player_count = GridOccupancy::<T, I>::iter_values().fold(
							0_usize,
							|acc, occupancy_state| {
								let count = match occupancy_state {
									OccupancyStateFor::<T>::Blocked => 0,
									OccupancyStateFor::<T>::Open(account_set) => account_set.len(),
								};

								acc.saturating_add(count)
							},
						);

						r = r.saturating_add(player_count as u64);

						if player_count < 2 {
							// Game finishes
							*phase = BattlePhase::Finished;

							w = w.saturating_add(
								1 + BattleSchedules::<T, I>::clear(10, None).unique as u64,
							);

							Self::deposit_event(Event::<T, I>::BattleFinished);
						}
					}
				} else {
					panic!("Expected Active battle state for wall shrinkage phase!");
				}
			});

			let (r1, w1) = Self::reset_player_states();
			r = r.saturating_add(r1);
			w = w.saturating_add(w1);

			T::DbWeight::get().reads_writes(r, w)
		}

		fn reset_state_for_new_battle() {
			let _ = BattleSchedules::<T, I>::clear(20, None);
			BattleStateStore::<T, I>::kill();
			let _ = PlayerDetails::<T, I>::clear(MAX_PLAYER_PER_BATTLE as u32, None);

			// TODO: Check if we can split this between blocks (maybe using cursor during idle time)
			let _ = GridOccupancy::<T, I>::clear(MAX_GRID_SIZE as u32 * MAX_GRID_SIZE as u32, None);
			GridOccupancyBitmap::<T, I>::kill();
		}

		fn update_player_data_using_action(
			account: &AccountIdFor<T>,
			player_data: &mut PlayerData,
			action: PlayerAction,
			boundaries: GridBoundaries,
		) {
			match action {
				PlayerAction::Move(new_position) =>
					if boundaries.is_in_boundaries(&new_position) {
						Self::update_player_positions_storage(
							account,
							player_data.position,
							new_position,
						);
						player_data.position = new_position;
					},
				PlayerAction::SwapWeapon(new_weapon) => {
					player_data.weapon = new_weapon;
				},
				PlayerAction::MoveAndSwap(new_position, new_weapon) => {
					if boundaries.is_in_boundaries(&new_position) {
						Self::update_player_positions_storage(
							account,
							player_data.position,
							new_position,
						);
						player_data.position = new_position;
					}
					player_data.weapon = new_weapon;
				},
			}

			player_data.state = PlayerState::RevealedAction;
		}

		pub(crate) fn update_player_positions_storage(
			account: &AccountIdFor<T>,
			prev_position: Coordinates,
			new_position: Coordinates,
		) {
			GridOccupancy::<T, I>::mutate(new_position, |occupancy_state| {
				if let OccupancyStateFor::<T>::Open(account_set) = occupancy_state {
					GridOccupancy::<T, I>::mutate(prev_position, |occupancy_state| {
						if let OccupancyStateFor::<T>::Open(account_set) = occupancy_state {
							let prev_length = account_set.len();
							account_set.remove(account);
							let new_length = account_set.len();

							if prev_length > 0 && new_length == 0 {
								GridOccupancyBitmap::<T, I>::mutate(|bitmap| {
									bitmap.mark_player_cell_as(&prev_position, false);
								});
							}
						} else {
							panic!("Tried to remove account from Blocked position!")
						}
					});
					let prev_length = account_set.len();
					account_set.try_insert(account.clone()).expect("Inserted account into set");
					let new_length = account_set.len();

					if prev_length == 0 && new_length > 0 {
						GridOccupancyBitmap::<T, I>::mutate(|bitmap| {
							bitmap.mark_player_cell_as(&new_position, true);
						});
					}
				}
			});
		}

		#[inline]
		fn is_cell_unoccupied(coordinates: &Coordinates) -> bool {
			matches!(GridOccupancy::<T, I>::get(coordinates), OccupancyStateFor::<T>::Open(account_set) if account_set.is_empty())
		}

		fn try_get_random_unoccupied_cell_for(
			account: &AccountIdFor<T>,
			in_bounds: &GridBoundaries,
			for_block: BlockNumberFor<T>,
		) -> Option<Coordinates> {
			let hash = account.blake2_256();

			let mut block_seed: usize = for_block.saturated_into();
			let mut coordinate = in_bounds.random_coordinates_in(&hash, block_seed);

			if !Self::is_cell_unoccupied(&coordinate) {
				block_seed = block_seed.saturating_add(1);
				coordinate = in_bounds.random_coordinates_in(&hash, block_seed);

				if !Self::is_cell_unoccupied(&coordinate) {
					None
				} else {
					Some(coordinate)
				}
			} else {
				Some(coordinate)
			}
		}
	}

	impl<T: Config<I>, I: 'static> BattleProvider<AccountIdFor<T>> for Pallet<T, I> {
		fn try_start_battle(
			game_duration: u32,
			max_players: u8,
			grid_size: Coordinates,
			shrink_frequency: u8,
			shrink_sides: [bool; 4],
			blocked_cells: Vec<Coordinates>,
		) -> DispatchResult {
			BattleStateStore::<T, I>::try_mutate(|state| {
				ensure!(
					matches!(state, BattleStateFor::<T>::Inactive),
					Error::<T, I>::BattleAlreadyStarted
				);
				ensure!(
					game_duration >= MIN_BATTLE_DURATION,
					Error::<T, I>::BattleConfigDurationTooLow
				);
				ensure!(
					max_players >= MIN_PLAYER_PER_BATTLE,
					Error::<T, I>::BattleConfigTooFewPlayers
				);
				ensure!(
					max_players <= MAX_PLAYER_PER_BATTLE,
					Error::<T, I>::BattleConfigTooManyPlayers
				);
				ensure!(
					grid_size.x >= MIN_GRID_SIZE &&
						grid_size.x <= MAX_GRID_SIZE &&
						grid_size.y >= MIN_GRID_SIZE &&
						grid_size.y <= MAX_GRID_SIZE,
					Error::<T, I>::BattleConfigGridSizeInvalid
				);

				let current_block = <frame_system::Pallet<T>>::block_number();
				let total_duration = QUEUE_DURATION + game_duration;
				let run_until = current_block.saturating_add(total_duration.into());

				let maybe_shrink_boundaries = shrink_sides
					.iter()
					.any(|boundary| *boundary)
					.then(|| ShrinkBoundaries::new(shrink_sides));
				let boundaries = GridBoundaries::new(grid_size, maybe_shrink_boundaries);

				if !blocked_cells.is_empty() {
					GridOccupancyBitmap::<T, I>::mutate(|bitmap| {
						bitmap.mark_blocked_cells_as(blocked_cells.as_slice(), true);
					});

					for blocked_cell in
						blocked_cells.into_iter().filter(|cell| boundaries.is_in_boundaries(cell))
					{
						GridOccupancy::<T, I>::mutate(blocked_cell, |state| {
							*state = OccupancyState::Blocked;
						});
					}
				}

				*state = BattleStateFor::<T>::Active {
					phase: BattlePhase::Queueing,
					config: BattleConfigFor::<T> { max_players, shrink_frequency, run_until },
					boundaries,
				};

				let verification_phase_duration = u32::from(T::VerificationPhaseDuration::get());
				// We add the current block to the QUEUE_DURATION as baseline, then we decrease that
				// time so that the moment the Queueing phase is over we switch to Idle phase
				// immediately independently of how long is the phase configured
				let switch_at = current_block
					.saturating_add(QUEUE_DURATION.into())
					.saturating_sub(verification_phase_duration.into());
				let _ = Self::insert_next_schedule(switch_at, 0);

				Self::deposit_event(Event::<T, I>::BattleStarted);

				Ok(())
			})
		}

		fn try_finish_battle() -> Result<Vec<AccountIdFor<T>>, DispatchError> {
			BattleStateStore::<T, I>::try_mutate(|state| {
				ensure!(
					matches!(
						state,
						BattleStateFor::<T>::Active { phase: BattlePhase::Finished, .. }
					),
					Error::<T, I>::BattleNotInFinishedPhase
				);

				let defeated_players = PlayerDetails::<T, I>::iter()
					.filter(|(_, data)| data.state == PlayerState::Defeated)
					.map(|(account_id, _)| account_id.clone())
					.collect();

				Self::reset_state_for_new_battle();

				*state = BattleStateFor::<T>::Inactive;

				Ok(defeated_players)
			})
		}

		fn try_queue_player(
			account: &AccountIdFor<T>,
			initial_weapon: PlayerWeapon,
			initial_position: Option<Coordinates>,
		) -> sp_runtime::DispatchResult {
			if let BattleStateFor::<T>::Active {
				phase: BattlePhase::Queueing,
				config: BattleConfigFor::<T> { max_players, .. },
				boundaries,
			} = BattleStateStore::<T, I>::get()
			{
				ensure!(
					!PlayerDetails::<T, I>::contains_key(account),
					Error::<T, I>::PlayerAlreadyQueued
				);
				ensure!(
					PlayerDetails::<T, I>::iter().count() < max_players as usize,
					Error::<T, I>::PlayerQueueFull
				);

				let current_block = <frame_system::Pallet<T>>::block_number();
				let initial_position = {
					if let Some(position) = initial_position {
						position
					} else if let Some(position) = Self::try_get_random_unoccupied_cell_for(
						account,
						&boundaries,
						current_block,
					) {
						position
					} else {
						return Err(Error::<T, I>::CouldNotQueuePlayer.into())
					}
				};

				let player_data = PlayerData {
					position: initial_position,
					weapon: initial_weapon,
					state: PlayerState::Inactive,
				};

				// Setting player initial position
				GridOccupancy::<T, I>::mutate(initial_position, |occupancy_state| {
					if let OccupancyStateFor::<T>::Open(account_set) = occupancy_state {
						account_set
							.try_insert(account.clone())
							.map_err(|_| Error::<T, I>::CouldNotQueuePlayer)
					} else {
						Err(Error::<T, I>::CouldNotQueuePlayer)
					}
				})?;
				// Marking cell bit as occupied by a player
				GridOccupancyBitmap::<T, I>::mutate(|bitmap| {
					bitmap.mark_player_cell_as(&initial_position, true);
				});

				PlayerDetails::<T, I>::insert(account, player_data);

				Self::deposit_event(Event::<T, I>::PlayerQueued { player: account.clone() });

				Ok(())
			} else {
				Err(Error::<T, I>::BattleNotInQueueingPhase.into())
			}
		}

		fn try_perform_player_action(
			account: &AccountIdFor<T>,
			action: PlayerActionHash,
		) -> DispatchResult {
			match BattleStateStore::<T, I>::get() {
				BattleStateFor::<T>::Inactive => Err(Error::<T, I>::BattleIsInactive.into()),
				BattleStateFor::<T>::Active { phase, boundaries, .. } => match phase {
					BattlePhase::Input => PlayerDetails::<T, I>::try_mutate(account, |details| {
						if let Some(player_data) = details {
							if matches!(
								player_data.state,
								PlayerState::Inactive | PlayerState::PerformedAction(_)
							) {
								player_data.state = PlayerState::PerformedAction(action);

								Self::deposit_event(Event::<T, I>::PlayerPerformedAction {
									player: account.clone(),
								});

								Ok(())
							} else {
								Err(Error::<T, I>::PlayerCannotPerformAction.into())
							}
						} else {
							Err(Error::<T, I>::PlayerNotFound.into())
						}
					}),
					BattlePhase::Reveal => PlayerDetails::<T, I>::try_mutate(account, |details| {
						if let Some(player_data) = details {
							if let PlayerState::PerformedAction(player_action_hash) =
								&player_data.state
							{
								let reveal_action_hash = sp_crypto_hashing::blake2_256(&action);
								if reveal_action_hash == *player_action_hash {
									let action_payload = {
										let mut payload = [0_u8; 4];
										payload.copy_from_slice(&action[28..=31]);
										payload
									};
									let revealed_action =
										PlayerAction::try_decode_from_payload(action_payload)
											.ok_or(Error::<T, I>::PlayerActionCouldNotBeDecoded)?;

									Self::update_player_data_using_action(
										account,
										player_data,
										revealed_action,
										boundaries,
									);

									Self::deposit_event(Event::<T, I>::PlayerRevealedAction {
										player: account.clone(),
									});

									Ok(())
								} else {
									Err(Error::<T, I>::PlayerRevealDoesntMatchOriginalAction.into())
								}
							} else {
								Err(Error::<T, I>::PlayerDoesntHaveOriginalActionToReveal.into())
							}
						} else {
							Err(Error::<T, I>::PlayerNotFound.into())
						}
					}),
					_ => Err(Error::<T, I>::BattleNotInPlayablePhases.into()),
				},
			}
		}
	}
}
