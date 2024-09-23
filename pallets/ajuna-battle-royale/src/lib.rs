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
	use crate::types::*;
	use frame_support::pallet_prelude::*;
	use frame_system::pallet_prelude::*;
	use sp_runtime::Saturating;
	use sp_std::prelude::*;

	pub(crate) type AccountIdFor<T> = <T as frame_system::Config>::AccountId;

	pub(crate) type BattleConfigFor<T> = BattleConfig<BlockNumberFor<T>>;
	pub(crate) type BattleStateFor<T> = BattleState<BlockNumberFor<T>>;

	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self, I>>
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;
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
	pub type PlayerPositions<T: Config<I>, I: 'static = ()> =
		StorageDoubleMap<_, Identity, Coordinates, Identity, AccountIdFor<T>, (), OptionQuery>;

	#[pallet::storage]
	pub type GridOccupancy<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, Coordinates, u8, ValueQuery>;

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
		InitialPositionOutsideBoundaries,
		InitialPositionAlreadyOccupied,
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
					SchedulerAction::Input(count) =>
						Self::switch_to_phase(BattlePhase::Input) +
							Self::insert_next_schedule(now, count),
					SchedulerAction::Reveal => Self::switch_to_phase(BattlePhase::Reveal),
					SchedulerAction::Execution =>
						Self::switch_to_phase(BattlePhase::Execution) + Self::resolve_battles(),
					SchedulerAction::Shrink =>
						Self::switch_to_phase(BattlePhase::Shrink) + Self::resolve_wall_shrinkage(),
					SchedulerAction::Verify =>
						Self::switch_to_phase(BattlePhase::Verification) +
							Self::resolve_game_state(now),
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

		pub(crate) fn insert_next_schedule(
			now: BlockNumberFor<T>,
			input_phase_count: u8,
		) -> Weight {
			let mut time = now.saturating_add(INPUT_PHASE_DURATION.into());
			BattleSchedules::<T, I>::insert(time, SchedulerAction::Reveal);

			time = time.saturating_add(REVEAL_PHASE_DURATION.into());
			BattleSchedules::<T, I>::insert(time, SchedulerAction::Execution);

			let should_shrink = input_phase_count % SHRINK_PHASE_FREQUENCY == 0;
			if should_shrink {
				time = time.saturating_add(EXECUTION_PHASE_DURATION.into());
				BattleSchedules::<T, I>::insert(time, SchedulerAction::Shrink);

				time = time.saturating_add(SHRINK_PHASE_DURATION.into());
				BattleSchedules::<T, I>::insert(time, SchedulerAction::Verify);
			} else {
				time = time.saturating_add(EXECUTION_PHASE_DURATION.into());
				BattleSchedules::<T, I>::insert(time, SchedulerAction::Verify);
			}

			time = time.saturating_add(VERIFICATION_PHASE_DURATION.into());

			let new_count = core::cmp::max(1, (input_phase_count % SHRINK_PHASE_FREQUENCY) + 1);
			BattleSchedules::<T, I>::insert(time, SchedulerAction::Input(new_count));

			if should_shrink {
				T::DbWeight::get().writes(5)
			} else {
				T::DbWeight::get().writes(4)
			}
		}

		pub(crate) fn mark_account_as_defeated(account: &AccountIdFor<T>) -> (u64, u64) {
			let mut r = 0;
			let mut w = 0;

			PlayerDetails::<T, I>::mutate(account, |maybe_details| {
				if let Some(details) = maybe_details {
					if !matches!(details.state, PlayerState::Defeated) {
						details.state = PlayerState::Defeated;

						GridOccupancy::<T, I>::mutate(details.position, |player_count| {
							*player_count = player_count.saturating_sub(1);
						});
						PlayerPositions::<T, I>::remove(details.position, account);

						r = 2;
						w = 3;

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

			let occupancy_vec = GridOccupancy::<T, I>::iter()
				.filter(|(_, count)| *count > 1)
				.collect::<Vec<_>>();
			r = r.saturating_add(occupancy_vec.len() as u64);

			for (position, _) in occupancy_vec.iter() {
				let mut accounts =
					PlayerPositions::<T, I>::iter_key_prefix(position).collect::<Vec<_>>();
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

						if acc1_weapon.battle_against(acc2_weapon) {
							let (r1, w1) = Self::mark_account_as_defeated(&acc2);
							r = r.saturating_add(r1);
							w = w.saturating_add(w1);
							accounts.push(acc1);
						} else {
							let (r1, w1) = Self::mark_account_as_defeated(&acc1);
							r = r.saturating_add(r1);
							w = w.saturating_add(w1);
							accounts.push(acc2);
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

					for account in PlayerPositions::<T, I>::iter()
						.filter(|(coordinates, _, _)| !boundaries.is_in_boundaries(coordinates))
						.map(|(_, account, _)| account)
					{
						let (r1, w1) = Self::mark_account_as_defeated(&account);
						r = r.saturating_add(r1 + 1);
						w = w.saturating_add(w1);
					}
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
						let player_count = PlayerPositions::<T, I>::iter_values().count();

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
			let _ = PlayerPositions::<T, I>::clear(MAX_PLAYER_PER_BATTLE as u32, None);
			let _ = PlayerDetails::<T, I>::clear(MAX_PLAYER_PER_BATTLE as u32, None);
			let _ = GridOccupancy::<T, I>::clear(MAX_PLAYER_PER_BATTLE as u32, None);
		}

		fn update_player_data_using_action(
			account: &AccountIdFor<T>,
			player_data: &mut PlayerData,
			action: PlayerAction,
		) {
			match action {
				PlayerAction::Move(new_position) => {
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
					Self::update_player_positions_storage(
						account,
						player_data.position,
						new_position,
					);
					player_data.position = new_position;
					player_data.weapon = new_weapon;
				},
			}

			player_data.state = PlayerState::RevealedAction;
		}

		fn update_player_positions_storage(
			account: &AccountIdFor<T>,
			prev_position: Coordinates,
			new_position: Coordinates,
		) {
			PlayerPositions::<T, I>::remove(prev_position, account);
			GridOccupancy::<T, I>::mutate(prev_position, |value| {
				*value = value.saturating_sub(1);
			});

			PlayerPositions::<T, I>::insert(new_position, account, ());
			GridOccupancy::<T, I>::mutate(new_position, |value| {
				*value = value.saturating_add(1);
			});
		}
	}

	impl<T: Config<I>, I: 'static> BattleProvider<AccountIdFor<T>> for Pallet<T, I> {
		fn try_start_battle(
			game_duration: u32,
			max_players: u8,
			grid_size: Coordinates,
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

				*state = BattleStateFor::<T>::Active {
					phase: BattlePhase::Queueing,
					config: BattleConfigFor::<T> { max_players, run_until },
					boundaries: GridBoundaries::new(grid_size),
				};

				let switch_at = current_block.saturating_add(QUEUE_DURATION.into());

				BattleSchedules::<T, I>::insert(switch_at, SchedulerAction::Input(1));

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
			initial_position: Coordinates,
			initial_weapon: PlayerWeapon,
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
				ensure!(
					boundaries.is_in_boundaries(&initial_position),
					Error::<T, I>::InitialPositionOutsideBoundaries
				);
				ensure!(
					!GridOccupancy::<T, I>::contains_key(initial_position),
					Error::<T, I>::InitialPositionAlreadyOccupied
				);

				let player_data = PlayerData {
					position: initial_position,
					weapon: initial_weapon,
					state: PlayerState::Inactive,
				};

				// Setting player initial position
				PlayerPositions::<T, I>::insert(initial_position, account, ());
				GridOccupancy::<T, I>::mutate(initial_position, |value| {
					*value = value.saturating_add(1);
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
				BattleStateFor::<T>::Active { phase, .. } => match phase {
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
