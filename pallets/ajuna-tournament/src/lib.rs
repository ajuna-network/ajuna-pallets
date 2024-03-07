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

use traits::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::traits::Currency;

	pub type AccountIdFor<T> = <T as frame_system::Config>::AccountId;
	pub(crate) type BalanceOf<T, I> =
		<<T as Config<I>>::Currency as Currency<AccountIdFor<T>>>::Balance;
	pub type TournamentConfigFor<T, I> = TournamentConfig<BlockNumberFor<T>, BalanceOf<T, I>>;

	/// The current storage version.
	const STORAGE_VERSION: StorageVersion = StorageVersion::new(1);

	#[pallet::pallet]
	#[pallet::storage_version(STORAGE_VERSION)]
	pub struct Pallet<T, I = ()>(PhantomData<(T, I)>);

	#[pallet::config]
	pub trait Config<I: 'static = ()>: frame_system::Config {
		/// The overarching event type.
		type RuntimeEvent: From<Event<Self, I>>
			+ IsType<<Self as frame_system::Config>::RuntimeEvent>;

		type Currency: Currency<Self::AccountId>;

		/// The season identifier type.
		type SeasonId: Member + Parameter + MaxEncodedLen + Copy;

		/// The ranked entities identifier
		type EntityId: Member + Parameter + MaxEncodedLen + Copy;

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
		Twox128,
		T::SeasonId,
		Twox128,
		TournamentId,
		TournamentConfigFor<T, I>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn active_tournaments)]
	pub type ActiveTournaments<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, T::SeasonId, TournamentId, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn rankings)]
	pub type TournamentRankings<T: Config<I>, I: 'static = ()> = StorageNMap<
		_,
		(
			NMapKey<Twox128, T::SeasonId>,
			NMapKey<Twox128, TournamentId>,
			NMapKey<Twox128, T::RankCategory>,
		),
		PlayerTable<T::EntityId>,
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
	}

	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		fn ensure_valid_tournament(config: &TournamentConfigFor<T, I>) -> DispatchResult {
			Ok(())
		}
	}

	impl<T: Config<I>, I: 'static>
		TournamentInspector<T::SeasonId, BlockNumberFor<T>, BalanceOf<T, I>> for Pallet<T, I>
	{
		fn get_active_tournament_for(season_id: &T::SeasonId) -> Option<TournamentConfigFor<T, I>> {
			if let Some(tournament_id) = ActiveTournaments::<T, I>::get(season_id) {
				Tournaments::<T, I>::get(season_id, tournament_id)
			} else {
				None
			}
		}
	}

	impl<T: Config<I>, I: 'static>
		TournamentMutator<T::SeasonId, BlockNumberFor<T>, BalanceOf<T, I>> for Pallet<T, I>
	{
		fn try_create_new_tournament_for(
			season_id: &T::SeasonId,
			config: TournamentConfigFor<T, I>,
		) -> Result<TournamentId, DispatchError> {
			Self::ensure_valid_tournament(&config)?;

			let next_tournament_id =
				NextTournamentIds::<T, I>::mutate(season_id, |tournament_id| {
					let assigned_id = tournament_id.saturating_add(1);
					*tournament_id = assigned_id;
					assigned_id
				});

			Tournaments::<T, I>::insert(season_id, next_tournament_id, config);

			Self::deposit_event(Event::<T, I>::TournamentCreated {
				season_id: season_id.clone(),
				tournament_id: next_tournament_id,
			});

			Ok(next_tournament_id)
		}

		fn try_start_next_tournament_for(season_id: &T::SeasonId) -> DispatchResult {
			ensure!(
				ActiveTournaments::<T, I>::get(season_id).is_none(),
				Error::<T, I>::AnotherTournamentAlreadyActiveForSeason
			);

			let next_tournament_id = NextTournamentIds::<T, I>::get(season_id);
			let current_block = <frame_system::Pallet<T>>::block_number();

			if let Some(tournament_config) = Tournaments::<T, I>::get(season_id, next_tournament_id)
			{
				if tournament_config.start >= current_block {
					// TODO: Tournament start logic

					Self::deposit_event(Event::<T, I>::TournamentStarted {
						season_id: season_id.clone(),
						tournament_id: next_tournament_id,
					});

					Ok(())
				} else {
					Err(Error::<T, I>::TournamentActivationTooEarly.into())
				}
			} else {
				Err(Error::<T, I>::TournamentNotFound.into())
			}
		}

		fn try_finish_active_tournament_for(season_id: &T::SeasonId) -> DispatchResult {
			let tournament_id = {
				let maybe_id = ActiveTournaments::<T, I>::take(season_id);
				ensure!(maybe_id.is_some(), Error::<T, I>::NoActiveTournamentForSeason);
				maybe_id.unwrap()
			};
			let current_block = <frame_system::Pallet<T>>::block_number();

			if let Some(tournament_config) = Tournaments::<T, I>::get(season_id, tournament_id) {
				if tournament_config.end <= current_block {
					// TODO: Tournament ending logic

					Self::deposit_event(Event::<T, I>::TournamentEnded {
						season_id: season_id.clone(),
						tournament_id,
					});

					Ok(())
				} else {
					Err(Error::<T, I>::TournamentEndingTooEarly.into())
				}
			} else {
				Err(Error::<T, I>::TournamentNotFound.into())
			}
		}
	}

	impl<T: Config<I>, I: 'static, E>
		TournamentRanker<T::SeasonId, AccountIdFor<T>, T::RankCategory, E, T::EntityId> for Pallet<T, I>
	{
		fn try_rank_entity_in_tournament_for<R>(
			season_id: &T::SeasonId,
			account: &AccountIdFor<T>,
			ranker: &R,
			entity: &E,
		) -> DispatchResult
		where
			R: Ranker<T::EntityId, Category = T::RankCategory, Entity = E>,
		{
			// TODO: Ranking logic
			Ok(())
		}
	}
}
