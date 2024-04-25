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

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub mod account;
pub mod config;
pub mod traits;

use frame_support::{pallet_prelude::*, PalletId};
use frame_system::pallet_prelude::*;

use account::*;
pub use config::*;
pub use traits::*;

const LOG_TARGET: &str = "runtime::ajuna-tournament";

pub const MAX_PLAYERS: u32 = 10;

pub type TournamentId = u32;
pub type Rank = u32;
pub type RankingTable<T> = BoundedVec<T, ConstU32<MAX_PLAYERS>>;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::traits::{Currency, ExistenceRequirement};
	use sp_arithmetic::traits::AtLeast16BitUnsigned;
	use sp_runtime::{
		traits::{AccountIdConversion, CheckedDiv, SaturatedConversion},
		Saturating,
	};

	pub type AccountIdFor<T> = <T as frame_system::Config>::AccountId;
	pub(crate) type BalanceOf<T, I> =
		<<T as Config<I>>::Currency as Currency<AccountIdFor<T>>>::Balance;
	pub(crate) type TournamentScheduledActionFor<T, I> =
		TournamentScheduledAction<<T as Config<I>>::SeasonId>;
	pub type TournamentConfigFor<T, I> = TournamentConfig<BlockNumberFor<T>, BalanceOf<T, I>>;
	pub(crate) type RankingTableFor<T, I> =
		RankingTable<(<T as Config<I>>::EntityId, <T as Config<I>>::RankedEntity)>;
	pub(crate) type RewardClaimStateFor<T> = RewardClaimState<AccountIdFor<T>>;
	pub(crate) type TournamentStateFor<T, I> = TournamentState<BalanceOf<T, I>>;
	pub(crate) type GoldenDuckStateFor<T, I> = GoldenDuckState<<T as Config<I>>::EntityId>;

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config {
		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// The overarching event type.
		type RuntimeEvent: From<Event<Self, I>>
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type Currency: Currency<Self::AccountId>;

		/// The season identifier type.
		type SeasonId: Member
			+ Parameter
			+ MaxEncodedLen
			+ AtLeast16BitUnsigned
			+ Into<u32>
			+ Copy
			+ Default;

		/// The ranked entity identifier type.
		type EntityId: Member + Parameter + MaxEncodedLen + PartialOrd + Ord;

		/// The ranked entities type
		type RankedEntity: Member + Parameter + MaxEncodedLen;

		/// Minimum duration of the tournament active and claim periods in blocks.
		#[pallet::constant]
		type MinimumTournamentPhaseDuration: Get<BlockNumberFor<Self>>;
	}

	#[pallet::storage]
	pub type TournamentSchedules<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, BlockNumberFor<T>, TournamentScheduledActionFor<T, I>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn treasury_accounts)]
	pub type TreasuryAccountsCache<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, T::SeasonId, AccountIdFor<T>, OptionQuery>;

	#[pallet::storage]
	pub type NextTournamentIds<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, T::SeasonId, TournamentId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn tournaments)]
	pub type Tournaments<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
		_,
		Identity,
		T::SeasonId,
		Identity,
		TournamentId,
		TournamentConfigFor<T, I>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn active_tournaments)]
	pub type ActiveTournaments<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, T::SeasonId, TournamentStateFor<T, I>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn rankings)]
	pub type TournamentRankings<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
		_,
		Identity,
		T::SeasonId,
		Identity,
		TournamentId,
		RankingTableFor<T, I>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn tournament_reward_claims)]
	pub type TournamentRewardClaims<T: Config<I>, I: 'static = ()> = StorageNMap<
		_,
		(
			NMapKey<Identity, T::SeasonId>,
			NMapKey<Identity, TournamentId>,
			NMapKey<Blake2_128Concat, RankingTableIndex>,
		),
		RewardClaimStateFor<T>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn golden_ducks)]
	pub type GoldenDucks<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
		_,
		Identity,
		T::SeasonId,
		Identity,
		TournamentId,
		GoldenDuckStateFor<T, I>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn golden_duck_reward_claims)]
	pub type GoldenDuckRewardClaims<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
		_,
		Identity,
		T::SeasonId,
		Identity,
		TournamentId,
		RewardClaimStateFor<T>,
		OptionQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		TournamentCreated {
			season_id: T::SeasonId,
			tournament_id: TournamentId,
		},
		TournamentRemoved {
			season_id: T::SeasonId,
			tournament_id: TournamentId,
		},
		TournamentActivePeriodStarted {
			season_id: T::SeasonId,
			tournament_id: TournamentId,
		},
		TournamentClaimPeriodStarted {
			season_id: T::SeasonId,
			tournament_id: TournamentId,
		},
		TournamentEnded {
			season_id: T::SeasonId,
			tournament_id: TournamentId,
		},
		EntityEnteredRanking {
			season_id: T::SeasonId,
			tournament_id: TournamentId,
			entity_id: T::EntityId,
			rank: Rank,
		},
		EntityBecameGoldenDuck {
			season_id: T::SeasonId,
			tournament_id: TournamentId,
			entity_id: T::EntityId,
		},
		RankingRewardClaimed {
			season_id: T::SeasonId,
			tournament_id: TournamentId,
			entity_id: T::EntityId,
			account: AccountIdFor<T>,
		},
		GoldenDuckRewardClaimed {
			season_id: T::SeasonId,
			tournament_id: TournamentId,
			entity_id: T::EntityId,
			account: AccountIdFor<T>,
		},
	}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		/// There's no active tournament for the selected season.
		NoActiveTournamentForSeason,
		/// The current tournament is not in its reward claim period.
		TournamentNotInClaimPeriod,
		/// The latest tournament for the selected season identifier already started,
		/// so it cannot be removed anymore.
		LatestTournamentAlreadyStarted,
		/// There's already an active tournament for the selected season.
		AnotherTournamentAlreadyActiveForSeason,
		/// Cannot find tournament data for the selected season id and tournament id combination.
		TournamentNotFound,
		/// Cannot activate a tournament before its configured block start,
		TournamentActivationTooEarly,
		/// Cannot deactivate a tournament before its configured block end,
		TournamentEndingTooEarly,
		/// An error occurred trying to rank an entity,
		FailedToRankEntity,
		/// Tournament configuration is invalid.
		InvalidTournamentConfig,
		/// Tournament schedule already in use by another tournament.
		CannotScheduleTournament,
		/// A ranking duck candidate proposed by an account is not in the winner's table.
		RankingCandidateNotInWinnerTable,
		/// A golden duck candidate proposed by an account is not the actual golden duck winner.
		GoldenDuckCandidateNotWinner,
		/// The reward for this tournament has already been claimed
		TournamentRewardAlreadyClaimed,
	}

	#[pallet::hooks]
	impl<T: Config<I>, I: 'static> Hooks<BlockNumberFor<T>> for Pallet<T, I> {
		fn on_initialize(now: BlockNumberFor<T>) -> Weight {
			let mut weight = T::DbWeight::get().reads(1);

			if let Some(action) = TournamentSchedules::<T, I>::take(now) {
				let w = match action {
					TournamentScheduledAction::StartActivePhase(season_id, tournament_id) =>
						Self::try_start_next_tournament_for(season_id, tournament_id),
					TournamentScheduledAction::SwitchToClaimPhase(season_id, tournament_id) =>
						Self::try_switch_tournament_to_claim_period_for(season_id, tournament_id),
					TournamentScheduledAction::EndClaimPhase(season_id, tournament_id) =>
						Self::try_finish_tournament_claim_period_for(season_id, tournament_id),
				};
				weight.saturating_accrue(w);
			};

			weight
		}
	}

	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		/// The account ID of the sub-account for a given season_id/tournament_id.
		pub fn tournament_treasury_account_id(season_id: T::SeasonId) -> T::AccountId {
			if let Some(account) = TreasuryAccountsCache::<T, I>::get(season_id) {
				account
			} else {
				let account_builder =
					TournamentTreasuryAccount::<T::SeasonId>::new(T::PalletId::get(), season_id);
				let account: AccountIdFor<T> = account_builder.into_account_truncating();
				TreasuryAccountsCache::<T, I>::insert(season_id, account.clone());
				account
			}
		}

		fn ensure_valid_tournament(
			season_id: &T::SeasonId,
			config: &TournamentConfigFor<T, I>,
		) -> DispatchResult {
			let current_block = <frame_system::Pallet<T>>::block_number();
			ensure!(current_block < config.start, Error::<T, I>::InvalidTournamentConfig);
			ensure!(config.start < config.active_end, Error::<T, I>::InvalidTournamentConfig);
			ensure!(config.active_end < config.claim_end, Error::<T, I>::InvalidTournamentConfig);

			ensure!(
				config.active_end.saturating_sub(config.start) >=
					T::MinimumTournamentPhaseDuration::get(),
				Error::<T, I>::InvalidTournamentConfig
			);
			ensure!(
				config.claim_end.saturating_sub(config.active_end) >=
					T::MinimumTournamentPhaseDuration::get(),
				Error::<T, I>::InvalidTournamentConfig
			);

			if let Some(prev_config) = Tournaments::<T, I>::get(
				season_id,
				NextTournamentIds::<T, I>::get(season_id).saturating_sub(1),
			) {
				ensure!(
					prev_config.claim_end < config.start,
					Error::<T, I>::InvalidTournamentConfig
				);
			}

			ensure!(
				config.initial_reward.is_some() || config.take_fee_percentage.is_some(),
				Error::<T, I>::InvalidTournamentConfig
			);

			if let Some(initial_reward) = config.initial_reward {
				ensure!(initial_reward > 0_u32.into(), Error::<T, I>::InvalidTournamentConfig);
			}

			if let Some(max_reward) = config.max_reward {
				ensure!(max_reward > 0_u32.into(), Error::<T, I>::InvalidTournamentConfig);
			}

			if let Some(fee_perc) = config.take_fee_percentage {
				ensure!(fee_perc <= 100, Error::<T, I>::InvalidTournamentConfig);
			}

			// Because the entries in the 'reward_distribution' table are u8, by upcasting them to
			// u16 before folding them together, we can avoid any potential overflows
			let reward_table_total_dist =
				config.reward_distribution.iter().fold(0_u16, |a, b| a + (*b as u16));
			let golden_duck_dist = match config.golden_duck_config {
				GoldenDuckConfig::Disabled => 0,
				GoldenDuckConfig::Enabled(percentage) => {
					ensure!(percentage > 0, Error::<T, I>::InvalidTournamentConfig);
					percentage
				},
			} as u16;

			ensure!(
				(reward_table_total_dist + golden_duck_dist) <= 100,
				Error::<T, I>::InvalidTournamentConfig
			);

			ensure!(
				config.max_players > 0 && config.max_players <= MAX_PLAYERS,
				Error::<T, I>::InvalidTournamentConfig
			);

			Ok(())
		}

		fn try_insert_tournament_schedule(
			season_id: &T::SeasonId,
			tournament_id: &TournamentId,
			config: &TournamentConfigFor<T, I>,
		) -> DispatchResult {
			TournamentSchedules::<T, I>::try_mutate(config.start, |action| match action {
				None => {
					*action = Some(TournamentScheduledAction::StartActivePhase(
						*season_id,
						*tournament_id,
					));
					Ok(())
				},
				Some(_) => Err(Error::<T, I>::CannotScheduleTournament),
			})?;
			TournamentSchedules::<T, I>::try_mutate(config.active_end, |action| match action {
				None => {
					*action = Some(TournamentScheduledAction::SwitchToClaimPhase(
						*season_id,
						*tournament_id,
					));
					Ok(())
				},
				Some(_) => Err(Error::<T, I>::CannotScheduleTournament),
			})?;
			TournamentSchedules::<T, I>::try_mutate(config.claim_end, |action| match action {
				None => {
					*action =
						Some(TournamentScheduledAction::EndClaimPhase(*season_id, *tournament_id));
					Ok(())
				},
				Some(_) => Err(Error::<T, I>::CannotScheduleTournament),
			})?;

			Ok(())
		}

		fn update_tournament_rewards_storage_for(
			season_id: &T::SeasonId,
			tournament_id: TournamentId,
		) {
			for index in 0..TournamentRankings::<T, I>::get(season_id, tournament_id).len() {
				TournamentRewardClaims::<T, I>::insert(
					(season_id, tournament_id, index as u32),
					RewardClaimState::Unclaimed,
				);
			}

			if matches!(
				GoldenDucks::<T, I>::get(season_id, tournament_id),
				GoldenDuckState::Enabled(_, Some(_))
			) {
				GoldenDuckRewardClaims::<T, I>::insert(
					season_id,
					tournament_id,
					RewardClaimState::Unclaimed,
				);
			}
		}

		fn try_get_active_tournament_id_for(
			season_id: &T::SeasonId,
		) -> Result<TournamentId, DispatchError> {
			match ActiveTournaments::<T, I>::get(season_id) {
				TournamentState::ActivePeriod(tournament_id) |
				TournamentState::ClaimPeriod(tournament_id, _) => Ok(tournament_id),
				_ => Err(Error::<T, I>::NoActiveTournamentForSeason.into()),
			}
		}

		/// Returns the available funds for payout in the winner/golden duck accounts.
		/// If the amount is limited by 'max_reward' config in the tournament the amount will be
		/// limited to that amount.
		fn get_reward_payout(
			tournament_config: &TournamentConfigFor<T, I>,
			treasury_account: &AccountIdFor<T>,
		) -> BalanceOf<T, I> {
			let total_payout = T::Currency::free_balance(treasury_account);
			if let Some(max_payout) = tournament_config.max_reward {
				sp_std::cmp::min(total_payout, max_payout)
			} else {
				total_payout
			}
		}

		fn try_update_rank_table(
			table: &mut RankingTableFor<T, I>,
			tournament_config: &TournamentConfigFor<T, I>,
			index: usize,
			entity_id: &T::EntityId,
			entity: &T::RankedEntity,
		) -> Result<RankingResult, DispatchError> {
			if index < tournament_config.max_players as usize {
				if table.len() == tournament_config.max_players as usize {
					let _ = table.pop();
				}

				table
					.force_insert_keep_left(index, (entity_id.clone(), entity.clone()))
					.map(|_| RankingResult::Ranked { rank: index.saturated_into() })
					.map_err(|_| Error::<T, I>::FailedToRankEntity.into())
			} else {
				Ok(RankingResult::ScoreTooLow)
			}
		}

		fn try_start_next_tournament_for(
			season_id: T::SeasonId,
			tournament_id: TournamentId,
		) -> Weight {
			if let Some(tournament_config) = Tournaments::<T, I>::get(season_id, tournament_id) {
				let current_block = <frame_system::Pallet<T>>::block_number();

				if tournament_config.start > current_block {
					log::error!(target: LOG_TARGET, "Tried to start a tournament in the incorrect block!");
					return T::DbWeight::get().reads(1)
				}

				ActiveTournaments::<T, I>::insert(
					season_id,
					TournamentState::ActivePeriod(tournament_id),
				);

				Self::deposit_event(Event::<T, I>::TournamentActivePeriodStarted {
					season_id,
					tournament_id,
				});

				T::DbWeight::get().reads_writes(1, 1)
			} else {
				log::error!(target: LOG_TARGET, "Tried to start a tournament with missing config!");
				T::DbWeight::get().reads(1)
			}
		}

		fn try_switch_tournament_to_claim_period_for(
			season_id: T::SeasonId,
			tournament_id: TournamentId,
		) -> Weight {
			let current_block = <frame_system::Pallet<T>>::block_number();

			if let Some(tournament_config) = Tournaments::<T, I>::get(season_id, tournament_id) {
				if tournament_config.active_end > current_block {
					log::error!(target: LOG_TARGET, "Tried to switch to claim a tournament in the incorrect block!");
					return T::DbWeight::get().reads(1)
				}

				let treasury_account = Self::tournament_treasury_account_id(season_id);
				let reward_pot = Self::get_reward_payout(&tournament_config, &treasury_account);
				ActiveTournaments::<T, I>::mutate(season_id, |state| {
					*state = TournamentState::ClaimPeriod(tournament_id, reward_pot)
				});

				Self::update_tournament_rewards_storage_for(&season_id, tournament_id);

				Self::deposit_event(Event::<T, I>::TournamentClaimPeriodStarted {
					season_id,
					tournament_id,
				});

				T::DbWeight::get().reads_writes(1, 1)
			} else {
				log::error!(target: LOG_TARGET, "Tried to switch a tournament to claim phase with missing config!");
				T::DbWeight::get().reads(1)
			}
		}

		fn try_finish_tournament_claim_period_for(
			season_id: T::SeasonId,
			tournament_id: TournamentId,
		) -> Weight {
			let current_block = <frame_system::Pallet<T>>::block_number();

			if let Some(tournament_config) = Tournaments::<T, I>::get(season_id, tournament_id) {
				if tournament_config.claim_end > current_block {
					log::error!(target: LOG_TARGET, "Tried to finish a tournament in the incorrect block!");
					return T::DbWeight::get().reads(1)
				}

				ActiveTournaments::<T, I>::mutate(season_id, |state| {
					*state = TournamentState::Finished(tournament_id)
				});

				Self::deposit_event(Event::<T, I>::TournamentEnded { season_id, tournament_id });

				T::DbWeight::get().reads_writes(1, 2)
			} else {
				log::error!(target: LOG_TARGET, "Tried to finish a tournament with missing config!");
				T::DbWeight::get().reads(1)
			}
		}
	}

	impl<T: Config<I>, I: 'static>
		TournamentInspector<T::SeasonId, BlockNumberFor<T>, BalanceOf<T, I>, AccountIdFor<T>>
		for Pallet<T, I>
	{
		fn get_active_tournament_config_for(
			season_id: &T::SeasonId,
		) -> Option<(TournamentId, TournamentConfigFor<T, I>)> {
			match ActiveTournaments::<T, I>::get(season_id) {
				TournamentState::ActivePeriod(tournament_id) |
				TournamentState::ClaimPeriod(tournament_id, _) =>
					if let Some(tournament_config) =
						Tournaments::<T, I>::get(season_id, tournament_id)
					{
						Some((tournament_id, tournament_config))
					} else {
						log::error!(target: LOG_TARGET, "No tournament config found for active tournament!");
						None
					},
				_ => None,
			}
		}

		fn get_active_tournament_state_for(season_id: &T::SeasonId) -> TournamentStateFor<T, I> {
			ActiveTournaments::<T, I>::get(season_id)
		}

		fn is_golden_duck_enabled_for(season_id: &T::SeasonId) -> bool {
			match ActiveTournaments::<T, I>::get(season_id) {
				TournamentState::ActivePeriod(tournament_id) |
				TournamentState::ClaimPeriod(tournament_id, _) => {
					matches!(
						GoldenDucks::<T, I>::get(season_id, tournament_id),
						GoldenDuckStateFor::<T, I>::Enabled(_, _)
					)
				},
				_ => false,
			}
		}

		fn get_treasury_account_for(season_id: &T::SeasonId) -> AccountIdFor<T> {
			Self::tournament_treasury_account_id(*season_id)
		}
	}

	impl<T: Config<I>, I: 'static>
		TournamentMutator<AccountIdFor<T>, T::SeasonId, BlockNumberFor<T>, BalanceOf<T, I>>
		for Pallet<T, I>
	{
		fn try_create_new_tournament_for(
			creator: &AccountIdFor<T>,
			season_id: &T::SeasonId,
			config: TournamentConfigFor<T, I>,
		) -> Result<TournamentId, DispatchError> {
			Self::ensure_valid_tournament(season_id, &config)?;

			let next_tournament_id =
				NextTournamentIds::<T, I>::mutate(season_id, |tournament_id| {
					let assigned_id = *tournament_id;
					*tournament_id = tournament_id.saturating_add(1);
					assigned_id
				});

			Self::try_insert_tournament_schedule(season_id, &next_tournament_id, &config)?;

			if let Some(reward) = config.initial_reward {
				let treasury_account = Self::tournament_treasury_account_id(*season_id);
				T::Currency::transfer(
					creator,
					&treasury_account,
					reward,
					ExistenceRequirement::KeepAlive,
				)?;
			}

			if let GoldenDuckConfig::Enabled(percentage) = config.golden_duck_config {
				GoldenDucks::<T, I>::insert(
					season_id,
					next_tournament_id,
					GoldenDuckStateFor::<T, I>::Enabled(percentage, None),
				);
			}

			Tournaments::<T, I>::insert(season_id, next_tournament_id, config);

			Self::deposit_event(Event::<T, I>::TournamentCreated {
				season_id: *season_id,
				tournament_id: next_tournament_id,
			});

			Ok(next_tournament_id)
		}

		fn try_remove_latest_tournament_for(season_id: &T::SeasonId) -> DispatchResult {
			match Self::get_active_tournament_state_for(season_id) {
				TournamentState::Inactive =>
					NextTournamentIds::<T, I>::try_mutate(season_id, |tournament_id| {
						let prev_id = tournament_id.saturating_sub(1);
						if let Some(config) = Tournaments::<T, I>::take(season_id, prev_id) {
							GoldenDucks::<T, I>::remove(season_id, prev_id);

							TournamentSchedules::<T, I>::remove(config.start);
							TournamentSchedules::<T, I>::remove(config.active_end);
							TournamentSchedules::<T, I>::remove(config.claim_end);

							*tournament_id = prev_id;

							Self::deposit_event(Event::<T, I>::TournamentRemoved {
								season_id: *season_id,
								tournament_id: prev_id,
							});

							Ok(())
						} else {
							Err(Error::<T, I>::TournamentNotFound.into())
						}
					}),
				_ => Err(Error::<T, I>::TournamentNotInClaimPeriod.into()),
			}
		}
	}

	impl<T: Config<I>, I: 'static> TournamentRanker<T::SeasonId, T::RankedEntity, T::EntityId>
		for Pallet<T, I>
	{
		fn try_rank_entity_in_tournament_for<R>(
			season_id: &T::SeasonId,
			entity_id: &T::EntityId,
			entity: &T::RankedEntity,
			ranker: &R,
		) -> DispatchResult
		where
			R: EntityRank<EntityId = T::EntityId, Entity = T::RankedEntity>,
		{
			if !ranker.can_rank((entity_id, entity)) {
				return Ok(());
			}

			ensure!(
				matches!(
					Self::get_active_tournament_state_for(season_id),
					TournamentState::ActivePeriod(_)
				),
				Error::<T, I>::NoActiveTournamentForSeason
			);

			let (tournament_id, tournament_config) =
				Self::get_active_tournament_config_for(season_id)
					.ok_or::<Error<T, I>>(Error::<T, I>::TournamentNotFound)?;

			TournamentRankings::<T, I>::mutate(season_id, tournament_id, |table| {
				match table.binary_search_by(|(other_id, other)| {
					ranker.rank_against((entity_id, entity), (other_id, other))
				}) {
					// The entity is already in the ranking table,
					// nothing to do here
					Ok(_) => Ok(()),
					// The entity is not in the table,
					// we need to check if it should be
					// inserted or not
					Err(index) => {
						match Self::try_update_rank_table(
							table,
							&tournament_config,
							index,
							entity_id,
							entity,
						)? {
							// The entity didn't make it to the ranking
							RankingResult::ScoreTooLow => Ok(()),
							// The entity made it to the ranking and the
							// table has been successfully updated
							RankingResult::Ranked { rank } => {
								Self::deposit_event(
									crate::pallet::Event::<T, I>::EntityEnteredRanking {
										season_id: *season_id,
										tournament_id,
										entity_id: entity_id.clone(),
										rank,
									},
								);
								Ok(())
							},
						}
					},
				}
			})
		}

		fn try_rank_entity_for_golden_duck(
			season_id: &T::SeasonId,
			entity_id: &T::EntityId,
		) -> DispatchResult {
			ensure!(
				matches!(
					Self::get_active_tournament_state_for(season_id),
					TournamentState::ActivePeriod(_)
				),
				Error::<T, I>::NoActiveTournamentForSeason
			);

			let tournament_id = Self::try_get_active_tournament_id_for(season_id)?;

			GoldenDucks::<T, I>::mutate(season_id, tournament_id, |state| {
				if let GoldenDuckState::Enabled(payout_perc, ref maybe_entry_id) = state {
					match maybe_entry_id {
						None => {
							*state =
								GoldenDuckState::Enabled(*payout_perc, Some(entity_id.clone()));
							Self::deposit_event(Event::<T, I>::EntityBecameGoldenDuck {
								season_id: *season_id,
								tournament_id,
								entity_id: entity_id.clone(),
							});
						},
						Some(entry_id) if entity_id < entry_id => {
							*state =
								GoldenDuckState::Enabled(*payout_perc, Some(entity_id.clone()));
							Self::deposit_event(Event::<T, I>::EntityBecameGoldenDuck {
								season_id: *season_id,
								tournament_id,
								entity_id: entity_id.clone(),
							});
						},
						_ => {},
					}
				}
			});

			Ok(())
		}
	}

	impl<T: Config<I>, I: 'static> TournamentClaimer<T::SeasonId, AccountIdFor<T>, T::EntityId>
		for Pallet<T, I>
	{
		fn try_claim_tournament_reward_for(
			season_id: &T::SeasonId,
			account: &AccountIdFor<T>,
			entity_id: &T::EntityId,
		) -> DispatchResult {
			ensure!(
				matches!(
					Self::get_active_tournament_state_for(season_id),
					TournamentState::ClaimPeriod(_, _)
				),
				Error::<T, I>::TournamentNotInClaimPeriod
			);

			match ActiveTournaments::<T, I>::get(season_id) {
				TournamentState::ActivePeriod(_) =>
					Err(Error::<T, I>::TournamentNotInClaimPeriod.into()),
				TournamentState::ClaimPeriod(tournament_id, reward_pot) => {
					let index = TournamentRankings::<T, I>::get(season_id, tournament_id)
						.iter()
						.position(|(entry_id, _)| entry_id == entity_id)
						.ok_or(Error::<T, I>::RankingCandidateNotInWinnerTable)?;

					TournamentRewardClaims::<T, I>::try_mutate(
						(season_id, tournament_id, index as u32),
						|state| {
							ensure!(
								matches!(state, Some(RewardClaimState::Unclaimed)),
								Error::<T, I>::TournamentRewardAlreadyClaimed
							);

							let tournament_config =
								Tournaments::<T, I>::get(season_id, tournament_id)
									.ok_or(Error::<T, I>::TournamentNotFound)?;
							let treasury_account = Self::tournament_treasury_account_id(*season_id);

							let payout_percentage = tournament_config
								.reward_distribution
								.get(index)
								.copied()
								.unwrap_or_default();

							let account_payout = reward_pot
								.saturating_mul(payout_percentage.into())
								.checked_div(&100_u32.into())
								.unwrap_or_default();

							if account_payout > 0_u32.into() {
								T::Currency::transfer(
									&treasury_account,
									account,
									account_payout,
									ExistenceRequirement::AllowDeath,
								)?;
							}

							*state = Some(RewardClaimState::Claimed(account.clone()));

							Self::deposit_event(Event::<T, I>::RankingRewardClaimed {
								season_id: *season_id,
								tournament_id,
								entity_id: entity_id.clone(),
								account: account.clone(),
							});

							Ok(())
						},
					)
				},
				_ => Err(Error::<T, I>::NoActiveTournamentForSeason.into()),
			}
		}

		fn try_claim_golden_duck_for(
			season_id: &T::SeasonId,
			account: &AccountIdFor<T>,
			entity_id: &T::EntityId,
		) -> DispatchResult {
			ensure!(
				matches!(
					Self::get_active_tournament_state_for(season_id),
					TournamentState::ClaimPeriod(_, _)
				),
				Error::<T, I>::TournamentNotInClaimPeriod
			);

			match ActiveTournaments::<T, I>::get(season_id) {
				TournamentState::ActivePeriod(_) =>
					Err(Error::<T, I>::TournamentNotInClaimPeriod.into()),
				TournamentState::ClaimPeriod(tournament_id, reward_pot) =>
					match GoldenDucks::<T, I>::get(season_id, tournament_id) {
						GoldenDuckState::Enabled(payout_percentage, Some(ref winner_id))
							if winner_id == entity_id =>
							GoldenDuckRewardClaims::<T, I>::try_mutate(
								season_id,
								tournament_id,
								|state| {
									ensure!(
										matches!(state, Some(RewardClaimState::Unclaimed)),
										Error::<T, I>::TournamentRewardAlreadyClaimed
									);

									let treasury_account =
										Self::tournament_treasury_account_id(*season_id);

									let account_payout = reward_pot
										.saturating_mul(payout_percentage.into())
										.checked_div(&100_u32.into())
										.unwrap_or_default();

									T::Currency::transfer(
										&treasury_account,
										account,
										account_payout,
										ExistenceRequirement::AllowDeath,
									)?;

									*state = Some(RewardClaimState::Claimed(account.clone()));

									Self::deposit_event(Event::<T, I>::GoldenDuckRewardClaimed {
										season_id: *season_id,
										tournament_id,
										entity_id: entity_id.clone(),
										account: account.clone(),
									});

									Ok(())
								},
							),
						_ => Err(Error::<T, I>::GoldenDuckCandidateNotWinner.into()),
					},
				_ => Err(Error::<T, I>::NoActiveTournamentForSeason.into()),
			}
		}
	}
}

/// Result of an attempt to enter the ranks.
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Debug, PartialEq, Copy, Clone)]
pub enum RankingResult {
	/// The entity was successfully ranked.
	Ranked { rank: Rank },
	/// The entity did not make it into the rankings.
	ScoreTooLow,
}
