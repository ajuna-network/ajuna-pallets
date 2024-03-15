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
		Saturating,
	};

	pub type AccountIdFor<T> = <T as frame_system::Config>::AccountId;
	pub(crate) type BalanceOf<T, I> =
		<<T as Config<I>>::Currency as Currency<AccountIdFor<T>>>::Balance;
	pub type TournamentConfigFor<T, I> = TournamentConfig<BlockNumberFor<T>, BalanceOf<T, I>>;
	pub type PlayerTableFor<T, I> = PlayerTable<(AccountIdFor<T>, <T as Config<I>>::RankedEntity)>;
	pub type GoldenDuckStateFor<T, I> =
		GoldenDuckState<AccountIdFor<T>, <T as Config<I>>::EntityId>;

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

		/// The amount needed to rank an avatar
		#[pallet::constant]
		type RankDeposit: Get<BalanceOf<Self, I>>;

		/// The season identifier type.
		type SeasonId: Member + Parameter + MaxEncodedLen + AtLeast32BitUnsigned + Copy;

		/// The ranked entity identifier type.
		type EntityId: Member + Parameter + MaxEncodedLen + PartialOrd + Ord;

		/// The ranked entities type
		type RankedEntity: Member + Parameter + MaxEncodedLen + PartialOrd + Ord;

		/// Specific category in a tournament an entity can be ranked into.
		type RankCategory: Member + Parameter + MaxEncodedLen + Copy;
	}

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
	pub type TournamentTreasuries<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
		_,
		Identity,
		T::SeasonId,
		Identity,
		TournamentId,
		AccountIdFor<T>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn active_tournaments)]
	pub type ActiveTournaments<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, T::SeasonId, TournamentId, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn latest_tournaments)]
	pub type LatestTournaments<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, T::SeasonId, TournamentId, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn rankings)]
	pub type TournamentRankings<T: Config<I>, I: 'static = ()> = StorageNMap<
		_,
		(
			NMapKey<Identity, T::SeasonId>,
			NMapKey<Identity, TournamentId>,
			NMapKey<Identity, T::RankCategory>,
		),
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
		TournamentStarted { season_id: T::SeasonId, tournament_id: TournamentId },
		TournamentEnded { season_id: T::SeasonId, tournament_id: TournamentId },
	}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		/// There's no active tournament for the selected season.
		NoActiveTournamentForSeason,
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
	}

	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		/// The account ID of the treasury.
		pub fn global_treasury_account_id() -> AccountIdFor<T> {
			T::PalletId::get().into_account_truncating()
		}

		/// The account ID of the sub-account for a given season_id/tournament_id.
		pub fn tournament_treasury_account_id(
			season_id: T::SeasonId,
			tournament_id: TournamentId,
		) -> T::AccountId {
			let account = TournamentTreasuryAccount::<T::SeasonId>::new(
				T::PalletId::get(),
				season_id,
				tournament_id,
			);
			account.into_account_truncating()
		}

		fn ensure_valid_tournament(
			config: &TournamentConfigFor<T, I>,
		) -> Result<u16, DispatchError> {
			ensure!(config.start < config.end, Error::<T, I>::InvalidTournamentConfig);

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
			let maybe_id = ActiveTournaments::<T, I>::get(season_id);
			ensure!(maybe_id.is_some(), Error::<T, I>::NoActiveTournamentForSeason);
			Ok(maybe_id.unwrap())
		}

		fn process_rank_fee_deposit(
			payer: &AccountIdFor<T>,
			fee_percentage: u8,
		) -> Result<BalanceOf<T, I>, DispatchError> {
			let base_deposit = T::RankDeposit::get();
			let fee_deposit = base_deposit
				.saturating_mul(fee_percentage.into())
				.checked_div(&100_u32.into())
				.unwrap_or_default();
			let global_treasury_account = Self::global_treasury_account_id();

			T::Currency::transfer(
				payer,
				&global_treasury_account,
				fee_deposit,
				ExistenceRequirement::KeepAlive,
			)?;

			Ok(base_deposit.saturating_sub(fee_deposit))
		}

		fn process_max_reward_payout(
			payer: &AccountIdFor<T>,
			tournament_treasury_account: &AccountIdFor<T>,
			rank_deposit: BalanceOf<T, I>,
			max_reward: BalanceOf<T, I>,
		) -> DispatchResult {
			let tournament_treasury_balance =
				T::Currency::free_balance(tournament_treasury_account);
			let tournament_deposit_left = max_reward.saturating_sub(tournament_treasury_balance);
			let effective_tournament_deposit =
				sp_std::cmp::min(tournament_deposit_left, rank_deposit);

			if effective_tournament_deposit > 0_u32.into() {
				T::Currency::transfer(
					payer,
					tournament_treasury_account,
					effective_tournament_deposit,
					ExistenceRequirement::KeepAlive,
				)?;
			}

			if effective_tournament_deposit < rank_deposit {
				let effective_global_deposit =
					rank_deposit.saturating_sub(effective_tournament_deposit);
				let global_treasury_account = Self::global_treasury_account_id();

				T::Currency::transfer(
					payer,
					&global_treasury_account,
					effective_global_deposit,
					ExistenceRequirement::KeepAlive,
				)?;
			}

			Ok(())
		}

		fn process_category_payout(
			treasury_account: &AccountIdFor<T>,
			_category: &T::RankCategory,
			tournament_config: &TournamentConfigFor<T, I>,
			rank_table: &PlayerTableFor<T, I>,
		) -> DispatchResult {
			let total_payout = T::Currency::free_balance(treasury_account);

			for ((account, _), payout_perc) in rank_table
				.iter()
				.zip(&tournament_config.reward_table)
				.filter(|(_, perc)| **perc > 0)
			{
				let account_payout = total_payout
					.saturating_mul((*payout_perc).into())
					.checked_div(&100_u32.into())
					.unwrap_or_default();

				if account_payout > 0_u32.into() {
					T::Currency::transfer(
						treasury_account,
						account,
						account_payout,
						ExistenceRequirement::AllowDeath,
					)?;
				}
			}

			match tournament_config.reward_table.last() {
				Some(perc) if *perc > 0 => {
					// TODO: Golden duck logic would go here
					// Remaining payout to be sent to golden duck account
				},
				_ => {},
			}

			Ok(())
		}
	}

	impl<T: Config<I>, I: 'static>
		TournamentInspector<T::SeasonId, BlockNumberFor<T>, BalanceOf<T, I>> for Pallet<T, I>
	{
		fn get_active_tournament_for(season_id: &T::SeasonId) -> Option<TournamentConfigFor<T, I>> {
			ActiveTournaments::<T, I>::get(season_id)
				.map(|tournament_id| Tournaments::<T, I>::get(season_id, tournament_id))
		}

		fn is_golden_duck_enabled_for(season_id: &T::SeasonId) -> bool {
			ActiveTournaments::<T, I>::get(season_id)
				.map(|tournament_id| {
					matches!(
						GoldenDucks::<T, I>::get(season_id, tournament_id),
						GoldenDuckState::Enabled(_)
					)
				})
				.unwrap_or(false)
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
			let reward_table_dist = Self::ensure_valid_tournament(&config)?;

			let next_tournament_id =
				NextTournamentIds::<T, I>::mutate(season_id, |tournament_id| {
					let assigned_id = tournament_id.saturating_add(1);
					*tournament_id = assigned_id;
					assigned_id
				});

			if let Some(reward) = config.initial_reward {
				let treasury_account =
					Self::tournament_treasury_account_id(*season_id, next_tournament_id);
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
					GoldenDuckStateFor::<T, I>::Enabled(None),
				);
			}

			Self::deposit_event(Event::<T, I>::TournamentCreated {
				season_id: *season_id,
				tournament_id: next_tournament_id,
			});

			Ok(next_tournament_id)
		}

		fn try_start_next_tournament_for(season_id: &T::SeasonId) -> DispatchResult {
			ensure!(
				ActiveTournaments::<T, I>::get(season_id).is_none(),
				Error::<T, I>::AnotherTournamentAlreadyActiveForSeason
			);

			let next_tournament_id = LatestTournaments::<T, I>::get(season_id).unwrap_or(1);
			let current_block = <frame_system::Pallet<T>>::block_number();

			let tournament_config = Tournaments::<T, I>::get(season_id, next_tournament_id)
				.ok_or(Error::<T, I>::TournamentNotFound)?;

			if tournament_config.start >= current_block {
				Err(Error::<T, I>::TournamentActivationTooEarly.into());
			}

			ActiveTournaments::<T, I>::insert(season_id, next_tournament_id);

			Self::deposit_event(Event::<T, I>::TournamentStarted {
				season_id: *season_id,
				tournament_id: next_tournament_id,
			});

			Ok(())
		}

		fn try_finish_active_tournament_for(season_id: &T::SeasonId) -> DispatchResult {
			let tournament_id = Self::try_get_current_tournament_id_for(season_id)?;
			let current_block = <frame_system::Pallet<T>>::block_number();

			let tournament_config = Tournaments::<T, I>::get(season_id, tournament_id)
				.ok_or(Error::<T, I>::TournamentNotFound)?;

			if tournament_config.end <= current_block {
				Err(Error::<T, I>::TournamentEndingTooEarly.into());
			}

			let treasury_account = Self::tournament_treasury_account_id(*season_id, tournament_id);

			for (category, player_table) in
				TournamentRankings::<T, I>::iter_prefix((season_id, tournament_id))
			{
				Self::process_category_payout(
					&treasury_account,
					&category,
					&tournament_config,
					&player_table,
				)?;
			}

			if let GoldenDuckStateFor::<T, I>::Enabled(Some((golden_account, _))) =
				GoldenDucks::<T, I>::get(season_id, tournament_id)
			{
				let golden_duck_balance = T::Currency::free_balance(&treasury_account);
				T::Currency::transfer(
					&treasury_account,
					&golden_account,
					golden_duck_balance,
					ExistenceRequirement::AllowDeath,
				)?;
			}

			ActiveTournaments::<T, I>::remove(season_id);
			LatestTournaments::<T, I>::insert(season_id, tournament_id);

			Self::deposit_event(Event::<T, I>::TournamentEnded {
				season_id: *season_id,
				tournament_id,
			});

			Ok(())
		}
	}

	impl<T: Config<I>, I: 'static>
		TournamentRanker<
			AccountIdFor<T>,
			T::SeasonId,
			T::RankCategory,
			T::RankedEntity,
			T::EntityId,
		> for Pallet<T, I>
	{
		fn try_rank_entity_in_tournament_for<R>(
			account: &AccountIdFor<T>,
			season_id: &T::SeasonId,
			category: &T::RankCategory,
			entity: &T::RankedEntity,
			ranker: &R,
		) -> DispatchResult
		where
			R: EntityRank<Entity = T::RankedEntity>,
		{
			let tournament_id = Self::try_get_current_tournament_id_for(season_id)?;
			let tournament_config = Self::get_active_tournament_for(season_id)
				.ok_or::<Error<T, I>>(Error::<T, I>::TournamentNotFound)?;
			let treasury_account = Self::tournament_treasury_account_id(*season_id, tournament_id);

			let rank_deposit =
				if let Some(take_fee_percentage) = tournament_config.take_fee_percentage {
					Self::process_rank_fee_deposit(account, take_fee_percentage)?
				} else {
					T::RankDeposit::get()
				};

			if let Some(max_reward) = tournament_config.max_reward {
				Self::process_max_reward_payout(
					account,
					&treasury_account,
					rank_deposit,
					max_reward,
				)?;
			} else {
				T::Currency::transfer(
					account,
					&treasury_account,
					rank_deposit,
					ExistenceRequirement::KeepAlive,
				)?;
			}

			TournamentRankings::<T, I>::mutate((season_id, tournament_id, category), |table| {
				if let Err(index) =
					table.binary_search_by(|(_, other)| ranker.rank_against(entity, other))
				{
					if tournament_config.max_players == MAX_PLAYERS {
						if index < PlayerTable::<PlayerTableFor<T, I>>::bound() {
							table.force_insert_keep_left(index, (account.clone(), entity.clone()))
						} else {
							Ok(None)
						}
					} else if index < tournament_config.max_players as usize {
						if table.len() == tournament_config.max_players as usize {
							let _ = table.pop();
						}
						table.force_insert_keep_left(index, (account.clone(), entity.clone()))
					} else {
						Ok(None)
					}
				} else {
					Ok(None)
				}
			})
			.map(|_| ())
			.map_err(|_| Error::<T, I>::FailedToRankEntity.into())
		}

		fn try_rank_entity_for_golden_duck(
			account: &AccountIdFor<T>,
			season_id: &T::SeasonId,
			entity_id: &T::EntityId,
		) -> DispatchResult {
			let tournament_id = Self::try_get_current_tournament_id_for(season_id)?;

			GoldenDucks::<T, I>::mutate(season_id, tournament_id, |state| match state {
				GoldenDuckState::Enabled(None) => {
					*state = GoldenDuckState::Enabled(Some((account.clone(), entity_id.clone())));
				},
				GoldenDuckState::Enabled(Some((_, ref entry_id))) if entity_id < entry_id => {
					*state = GoldenDuckState::Enabled(Some((account.clone(), entity_id.clone())));
				},
				_ => {},
			});

			Ok(())
		}
	}
}
