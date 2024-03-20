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

pub mod traits;

use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::*;

pub use traits::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{
		traits::{Currency, ExistenceRequirement},
		PalletId,
	};
	use sp_runtime::{
		traits::{AccountIdConversion, AtLeast32BitUnsigned, CheckedDiv},
		SaturatedConversion, Saturating,
	};

	pub type AccountIdFor<T> = <T as frame_system::Config>::AccountId;
	pub(crate) type BalanceOf<T, I> =
		<<T as Config<I>>::Currency as Currency<AccountIdFor<T>>>::Balance;
	pub(crate) type SeasonSetFor<T, I> = SeasonSet<<T as Config<I>>::SeasonId>;
	pub type TournamentConfigFor<T, I> = TournamentConfig<BlockNumberFor<T>, BalanceOf<T, I>>;
	pub type PlayerTableFor<T, I> = PlayerTable<<T as Config<I>>::RankedEntity>;
	pub type TournamentStateFor<T, I> = TournamentState<BalanceOf<T, I>>;
	pub type GoldenDuckStateFor<T, I> = GoldenDuckState<<T as Config<I>>::EntityId>;

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
			+ AtLeast32BitUnsigned
			+ Into<u32>
			+ Copy
			+ Default;

		/// The ranked entity identifier type.
		type EntityId: Member + Parameter + MaxEncodedLen + PartialOrd + Ord;

		/// The ranked entities type
		type RankedEntity: Member + Parameter + MaxEncodedLen + PartialOrd + Ord;

		/// Minimum duration of the tournament active and claim periods in blocks.
		#[pallet::constant]
		type MinimumTournamentDuration: Get<BlockNumberFor<Self>>;

		/// The value at which Tournament ids will start at
		#[pallet::constant]
		type InitialTournamentId: Get<TournamentId>;
	}

	/// Stores a
	#[pallet::storage]
	pub type EnabledSeasons<T: Config<I>, I: 'static = ()> =
		StorageValue<_, SeasonSetFor<T, I>, ValueQuery>;

	/// Default value for NextTournamentIds
	#[pallet::type_value]
	pub fn NextTournamentIdDefault<T: Config<I>, I: 'static>() -> TournamentId {
		T::InitialTournamentId::get()
	}
	#[pallet::storage]
	pub type NextTournamentIds<T: Config<I>, I: 'static = ()> = StorageMap<
		_,
		Identity,
		T::SeasonId,
		TournamentId,
		ValueQuery,
		NextTournamentIdDefault<T, I>,
	>;

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
	#[pallet::getter(fn latest_tournaments)]
	pub type LatestTournaments<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, T::SeasonId, TournamentId, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn rankings)]
	pub type TournamentRankings<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
		_,
		Identity,
		T::SeasonId,
		Identity,
		TournamentId,
		PlayerTableFor<T, I>,
		ValueQuery,
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

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		TournamentCreated { season_id: T::SeasonId, tournament_id: TournamentId },
		TournamentActivePeriodStarted { season_id: T::SeasonId, tournament_id: TournamentId },
		TournamentClaimPeriodStarted { season_id: T::SeasonId, tournament_id: TournamentId },
		TournamentEnded { season_id: T::SeasonId, tournament_id: TournamentId },
	}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		/// There's no active tournament for the selected season.
		NoActiveTournamentForSeason,
		/// The current tournament is not in its reward claim period.
		TournamentNotInClaimPeriod,
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
		/// Cannot enable the processing of tournaments for the selected season id since the
		/// maximum amount of seasons is already enabled.
		CannotEnableTournamentProcessingForSeason,
		/// Cannot disable the processing of tournaments for the selected season id since there is
		/// still an active tournament for it.
		CannotDisableTournamentProcessingForSeason,
		/// A ranking duck candidate proposed by an account is not in the winner's table.
		RankingCandidateNotInWinnerTable,
		/// A golden duck candidate proposed by an account is not the actual golden duck winner.
		GoldenDuckCandidateNotWinner,
	}

	#[pallet::hooks]
	impl<T: Config<I>, I: 'static> Hooks<BlockNumberFor<T>> for Pallet<T, I> {
		fn on_initialize(now: BlockNumberFor<T>) -> Weight {
			let enabled_seasons = EnabledSeasons::<T, I>::get();
			let enabled_season_len = enabled_seasons.len();
			let mut weight = T::DbWeight::get().reads(1);

			if enabled_season_len > 0 {
				let base_modulo = BlockNumberFor::<T>::from(enabled_season_len as u32);
				let modulo = {
					let block_spacing_factor = 10_u32.into();
					(base_modulo * block_spacing_factor) + 1_u32.into()
				};

				// In order to check the next season_id tournament every 'max_season_id' multiplied
				// by the 'block_spacing_factor' we just have to compute the modulo of the current
				// block number with the 'max_season_id' multiply that value byt the spacing factor
				// and then add 1 so that at each check we move to the next season id. With this we
				// would check at block 0 season_id 0, then at block ('max_season_id' *
				// 'block_spacing_factor') + 1 season_id 1 and so on...
				if now % modulo == 0_u32.into() {
					let index_to_check = (now % base_modulo).saturated_into::<usize>();
					if let Some(season_id) = enabled_seasons.into_iter().nth(index_to_check) {
						match ActiveTournaments::<T, I>::get(season_id) {
							TournamentState::Inactive => {
								weight += Self::try_start_next_tournament_for(season_id);
							},
							TournamentState::ActivePeriod(tournament_id) =>
								weight += Self::try_finish_tournament_active_period_for(
									season_id,
									tournament_id,
								),
							TournamentState::ClaimPeriod(tournament_id, _) =>
								weight += Self::try_finish_tournament_claim_period_for(
									season_id,
									tournament_id,
								),
						}
					}

					weight
				} else {
					weight
				}
			} else {
				weight
			}
		}
	}

	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		/// The account ID of the sub-account for a given season_id/tournament_id.
		pub fn tournament_treasury_account_id(season_id: T::SeasonId) -> T::AccountId {
			let account =
				TournamentTreasuryAccount::<T::SeasonId>::new(T::PalletId::get(), season_id);
			account.into_account_truncating()
		}

		fn ensure_valid_tournament(
			season_id: &T::SeasonId,
			config: &TournamentConfigFor<T, I>,
		) -> Result<u16, DispatchError> {
			let current_block = <frame_system::Pallet<T>>::block_number();
			ensure!(current_block < config.start, Error::<T, I>::InvalidTournamentConfig);
			ensure!(config.start < config.end, Error::<T, I>::InvalidTournamentConfig);
			ensure!(config.end < config.claim_end, Error::<T, I>::InvalidTournamentConfig);

			ensure!(
				config.end.saturating_sub(config.start) >= T::MinimumTournamentDuration::get(),
				Error::<T, I>::InvalidTournamentConfig
			);
			ensure!(
				config.claim_end.saturating_sub(config.end) >= T::MinimumTournamentDuration::get(),
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

			let reward_table_total_dist =
				config.reward_table.iter().fold(0_u16, |a, b| a + (*b as u16));
			ensure!(reward_table_total_dist <= 100, Error::<T, I>::InvalidTournamentConfig);

			ensure!(
				config.max_players > 0 && config.max_players <= MAX_PLAYERS,
				Error::<T, I>::InvalidTournamentConfig
			);

			Ok(reward_table_total_dist)
		}

		fn try_get_current_tournament_id_for(
			season_id: &T::SeasonId,
		) -> Result<TournamentId, DispatchError> {
			match ActiveTournaments::<T, I>::get(season_id) {
				TournamentState::Inactive => Err(Error::<T, I>::NoActiveTournamentForSeason.into()),
				TournamentState::ActivePeriod(tournament_id) |
				TournamentState::ClaimPeriod(tournament_id, _) => Ok(tournament_id),
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
			table: &mut PlayerTableFor<T, I>,
			tournament_config: &TournamentConfigFor<T, I>,
			index: usize,
			entity: &T::RankedEntity,
		) -> DispatchResult {
			if index < tournament_config.max_players as usize {
				if table.len() == tournament_config.max_players as usize {
					let _ = table.pop();
				}
				table
					.force_insert_keep_left(index, entity.clone())
					.map(|_| ())
					.map_err(|_| Error::<T, I>::FailedToRankEntity.into())
			} else {
				Ok(())
			}
		}

		fn try_start_next_tournament_for(season_id: T::SeasonId) -> Weight {
			let next_tournament_id = LatestTournaments::<T, I>::get(season_id)
				.map(|id| id.saturating_add(1))
				.unwrap_or(T::InitialTournamentId::get());
			let current_block = <frame_system::Pallet<T>>::block_number();

			if let Some(tournament_config) = Tournaments::<T, I>::get(season_id, next_tournament_id)
			{
				if tournament_config.start > current_block {
					return T::DbWeight::get().reads(1)
				}

				ActiveTournaments::<T, I>::insert(
					season_id,
					TournamentState::ActivePeriod(next_tournament_id),
				);

				Self::deposit_event(Event::<T, I>::TournamentActivePeriodStarted {
					season_id,
					tournament_id: next_tournament_id,
				});

				T::DbWeight::get().reads_writes(1, 1)
			} else {
				T::DbWeight::get().reads(1)
			}
		}

		fn try_finish_tournament_active_period_for(
			season_id: T::SeasonId,
			tournament_id: TournamentId,
		) -> Weight {
			let current_block = <frame_system::Pallet<T>>::block_number();

			if let Some(tournament_config) = Tournaments::<T, I>::get(season_id, tournament_id) {
				if tournament_config.end > current_block {
					return T::DbWeight::get().reads(1)
				}

				let treasury_account = Self::tournament_treasury_account_id(season_id);
				let reward_pot = Self::get_reward_payout(&tournament_config, &treasury_account);
				ActiveTournaments::<T, I>::mutate(season_id, |state| {
					*state = TournamentState::ClaimPeriod(tournament_id, reward_pot)
				});

				Self::deposit_event(Event::<T, I>::TournamentClaimPeriodStarted {
					season_id,
					tournament_id,
				});

				T::DbWeight::get().reads_writes(1, 1)
			} else {
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
					return T::DbWeight::get().reads(1)
				}

				LatestTournaments::<T, I>::insert(season_id, tournament_id);
				ActiveTournaments::<T, I>::mutate(season_id, |state| {
					*state = TournamentState::Inactive
				});

				Self::deposit_event(Event::<T, I>::TournamentEnded { season_id, tournament_id });

				T::DbWeight::get().reads_writes(1, 2)
			} else {
				T::DbWeight::get().reads(0)
			}
		}
	}

	impl<T: Config<I>, I: 'static>
		TournamentInspector<T::SeasonId, BlockNumberFor<T>, BalanceOf<T, I>, AccountIdFor<T>>
		for Pallet<T, I>
	{
		fn get_active_tournament_config_for(
			season_id: &T::SeasonId,
		) -> Option<TournamentConfigFor<T, I>> {
			match ActiveTournaments::<T, I>::get(season_id) {
				TournamentState::Inactive => None,
				TournamentState::ActivePeriod(tournament_id) |
				TournamentState::ClaimPeriod(tournament_id, _) =>
					Tournaments::<T, I>::get(season_id, tournament_id),
			}
		}

		fn is_golden_duck_enabled_for(season_id: &T::SeasonId) -> bool {
			match ActiveTournaments::<T, I>::get(season_id) {
				TournamentState::Inactive => false,
				TournamentState::ActivePeriod(tournament_id) |
				TournamentState::ClaimPeriod(tournament_id, _) => {
					matches!(
						GoldenDucks::<T, I>::get(season_id, tournament_id),
						GoldenDuckStateFor::<T, I>::Enabled(_, _)
					)
				},
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
			let reward_table_dist = Self::ensure_valid_tournament(season_id, &config)?;

			let next_tournament_id =
				NextTournamentIds::<T, I>::mutate(season_id, |tournament_id| {
					let assigned_id = *tournament_id;
					*tournament_id = tournament_id.saturating_add(1);
					assigned_id
				});

			if let Some(reward) = config.initial_reward {
				let treasury_account = Self::tournament_treasury_account_id(*season_id);
				T::Currency::transfer(
					creator,
					&treasury_account,
					reward,
					ExistenceRequirement::KeepAlive,
				)?;
			}

			Tournaments::<T, I>::insert(season_id, next_tournament_id, config);

			if reward_table_dist < 100 {
				GoldenDucks::<T, I>::insert(
					season_id,
					next_tournament_id,
					GoldenDuckStateFor::<T, I>::Enabled((100 - reward_table_dist) as u8, None),
				);
			}

			Self::deposit_event(Event::<T, I>::TournamentCreated {
				season_id: *season_id,
				tournament_id: next_tournament_id,
			});

			Ok(next_tournament_id)
		}

		fn try_enable_tournament_processing_for_season(season_id: &T::SeasonId) -> DispatchResult {
			EnabledSeasons::<T, I>::try_mutate(|season_set| {
				season_set
					.try_insert(*season_id)
					.map(|_| ())
					.map_err(|_| Error::<T, I>::CannotEnableTournamentProcessingForSeason.into())
			})
		}

		fn try_disable_tournament_processing_for_season(season_id: &T::SeasonId) -> DispatchResult {
			ensure!(
				matches!(ActiveTournaments::<T, I>::get(season_id), TournamentState::Inactive),
				Error::<T, I>::CannotDisableTournamentProcessingForSeason
			);

			EnabledSeasons::<T, I>::mutate(|season_set| season_set.remove(season_id));

			Ok(())
		}
	}

	impl<T: Config<I>, I: 'static> TournamentRanker<T::SeasonId, T::RankedEntity, T::EntityId>
		for Pallet<T, I>
	{
		fn try_rank_entity_in_tournament_for<R>(
			season_id: &T::SeasonId,
			entity: &T::RankedEntity,
			ranker: &R,
		) -> DispatchResult
		where
			R: EntityRank<Entity = T::RankedEntity>,
		{
			let tournament_id = Self::try_get_current_tournament_id_for(season_id)?;
			let tournament_config = Self::get_active_tournament_config_for(season_id)
				.ok_or::<Error<T, I>>(Error::<T, I>::TournamentNotFound)?;

			TournamentRankings::<T, I>::mutate(season_id, tournament_id, |table| {
				if let Err(index) =
					table.binary_search_by(|other| ranker.rank_against(entity, other))
				{
					Self::try_update_rank_table(table, &tournament_config, index, entity)
				} else {
					Ok(())
				}
			})
		}

		fn try_rank_entity_for_golden_duck(
			season_id: &T::SeasonId,
			entity_id: &T::EntityId,
		) -> DispatchResult {
			let tournament_id = Self::try_get_current_tournament_id_for(season_id)?;

			GoldenDucks::<T, I>::mutate(season_id, tournament_id, |state| match state {
				GoldenDuckState::Enabled(payout_perc, None) => {
					*state = GoldenDuckState::Enabled(*payout_perc, Some(entity_id.clone()));
				},
				GoldenDuckState::Enabled(payout_perc, Some(ref entry_id))
					if entity_id < entry_id =>
				{
					*state = GoldenDuckState::Enabled(*payout_perc, Some(entity_id.clone()));
				},
				_ => {},
			});

			Ok(())
		}
	}

	impl<T: Config<I>, I: 'static>
		TournamentClaimer<T::SeasonId, AccountIdFor<T>, T::RankedEntity, T::EntityId> for Pallet<T, I>
	{
		fn try_claim_tournament_reward_for(
			season_id: &T::SeasonId,
			account: &AccountIdFor<T>,
			entity: &T::RankedEntity,
		) -> DispatchResult {
			match ActiveTournaments::<T, I>::get(season_id) {
				TournamentState::Inactive => Err(Error::<T, I>::NoActiveTournamentForSeason.into()),
				TournamentState::ActivePeriod(_) =>
					Err(Error::<T, I>::TournamentNotInClaimPeriod.into()),
				TournamentState::ClaimPeriod(tournament_id, reward_pot) => {
					if let Some(index) = TournamentRankings::<T, I>::get(season_id, tournament_id)
						.iter()
						.position(|entry| entry == entity)
					{
						let tournament_config = Tournaments::<T, I>::get(season_id, tournament_id)
							.ok_or(Error::<T, I>::TournamentNotFound)?;
						let treasury_account = Self::tournament_treasury_account_id(*season_id);

						let payout_percentage = tournament_config
							.reward_table
							.into_iter()
							.nth(index)
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

						Ok(())
					} else {
						Err(Error::<T, I>::RankingCandidateNotInWinnerTable.into())
					}
				},
			}
		}

		fn try_claim_golden_duck_for(
			season_id: &T::SeasonId,
			account: &AccountIdFor<T>,
			entity_id: &T::EntityId,
		) -> DispatchResult {
			match ActiveTournaments::<T, I>::get(season_id) {
				TournamentState::Inactive => Err(Error::<T, I>::NoActiveTournamentForSeason.into()),
				TournamentState::ActivePeriod(_) =>
					Err(Error::<T, I>::TournamentNotInClaimPeriod.into()),
				TournamentState::ClaimPeriod(tournament_id, reward_pot) =>
					match GoldenDucks::<T, I>::get(season_id, tournament_id) {
						GoldenDuckState::Enabled(payout_percentage, Some(ref winner_id))
							if winner_id == entity_id =>
						{
							let treasury_account = Self::tournament_treasury_account_id(*season_id);

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

							Ok(())
						},
						_ => Err(Error::<T, I>::GoldenDuckCandidateNotWinner.into()),
					},
			}
		}
	}
}
