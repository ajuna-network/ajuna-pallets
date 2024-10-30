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

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;
#[cfg(any(test, feature = "runtime-benchmarks"))]
mod mock;
#[cfg(test)]
mod tests;

pub mod account;
pub mod config;
pub mod traits;
pub mod weights;

use frame_support::{pallet_prelude::*, PalletId};
use frame_system::pallet_prelude::*;

use crate::weights::WeightInfo;

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
	use ajuna_primitives::{account_manager::AccountManager, asset_manager::AssetManager};
	use frame_support::traits::{Currency, ExistenceRequirement};
	use sp_runtime::{
		traits::{AccountIdConversion, CheckedDiv, SaturatedConversion},
		Saturating,
	};

	pub type AccountIdFor<T> = <T as frame_system::Config>::AccountId;
	pub(crate) type BalanceOf<T, I> =
		<<T as Config<I>>::Currency as Currency<AccountIdFor<T>>>::Balance;
	pub(crate) type TournamentScheduledActionFor<T, I> =
		TournamentScheduledAction<<T as Config<I>>::TournamentCategoryId>;
	pub(crate) type EntityRankerFor<T, I> = <T as Config<I>>::EntityRanker;
	pub(crate) type EntityIdFor<T, I> = <T as Config<I>>::EntityId;
	pub type TournamentConfigFor<T, I> =
		TournamentConfig<BlockNumberFor<T>, BalanceOf<T, I>, EntityRankerFor<T, I>>;
	pub type TournamentCategoryIdFor<T, I> = <T as Config<I>>::TournamentCategoryId;
	pub(crate) type RankingTableFor<T, I> =
		RankingTable<(<T as Config<I>>::EntityId, <T as Config<I>>::RankedEntity)>;
	pub(crate) type RewardClaimStateFor<T> = RewardClaimState<AccountIdFor<T>>;
	pub(crate) type TournamentStateFor<T, I> = TournamentState<BalanceOf<T, I>>;
	pub(crate) type GoldenDuckStateFor<T, I> = GoldenDuckState<<T as Config<I>>::EntityId>;

	#[cfg(feature = "runtime-benchmarks")]
	pub trait BenchmarkHelper<CategoryId, BlockNumber, Balance, Ranker> {
		fn create_category_id(id: u32) -> CategoryId;

		fn create_default_tournament_config() -> TournamentConfig<BlockNumber, Balance, Ranker>;
	}

	#[cfg(feature = "runtime-benchmarks")]
	impl<CategoryId: From<u32>, BlockNumber: From<u64>, Balance: From<u64>, Ranker: Default>
		BenchmarkHelper<CategoryId, BlockNumber, Balance, Ranker> for ()
	{
		fn create_category_id(id: u32) -> CategoryId {
			id.into()
		}

		fn create_default_tournament_config() -> TournamentConfig<BlockNumber, Balance, Ranker> {
			TournamentConfig {
				start: 20_u64.into(),
				active_end: 40_u64.into(),
				claim_end: 50_u64.into(),
				initial_reward: None,
				max_reward: None,
				take_fee_percentage: None,
				reward_distribution: Default::default(),
				golden_duck_config: Default::default(),
				max_players: 0,
				ranker: Ranker::default(),
			}
		}
	}

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

		/// The tournament category identifier type.
		type TournamentCategoryId: Member + Parameter + MaxEncodedLen + Copy;

		/// The ranked entity identifier type.
		type EntityId: Member + Parameter + MaxEncodedLen + PartialOrd + Ord;

		/// The ranked entities type
		type RankedEntity: Member + Parameter + MaxEncodedLen;

		type EntityRanker: EntityRank<EntityId = Self::EntityId, Entity = Self::RankedEntity>
			+ Member
			+ Parameter
			+ MaxEncodedLen;

		type AccountManager: AccountManager<AccountId = AccountIdFor<Self>>;

		type AssetManager: AssetManager<
			AccountId = AccountIdFor<Self>,
			AssetId = Self::EntityId,
			Asset = Self::RankedEntity,
		>;

		/// Minimum duration of the tournament active and claim periods in blocks.
		#[pallet::constant]
		type MinimumTournamentPhaseDuration: Get<BlockNumberFor<Self>>;

		type WeightInfo: WeightInfo;

		#[cfg(feature = "runtime-benchmarks")]
		type BenchmarkHelper: BenchmarkHelper<
			Self::TournamentCategoryId,
			BlockNumberFor<Self>,
			BalanceOf<Self, I>,
			Self::EntityRanker,
		>;
	}

	#[pallet::storage]
	pub type TournamentSchedules<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, BlockNumberFor<T>, TournamentScheduledActionFor<T, I>, OptionQuery>;

	#[pallet::storage]
	#[pallet::getter(fn treasury_accounts)]
	pub type TreasuryAccountsCache<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, TournamentCategoryIdFor<T, I>, AccountIdFor<T>, OptionQuery>;

	#[pallet::storage]
	pub type NextTournamentIds<T: Config<I>, I: 'static = ()> =
		StorageMap<_, Identity, TournamentCategoryIdFor<T, I>, TournamentId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn tournaments)]
	pub type Tournaments<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
		_,
		Identity,
		TournamentCategoryIdFor<T, I>,
		Identity,
		TournamentId,
		TournamentConfigFor<T, I>,
		OptionQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn active_tournaments)]
	pub type ActiveTournaments<T: Config<I>, I: 'static = ()> = StorageMap<
		_,
		Identity,
		TournamentCategoryIdFor<T, I>,
		TournamentStateFor<T, I>,
		ValueQuery,
	>;

	#[pallet::storage]
	#[pallet::getter(fn rankings)]
	pub type TournamentRankings<T: Config<I>, I: 'static = ()> = StorageDoubleMap<
		_,
		Identity,
		TournamentCategoryIdFor<T, I>,
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
			NMapKey<Identity, TournamentCategoryIdFor<T, I>>,
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
		TournamentCategoryIdFor<T, I>,
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
		TournamentCategoryIdFor<T, I>,
		Identity,
		TournamentId,
		RewardClaimStateFor<T>,
		OptionQuery,
	>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		TournamentCreated {
			category_id: TournamentCategoryIdFor<T, I>,
			tournament_id: TournamentId,
		},
		TournamentRemoved {
			category_id: TournamentCategoryIdFor<T, I>,
			tournament_id: TournamentId,
		},
		TournamentActivePeriodStarted {
			category_id: TournamentCategoryIdFor<T, I>,
			tournament_id: TournamentId,
		},
		TournamentClaimPeriodStarted {
			category_id: TournamentCategoryIdFor<T, I>,
			tournament_id: TournamentId,
		},
		TournamentEnded {
			category_id: TournamentCategoryIdFor<T, I>,
			tournament_id: TournamentId,
		},
		EntityEnteredRanking {
			category_id: TournamentCategoryIdFor<T, I>,
			tournament_id: TournamentId,
			entity_id: T::EntityId,
			rank: Rank,
		},
		EntityBecameGoldenDuck {
			category_id: TournamentCategoryIdFor<T, I>,
			tournament_id: TournamentId,
			entity_id: T::EntityId,
		},
		RankingRewardClaimed {
			category_id: TournamentCategoryIdFor<T, I>,
			tournament_id: TournamentId,
			entity_id: T::EntityId,
			account: AccountIdFor<T>,
		},
		GoldenDuckRewardClaimed {
			category_id: TournamentCategoryIdFor<T, I>,
			tournament_id: TournamentId,
			entity_id: T::EntityId,
			account: AccountIdFor<T>,
		},
	}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		/// There's no active tournament for the selected category.
		NoActiveTournamentForCategory,
		/// The current tournament is not in its reward claim period.
		TournamentNotInClaimPeriod,
		/// The latest tournament for the selected category identifier already started,
		/// so it cannot be removed anymore.
		LatestTournamentAlreadyStarted,
		/// There's already an active tournament for the selected category.
		AnotherTournamentAlreadyActiveForCategory,
		/// Cannot find tournament data for the selected (category, tournament)
		/// identifier combination.
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
					TournamentScheduledAction::StartActivePhase(category_id, tournament_id) =>
						Self::try_start_next_tournament_for(category_id, tournament_id),
					TournamentScheduledAction::SwitchToClaimPhase(category_id, tournament_id) =>
						Self::try_switch_tournament_to_claim_period_for(category_id, tournament_id),
					TournamentScheduledAction::EndClaimPhase(category_id, tournament_id) =>
						Self::try_finish_tournament_claim_period_for(category_id, tournament_id),
				};
				weight.saturating_accrue(w);
			};

			weight
		}
	}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::create_tournament())]
		pub fn create_tournament(
			origin: OriginFor<T>,
			category_id: TournamentCategoryIdFor<T, I>,
			config: TournamentConfigFor<T, I>,
		) -> DispatchResult {
			let organizer = ensure_signed(origin)?;
			T::AccountManager::is_organizer(&organizer)?;

			let _ = Self::try_create_new_tournament_for(&organizer, &category_id, config)?;

			Ok(())
		}

		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::remove_latest_tournament())]
		pub fn remove_latest_tournament(
			origin: OriginFor<T>,
			category_id: TournamentCategoryIdFor<T, I>,
		) -> DispatchResult {
			let organizer = ensure_signed(origin)?;
			T::AccountManager::is_organizer(&organizer)?;

			Self::try_remove_latest_tournament_for(&category_id)
		}

		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::claim_tournament_reward_for())]
		pub fn claim_tournament_reward_for(
			origin: OriginFor<T>,
			category_id: TournamentCategoryIdFor<T, I>,
			entity_id: EntityIdFor<T, I>,
		) -> DispatchResult {
			let account = ensure_signed(origin)?;
			T::AssetManager::ensure_ownership(&account, &entity_id)?;

			Self::try_claim_tournament_reward_for(&category_id, &account, &entity_id)
		}

		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::claim_golden_duck_for())]
		pub fn claim_golden_duck_for(
			origin: OriginFor<T>,
			category_id: TournamentCategoryIdFor<T, I>,
			entity_id: EntityIdFor<T, I>,
		) -> DispatchResult {
			let account = ensure_signed(origin)?;
			T::AssetManager::ensure_ownership(&account, &entity_id)?;

			Self::try_claim_golden_duck_for(&category_id, &account, &entity_id)
		}
	}

	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		/// The account ID of the subaccount for a given category_id/tournament_id.
		pub fn tournament_treasury_account_id(
			category_id: &TournamentCategoryIdFor<T, I>,
		) -> T::AccountId {
			if let Some(account) = TreasuryAccountsCache::<T, I>::get(category_id) {
				account
			} else {
				let account_builder =
					TournamentTreasuryAccount::<TournamentCategoryIdFor<T, I>>::new(
						T::PalletId::get(),
						*category_id,
					);
				let account: AccountIdFor<T> = account_builder.into_account_truncating();
				TreasuryAccountsCache::<T, I>::insert(category_id, account.clone());
				account
			}
		}

		fn ensure_valid_tournament(
			category_id: &TournamentCategoryIdFor<T, I>,
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
				category_id,
				NextTournamentIds::<T, I>::get(category_id).saturating_sub(1),
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
			category_id: &TournamentCategoryIdFor<T, I>,
			tournament_id: &TournamentId,
			config: &TournamentConfigFor<T, I>,
		) -> DispatchResult {
			TournamentSchedules::<T, I>::try_mutate(config.start, |action| match action {
				None => {
					*action = Some(TournamentScheduledAction::StartActivePhase(
						*category_id,
						*tournament_id,
					));
					Ok(())
				},
				Some(_) => Err(Error::<T, I>::CannotScheduleTournament),
			})?;
			TournamentSchedules::<T, I>::try_mutate(config.active_end, |action| match action {
				None => {
					*action = Some(TournamentScheduledAction::SwitchToClaimPhase(
						*category_id,
						*tournament_id,
					));
					Ok(())
				},
				Some(_) => Err(Error::<T, I>::CannotScheduleTournament),
			})?;
			TournamentSchedules::<T, I>::try_mutate(config.claim_end, |action| match action {
				None => {
					*action = Some(TournamentScheduledAction::EndClaimPhase(
						*category_id,
						*tournament_id,
					));
					Ok(())
				},
				Some(_) => Err(Error::<T, I>::CannotScheduleTournament),
			})?;

			Ok(())
		}

		fn update_tournament_rewards_storage_for(
			category_id: &TournamentCategoryIdFor<T, I>,
			tournament_id: TournamentId,
		) {
			for index in 0..TournamentRankings::<T, I>::get(category_id, tournament_id).len() {
				TournamentRewardClaims::<T, I>::insert(
					(category_id, tournament_id, index as u32),
					RewardClaimState::Unclaimed,
				);
			}

			if matches!(
				GoldenDucks::<T, I>::get(category_id, tournament_id),
				GoldenDuckState::Enabled(_, Some(_))
			) {
				GoldenDuckRewardClaims::<T, I>::insert(
					category_id,
					tournament_id,
					RewardClaimState::Unclaimed,
				);
			}
		}

		fn try_get_active_tournament_id_for(
			category_id: &TournamentCategoryIdFor<T, I>,
		) -> Result<TournamentId, DispatchError> {
			match ActiveTournaments::<T, I>::get(category_id) {
				TournamentState::ActivePeriod(tournament_id) |
				TournamentState::ClaimPeriod(tournament_id, _) => Ok(tournament_id),
				_ => Err(Error::<T, I>::NoActiveTournamentForCategory.into()),
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
			category_id: TournamentCategoryIdFor<T, I>,
			tournament_id: TournamentId,
		) -> Weight {
			if let Some(tournament_config) = Tournaments::<T, I>::get(category_id, tournament_id) {
				let current_block = <frame_system::Pallet<T>>::block_number();

				if tournament_config.start > current_block {
					log::error!(target: LOG_TARGET, "Tried to start a tournament in the incorrect block!");
					return T::DbWeight::get().reads(1)
				}

				ActiveTournaments::<T, I>::insert(
					category_id,
					TournamentState::ActivePeriod(tournament_id),
				);

				Self::deposit_event(Event::<T, I>::TournamentActivePeriodStarted {
					category_id,
					tournament_id,
				});

				T::DbWeight::get().reads_writes(1, 1)
			} else {
				log::error!(target: LOG_TARGET, "Tried to start a tournament with missing config!");
				T::DbWeight::get().reads(1)
			}
		}

		fn try_switch_tournament_to_claim_period_for(
			category_id: TournamentCategoryIdFor<T, I>,
			tournament_id: TournamentId,
		) -> Weight {
			let current_block = <frame_system::Pallet<T>>::block_number();

			if let Some(tournament_config) = Tournaments::<T, I>::get(category_id, tournament_id) {
				if tournament_config.active_end > current_block {
					log::error!(target: LOG_TARGET, "Tried to switch to claim a tournament in the incorrect block!");
					return T::DbWeight::get().reads(1)
				}

				let treasury_account = Self::tournament_treasury_account_id(&category_id);
				let reward_pot = Self::get_reward_payout(&tournament_config, &treasury_account);
				ActiveTournaments::<T, I>::mutate(category_id, |state| {
					*state = TournamentState::ClaimPeriod(tournament_id, reward_pot)
				});

				Self::update_tournament_rewards_storage_for(&category_id, tournament_id);

				Self::deposit_event(Event::<T, I>::TournamentClaimPeriodStarted {
					category_id,
					tournament_id,
				});

				T::DbWeight::get().reads_writes(1, 1)
			} else {
				log::error!(target: LOG_TARGET, "Tried to switch a tournament to claim phase with missing config!");
				T::DbWeight::get().reads(1)
			}
		}

		fn try_finish_tournament_claim_period_for(
			category_id: TournamentCategoryIdFor<T, I>,
			tournament_id: TournamentId,
		) -> Weight {
			let current_block = <frame_system::Pallet<T>>::block_number();

			if let Some(tournament_config) = Tournaments::<T, I>::get(category_id, tournament_id) {
				if tournament_config.claim_end > current_block {
					log::error!(target: LOG_TARGET, "Tried to finish a tournament in the incorrect block!");
					return T::DbWeight::get().reads(1)
				}

				ActiveTournaments::<T, I>::mutate(category_id, |state| {
					*state = TournamentState::Finished(tournament_id)
				});

				Self::deposit_event(Event::<T, I>::TournamentEnded { category_id, tournament_id });

				T::DbWeight::get().reads_writes(1, 2)
			} else {
				log::error!(target: LOG_TARGET, "Tried to finish a tournament with missing config!");
				T::DbWeight::get().reads(1)
			}
		}
	}

	impl<T: Config<I>, I: 'static>
		TournamentInspector<
			TournamentCategoryIdFor<T, I>,
			BlockNumberFor<T>,
			BalanceOf<T, I>,
			AccountIdFor<T>,
			EntityRankerFor<T, I>,
		> for Pallet<T, I>
	{
		fn get_active_tournament_config_for(
			category_id: &TournamentCategoryIdFor<T, I>,
		) -> Option<(TournamentId, TournamentConfigFor<T, I>)> {
			match ActiveTournaments::<T, I>::get(category_id) {
				TournamentState::ActivePeriod(tournament_id) |
				TournamentState::ClaimPeriod(tournament_id, _) =>
					if let Some(tournament_config) =
						Tournaments::<T, I>::get(category_id, tournament_id)
					{
						Some((tournament_id, tournament_config))
					} else {
						log::error!(target: LOG_TARGET, "No tournament config found for active tournament!");
						None
					},
				_ => None,
			}
		}

		fn get_active_tournament_state_for(
			category_id: &TournamentCategoryIdFor<T, I>,
		) -> TournamentStateFor<T, I> {
			ActiveTournaments::<T, I>::get(category_id)
		}

		fn is_golden_duck_enabled_for(category_id: &TournamentCategoryIdFor<T, I>) -> bool {
			match ActiveTournaments::<T, I>::get(category_id) {
				TournamentState::ActivePeriod(tournament_id) |
				TournamentState::ClaimPeriod(tournament_id, _) => {
					matches!(
						GoldenDucks::<T, I>::get(category_id, tournament_id),
						GoldenDuckStateFor::<T, I>::Enabled(_, _)
					)
				},
				_ => false,
			}
		}

		fn get_treasury_account_for(
			category_id: &TournamentCategoryIdFor<T, I>,
		) -> AccountIdFor<T> {
			Self::tournament_treasury_account_id(category_id)
		}
	}

	impl<T: Config<I>, I: 'static>
		TournamentMutator<
			AccountIdFor<T>,
			TournamentCategoryIdFor<T, I>,
			BlockNumberFor<T>,
			BalanceOf<T, I>,
			EntityRankerFor<T, I>,
		> for Pallet<T, I>
	{
		fn try_create_new_tournament_for(
			creator: &AccountIdFor<T>,
			category_id: &TournamentCategoryIdFor<T, I>,
			config: TournamentConfigFor<T, I>,
		) -> Result<TournamentId, DispatchError> {
			Self::ensure_valid_tournament(category_id, &config)?;

			let next_tournament_id =
				NextTournamentIds::<T, I>::mutate(category_id, |tournament_id| {
					let assigned_id = *tournament_id;
					*tournament_id = tournament_id.saturating_add(1);
					assigned_id
				});

			Self::try_insert_tournament_schedule(category_id, &next_tournament_id, &config)?;

			if let Some(reward) = config.initial_reward {
				let treasury_account = Self::tournament_treasury_account_id(category_id);
				T::Currency::transfer(
					creator,
					&treasury_account,
					reward,
					ExistenceRequirement::KeepAlive,
				)?;
			}

			if let GoldenDuckConfig::Enabled(percentage) = config.golden_duck_config {
				GoldenDucks::<T, I>::insert(
					category_id,
					next_tournament_id,
					GoldenDuckStateFor::<T, I>::Enabled(percentage, None),
				);
			}

			Tournaments::<T, I>::insert(category_id, next_tournament_id, config);

			Self::deposit_event(Event::<T, I>::TournamentCreated {
				category_id: *category_id,
				tournament_id: next_tournament_id,
			});

			Ok(next_tournament_id)
		}

		fn try_remove_latest_tournament_for(
			category_id: &TournamentCategoryIdFor<T, I>,
		) -> DispatchResult {
			match Self::get_active_tournament_state_for(category_id) {
				TournamentState::Inactive =>
					NextTournamentIds::<T, I>::try_mutate(category_id, |tournament_id| {
						let prev_id = tournament_id.saturating_sub(1);
						if let Some(config) = Tournaments::<T, I>::take(category_id, prev_id) {
							GoldenDucks::<T, I>::remove(category_id, prev_id);

							TournamentSchedules::<T, I>::remove(config.start);
							TournamentSchedules::<T, I>::remove(config.active_end);
							TournamentSchedules::<T, I>::remove(config.claim_end);

							*tournament_id = prev_id;

							Self::deposit_event(Event::<T, I>::TournamentRemoved {
								category_id: *category_id,
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

	impl<T: Config<I>, I: 'static>
		TournamentRanker<TournamentCategoryIdFor<T, I>, T::RankedEntity, T::EntityId> for Pallet<T, I>
	{
		fn try_rank_entity_in_tournament_for(
			category_id: &TournamentCategoryIdFor<T, I>,
			entity_id: &T::EntityId,
			entity: &T::RankedEntity,
		) -> DispatchResult {
			let (tournament_id, tournament_config) =
				Self::get_active_tournament_config_for(category_id)
					.ok_or::<Error<T, I>>(Error::<T, I>::NoActiveTournamentForCategory)?;

			if !tournament_config.ranker.can_rank((entity_id, entity)) {
				return Ok(());
			}

			TournamentRankings::<T, I>::mutate(category_id, tournament_id, |table| {
				match table.binary_search_by(|(other_id, other)| {
					tournament_config.ranker.rank_against((entity_id, entity), (other_id, other))
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
										category_id: *category_id,
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
			category_id: &TournamentCategoryIdFor<T, I>,
			entity_id: &T::EntityId,
		) -> DispatchResult {
			ensure!(
				matches!(
					Self::get_active_tournament_state_for(category_id),
					TournamentState::ActivePeriod(_)
				),
				Error::<T, I>::NoActiveTournamentForCategory
			);

			let tournament_id = Self::try_get_active_tournament_id_for(category_id)?;

			GoldenDucks::<T, I>::mutate(category_id, tournament_id, |state| {
				if let GoldenDuckState::Enabled(payout_perc, ref maybe_entry_id) = state {
					match maybe_entry_id {
						None => {
							*state =
								GoldenDuckState::Enabled(*payout_perc, Some(entity_id.clone()));
							Self::deposit_event(Event::<T, I>::EntityBecameGoldenDuck {
								category_id: *category_id,
								tournament_id,
								entity_id: entity_id.clone(),
							});
						},
						Some(entry_id) if entity_id < entry_id => {
							*state =
								GoldenDuckState::Enabled(*payout_perc, Some(entity_id.clone()));
							Self::deposit_event(Event::<T, I>::EntityBecameGoldenDuck {
								category_id: *category_id,
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

	impl<T: Config<I>, I: 'static>
		TournamentClaimer<TournamentCategoryIdFor<T, I>, AccountIdFor<T>, T::EntityId> for Pallet<T, I>
	{
		fn try_claim_tournament_reward_for(
			category_id: &TournamentCategoryIdFor<T, I>,
			account: &AccountIdFor<T>,
			entity_id: &T::EntityId,
		) -> DispatchResult {
			ensure!(
				matches!(
					Self::get_active_tournament_state_for(category_id),
					TournamentState::ClaimPeriod(_, _)
				),
				Error::<T, I>::TournamentNotInClaimPeriod
			);

			match ActiveTournaments::<T, I>::get(category_id) {
				TournamentState::ActivePeriod(_) =>
					Err(Error::<T, I>::TournamentNotInClaimPeriod.into()),
				TournamentState::ClaimPeriod(tournament_id, reward_pot) => {
					let index = TournamentRankings::<T, I>::get(category_id, tournament_id)
						.iter()
						.position(|(entry_id, _)| entry_id == entity_id)
						.ok_or(Error::<T, I>::RankingCandidateNotInWinnerTable)?;

					TournamentRewardClaims::<T, I>::try_mutate(
						(category_id, tournament_id, index as u32),
						|state| {
							ensure!(
								matches!(state, Some(RewardClaimState::Unclaimed)),
								Error::<T, I>::TournamentRewardAlreadyClaimed
							);

							let tournament_config =
								Tournaments::<T, I>::get(category_id, tournament_id)
									.ok_or(Error::<T, I>::TournamentNotFound)?;
							let treasury_account =
								Self::tournament_treasury_account_id(category_id);

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
								category_id: *category_id,
								tournament_id,
								entity_id: entity_id.clone(),
								account: account.clone(),
							});

							Ok(())
						},
					)
				},
				_ => Err(Error::<T, I>::NoActiveTournamentForCategory.into()),
			}
		}

		fn try_claim_golden_duck_for(
			category_id: &TournamentCategoryIdFor<T, I>,
			account: &AccountIdFor<T>,
			entity_id: &T::EntityId,
		) -> DispatchResult {
			ensure!(
				matches!(
					Self::get_active_tournament_state_for(category_id),
					TournamentState::ClaimPeriod(_, _)
				),
				Error::<T, I>::TournamentNotInClaimPeriod
			);

			match ActiveTournaments::<T, I>::get(category_id) {
				TournamentState::ActivePeriod(_) =>
					Err(Error::<T, I>::TournamentNotInClaimPeriod.into()),
				TournamentState::ClaimPeriod(tournament_id, reward_pot) =>
					match GoldenDucks::<T, I>::get(category_id, tournament_id) {
						GoldenDuckState::Enabled(payout_percentage, Some(ref winner_id))
							if winner_id == entity_id =>
							GoldenDuckRewardClaims::<T, I>::try_mutate(
								category_id,
								tournament_id,
								|state| {
									ensure!(
										matches!(state, Some(RewardClaimState::Unclaimed)),
										Error::<T, I>::TournamentRewardAlreadyClaimed
									);

									let treasury_account =
										Self::tournament_treasury_account_id(category_id);

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
										category_id: *category_id,
										tournament_id,
										entity_id: entity_id.clone(),
										account: account.clone(),
									});

									Ok(())
								},
							),
						_ => Err(Error::<T, I>::GoldenDuckCandidateNotWinner.into()),
					},
				_ => Err(Error::<T, I>::NoActiveTournamentForCategory.into()),
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
