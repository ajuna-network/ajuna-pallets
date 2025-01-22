use crate::{
	pallet::{EntityIdFor, GoldenDuckStateFor, TournamentStateFor},
	AccountIdFor, ActiveTournaments, Config, Error, Event, GoldenDuckConfig,
	GoldenDuckRewardClaims, GoldenDuckState, GoldenDucks, NextTournamentIds, Pallet, RankingResult,
	RewardClaimState, TournamentCategoryIdFor, TournamentConfigFor, TournamentId,
	TournamentRankings, TournamentRewardClaims, TournamentSchedules, TournamentState, Tournaments,
	LOG_TARGET,
};

use ajuna_primitives::tournament_manager::{
	EntityRanker, TournamentClaimer, TournamentInspector, TournamentMutator, TournamentRanker,
};

use frame_support::{
	dispatch::DispatchResult,
	ensure,
	traits::{Currency, ExistenceRequirement},
};
use sp_arithmetic::traits::{CheckedDiv, Saturating};
use sp_runtime::DispatchError;

impl<T: Config<I>, I: 'static> TournamentInspector for Pallet<T, I> {
	type CategoryId = TournamentCategoryIdFor<T, I>;
	type TournamentId = TournamentId;
	type TournamentConfig = TournamentConfigFor<T, I>;
	type TournamentState = TournamentStateFor<T, I>;
	type AccountId = AccountIdFor<T>;
	fn get_active_tournament_config_for(
		category_id: &Self::CategoryId,
	) -> Option<(Self::TournamentId, Self::TournamentConfig)> {
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

	fn get_active_tournament_state_for(category_id: &Self::CategoryId) -> Self::TournamentState {
		ActiveTournaments::<T, I>::get(category_id)
	}

	fn is_golden_duck_enabled_for(category_id: &Self::CategoryId) -> bool {
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

	fn get_treasury_account_for(category_id: &Self::CategoryId) -> Self::AccountId {
		Self::tournament_treasury_account_id(category_id)
	}
}

impl<T: Config<I>, I: 'static> TournamentMutator for Pallet<T, I> {
	fn try_create_new_tournament_for(
		creator: &Self::AccountId,
		category_id: &Self::CategoryId,
		config: Self::TournamentConfig,
	) -> Result<Self::TournamentId, DispatchError> {
		Self::ensure_valid_tournament(category_id, &config)?;

		let next_tournament_id = NextTournamentIds::<T, I>::mutate(category_id, |tournament_id| {
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

	fn try_remove_latest_tournament_for(category_id: &Self::CategoryId) -> DispatchResult {
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
			_ => Err(Error::<T, I>::CannotRemoveActiveTournament.into()),
		}
	}
}

impl<T: Config<I>, I: 'static> TournamentRanker for Pallet<T, I> {
	type EntityId = EntityIdFor<T, I>;
	type Entity = T::RankedEntity;

	fn try_rank_entity_in_tournament_for(
		category_id: &Self::CategoryId,
		entity_id: &Self::EntityId,
		entity: &Self::Entity,
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
		category_id: &Self::CategoryId,
		entity_id: &Self::EntityId,
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
						*state = GoldenDuckState::Enabled(*payout_perc, Some(entity_id.clone()));
						Self::deposit_event(Event::<T, I>::EntityBecameGoldenDuck {
							category_id: *category_id,
							tournament_id,
							entity_id: entity_id.clone(),
						});
					},
					Some(entry_id) if entity_id < entry_id => {
						*state = GoldenDuckState::Enabled(*payout_perc, Some(entity_id.clone()));
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

impl<T: Config<I>, I: 'static> TournamentClaimer for Pallet<T, I> {
	fn try_claim_tournament_reward_for(
		category_id: &Self::CategoryId,
		account: &Self::AccountId,
		entity_id: &Self::EntityId,
	) -> DispatchResult {
		match ActiveTournaments::<T, I>::get(category_id) {
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
						let treasury_account = Self::tournament_treasury_account_id(category_id);

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
			_ => Err(Error::<T, I>::TournamentNotInClaimPeriod.into()),
		}
	}

	fn try_claim_golden_duck_for(
		category_id: &Self::CategoryId,
		account: &Self::AccountId,
		entity_id: &Self::EntityId,
	) -> DispatchResult {
		match ActiveTournaments::<T, I>::get(category_id) {
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
			_ => Err(Error::<T, I>::TournamentNotInClaimPeriod.into()),
		}
	}
}
