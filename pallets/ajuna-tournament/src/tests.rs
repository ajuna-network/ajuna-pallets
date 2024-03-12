use crate::{mock::*, *};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::bounded_vec;

impl Default for TournamentConfig<BlockNumberFor<Test>, MockBalance> {
	fn default() -> Self {
		Self {
			start: 1,
			end: 10,
			initial_reward: None,
			max_reward: None,
			take_fee_percentage: None,
			reward_table: bounded_vec![50, 30, 10],
			max_players: 3,
		}
	}
}

impl TournamentConfig<BlockNumberFor<Test>, MockBalance> {
	pub(crate) fn start(mut self, start: BlockNumberFor<Test>) -> Self {
		self.start = start;
		self
	}

	pub(crate) fn end(mut self, end: BlockNumberFor<Test>) -> Self {
		self.end = end;
		self
	}

	pub(crate) fn initial_reward(mut self, initial_reward: Option<MockBalance>) -> Self {
		self.initial_reward = initial_reward;
		self
	}

	pub(crate) fn max_reward(mut self, max_reward: Option<MockBalance>) -> Self {
		self.max_reward = max_reward;
		self
	}

	pub(crate) fn take_fee_percentage(mut self, take_fee_percentage: Option<u8>) -> Self {
		self.take_fee_percentage = take_fee_percentage;
		self
	}

	pub(crate) fn reward_table(mut self, reward_table: RewardTable) -> Self {
		self.reward_table = reward_table;
		self
	}

	pub(crate) fn max_players(mut self, max_players: u32) -> Self {
		self.max_players = max_players;
		self
	}
}

mod tournament_inspector {
	use super::*;

	#[test]
	fn get_active_tournament_works() {
		let tournament_id = 3;
		let tournament_config = TournamentConfigFor::<Test, Instance1>::default();
		ExtBuilder::default().build().execute_with(|| {
			ActiveTournaments::<Test, Instance1>::insert(SEASON_ID_1, tournament_id);
			Tournaments::<Test, Instance1>::insert(
				SEASON_ID_1,
				tournament_id,
				tournament_config.clone(),
			);

			assert_eq!(
				TournamentAlpha::get_active_tournament_for(&SEASON_ID_1),
				Some(tournament_config)
			);
			assert_eq!(TournamentAlpha::get_active_tournament_for(&SEASON_ID_2), None);
		});
	}

	#[test]
	fn get_active_tournament_doesnt_apply_to_different_instance() {
		let tournament_id = 3;
		let tournament_config = TournamentConfigFor::<Test, Instance1>::default();
		ExtBuilder::default().build().execute_with(|| {
			ActiveTournaments::<Test, Instance1>::insert(SEASON_ID_1, tournament_id);
			Tournaments::<Test, Instance1>::insert(
				SEASON_ID_1,
				tournament_id,
				tournament_config.clone(),
			);

			assert_eq!(TournamentBeta::get_active_tournament_for(&SEASON_ID_1), None);
		});
	}
}

mod tournament_mutator {
	use super::*;

	#[test]
	fn try_create_new_tournament_works() {
		let tournament_config = TournamentConfigFor::<Test, Instance1>::default();
		ExtBuilder::default().build().execute_with(|| {
			let result = TournamentAlpha::try_create_new_tournament_for(
				&SEASON_ID_1,
				tournament_config.clone(),
			);
			assert_ok!(result);
			let tournament_id = result.unwrap();

			System::assert_last_event(mock::RuntimeEvent::TournamentAlpha(
				crate::Event::TournamentCreated { season_id: SEASON_ID_1, tournament_id },
			));

			assert_eq!(
				Tournaments::<Test, Instance1>::get(SEASON_ID_1, tournament_id),
				Some(tournament_config)
			);
			assert_eq!(NextTournamentIds::<Test, Instance1>::get(SEASON_ID_1), tournament_id);
		});
	}

	#[test]
	fn try_create_new_tournament_fails_for_invalid_configurations() {
		ExtBuilder::default().build().execute_with(|| {
			let tournament_config_1 =
				TournamentConfigFor::<Test, Instance1>::default().start(10).end(5);

			assert_noop!(
				TournamentAlpha::try_create_new_tournament_for(&SEASON_ID_1, tournament_config_1),
				Error::<Test, Instance1>::InvalidTournamentConfig
			);

			let tournament_config_2 =
				TournamentConfigFor::<Test, Instance1>::default().max_players(0);

			assert_noop!(
				TournamentAlpha::try_create_new_tournament_for(&SEASON_ID_1, tournament_config_2),
				Error::<Test, Instance1>::InvalidTournamentConfig
			);

			let tournament_config_3 =
				TournamentConfigFor::<Test, Instance1>::default().max_players(MAX_PLAYERS + 1);

			assert_noop!(
				TournamentAlpha::try_create_new_tournament_for(&SEASON_ID_1, tournament_config_3),
				Error::<Test, Instance1>::InvalidTournamentConfig
			);

			let tournament_config_4 = TournamentConfigFor::<Test, Instance1>::default()
				.reward_table(bounded_vec![80, 30, 20]);

			assert_noop!(
				TournamentAlpha::try_create_new_tournament_for(&SEASON_ID_1, tournament_config_4),
				Error::<Test, Instance1>::InvalidTournamentConfig
			);
		});
	}

	#[test]
	fn try_start_new_tournament_works() {
		let tournament_config = TournamentConfigFor::<Test, Instance1>::default();
		ExtBuilder::default().build().execute_with(|| {
			let result = TournamentAlpha::try_create_new_tournament_for(
				&SEASON_ID_1,
				tournament_config.clone(),
			);
			assert_ok!(result);
			let tournament_id = result.unwrap();

			assert_ok!(TournamentAlpha::try_create_new_tournament_for(
				&SEASON_ID_1,
				tournament_config.clone(),
			));

			assert_ok!(TournamentAlpha::try_start_next_tournament_for(&SEASON_ID_1));

			System::assert_last_event(mock::RuntimeEvent::TournamentAlpha(
				crate::Event::TournamentStarted { season_id: SEASON_ID_1, tournament_id },
			));

			assert_eq!(ActiveTournaments::<Test, Instance1>::get(SEASON_ID_1), Some(tournament_id));
			assert_eq!(LatestTournaments::<Test, Instance1>::get(SEASON_ID_1), None);
		});
	}

	#[test]
	fn try_start_new_tournament_fails_with_already_active_tournament() {
		let tournament_config = TournamentConfigFor::<Test, Instance1>::default();
		ExtBuilder::default().build().execute_with(|| {
			assert_ok!(TournamentAlpha::try_create_new_tournament_for(
				&SEASON_ID_1,
				tournament_config.clone(),
			));
			assert_ok!(TournamentAlpha::try_start_next_tournament_for(&SEASON_ID_1));

			assert_ok!(TournamentAlpha::try_create_new_tournament_for(
				&SEASON_ID_1,
				tournament_config.clone(),
			));

			assert_noop!(
				TournamentAlpha::try_start_next_tournament_for(&SEASON_ID_1),
				Error::<Test, Instance1>::AnotherTournamentAlreadyActiveForSeason
			);
		});
	}

	#[test]
	fn try_finish_active_tournament_works() {
		let tournament_config = TournamentConfigFor::<Test, Instance1>::default().start(1).end(5);
		ExtBuilder::default().build().execute_with(|| {
			let result = TournamentAlpha::try_create_new_tournament_for(
				&SEASON_ID_1,
				tournament_config.clone(),
			);
			assert_ok!(result);
			let tournament_id = result.unwrap();

			assert_ok!(TournamentAlpha::try_start_next_tournament_for(&SEASON_ID_1));

			run_to_block(6);

			assert_ok!(TournamentAlpha::try_finish_active_tournament_for(&SEASON_ID_1));

			System::assert_last_event(mock::RuntimeEvent::TournamentAlpha(
				crate::Event::TournamentEnded { season_id: SEASON_ID_1, tournament_id },
			));

			assert_eq!(ActiveTournaments::<Test, Instance1>::get(SEASON_ID_1), None);
			assert_eq!(LatestTournaments::<Test, Instance1>::get(SEASON_ID_1), Some(tournament_id));
		});
	}

	#[test]
	fn try_finish_active_tournament_fails_when_non_active_tournament() {
		ExtBuilder::default().build().execute_with(|| {
			assert_noop!(
				TournamentAlpha::try_finish_active_tournament_for(&SEASON_ID_1),
				Error::<Test, Instance1>::NoActiveTournamentForSeason
			);
		});
	}
}

mod tournament_ranker {
	use super::*;

	#[test]
	fn tournament_ranker_works() {
		let tournament_config =
			TournamentConfigFor::<Test, Instance1>::default().max_players(MAX_PLAYERS);
		ExtBuilder::default()
			.balances(&[
				(ALICE, 1_000),
				(BOB, 1_000),
				(CHARLIE, 1_000),
				(DAVE, 1_000),
				(EDWARD, 1_000),
			])
			.build()
			.execute_with(|| {
				let result = TournamentAlpha::try_create_new_tournament_for(
					&SEASON_ID_1,
					tournament_config.clone(),
				);
				assert_ok!(result);
				let tournament_id = result.unwrap();
				let tournament_account =
					TournamentAlpha::tournament_treasury_account_id(&SEASON_ID_1, &tournament_id);

				assert_ok!(TournamentAlpha::try_start_next_tournament_for(&SEASON_ID_1));

				assert_eq!(
					TournamentRankings::<Test, Instance1>::get((
						SEASON_ID_1,
						tournament_id,
						MockRankCategory::A
					)),
					PlayerTableFor::<Test, Instance1>::default()
				);

				assert_ok!(TournamentAlpha::try_rank_entity_in_tournament_for(
					&ALICE,
					&SEASON_ID_1,
					&MockRankCategory::A,
					&10_u32,
					&MockRanker
				));

				assert_eq!(Balances::free_balance(tournament_account), 100);
				assert_eq!(
					TournamentRankings::<Test, Instance1>::get((
						SEASON_ID_1,
						tournament_id,
						MockRankCategory::A
					)),
					PlayerTableFor::<Test, Instance1>::try_from(vec![(ALICE, 10)])
						.expect("Should build player_table")
				);

				let rankings: [(MockAccountId, MockEntity); 5] =
					[(BOB, 5), (CHARLIE, 12), (DAVE, 3), (EDWARD, 50), (BOB, 17)];

				for (account, entity) in rankings {
					assert_ok!(TournamentAlpha::try_rank_entity_in_tournament_for(
						&account,
						&SEASON_ID_1,
						&MockRankCategory::A,
						&entity,
						&MockRanker
					));
				}

				assert_eq!(Balances::free_balance(tournament_account), 600);
				assert_eq!(
					TournamentRankings::<Test, Instance1>::get((
						SEASON_ID_1,
						tournament_id,
						MockRankCategory::A
					)),
					PlayerTableFor::<Test, Instance1>::try_from(vec![
						(EDWARD, 50),
						(BOB, 17),
						(CHARLIE, 12),
						(ALICE, 10),
						(BOB, 5),
						(DAVE, 3),
					])
					.expect("Should build player_table")
				);

				let rankings: [(MockAccountId, MockEntity); 6] =
					[(ALICE, 70), (EDWARD, 80), (DAVE, 1), (EDWARD, 25), (CHARLIE, 35), (BOB, 9)];

				for (account, entity) in rankings {
					assert_ok!(TournamentAlpha::try_rank_entity_in_tournament_for(
						&account,
						&SEASON_ID_1,
						&MockRankCategory::A,
						&entity,
						&MockRanker
					));
				}

				assert_eq!(Balances::free_balance(tournament_account), 1_200);
				assert_eq!(
					TournamentRankings::<Test, Instance1>::get((
						SEASON_ID_1,
						tournament_id,
						MockRankCategory::A
					)),
					PlayerTableFor::<Test, Instance1>::try_from(vec![
						(EDWARD, 80),
						(ALICE, 70),
						(EDWARD, 50),
						(CHARLIE, 35),
						(EDWARD, 25),
						(BOB, 17),
						(CHARLIE, 12),
						(ALICE, 10),
						(BOB, 9),
						(BOB, 5),
					])
					.expect("Should build player_table")
				);
			});
	}

	#[test]
	fn tournament_ranker_fails_with_no_active_tournament() {
		ExtBuilder::default().build().execute_with(|| {
			assert_noop!(
				TournamentAlpha::try_rank_entity_in_tournament_for(
					&ALICE,
					&SEASON_ID_1,
					&MockRankCategory::A,
					&10_u32,
					&MockRanker
				),
				Error::<Test, Instance1>::NoActiveTournamentForSeason
			);
		});
	}
}

#[test]
fn test_full_tournament_workflow() {
	let tournament_config = TournamentConfigFor::<Test, Instance1>::default()
		.start(1)
		.end(5)
		.reward_table(RewardTable::try_from(vec![50, 10]).expect("Should created reward table"))
		.max_players(2);
	ExtBuilder::default()
		.balances(&[(ALICE, 1_000), (BOB, 1_000), (CHARLIE, 1_000), (DAVE, 1_000), (EDWARD, 1_000)])
		.build()
		.execute_with(|| {
			let tournament_id = {
				let result = TournamentAlpha::try_create_new_tournament_for(
					&SEASON_ID_1,
					tournament_config.clone(),
				);
				assert_ok!(result);
				result.unwrap()
			};

			System::assert_last_event(mock::RuntimeEvent::TournamentAlpha(
				crate::Event::TournamentCreated { season_id: SEASON_ID_1, tournament_id },
			));

			let tournament_account =
				TournamentAlpha::tournament_treasury_account_id(&SEASON_ID_1, &tournament_id);

			// Starting created tournament
			assert_ok!(TournamentAlpha::try_start_next_tournament_for(&SEASON_ID_1));

			System::assert_last_event(mock::RuntimeEvent::TournamentAlpha(
				crate::Event::TournamentStarted { season_id: SEASON_ID_1, tournament_id },
			));

			assert_eq!(
				TournamentAlpha::get_active_tournament_for(&SEASON_ID_1),
				Some(tournament_config)
			);

			// Ranking some entities
			let rankings: [(MockAccountId, MockEntity); 6] =
				[(ALICE, 120), (BOB, 30), (DAVE, 22), (EDWARD, 99), (CHARLIE, 70), (BOB, 56)];

			for (account, entity) in rankings {
				assert_ok!(TournamentAlpha::try_rank_entity_in_tournament_for(
					&account,
					&SEASON_ID_1,
					&MockRankCategory::A,
					&entity,
					&MockRanker
				));
			}

			assert_eq!(Balances::free_balance(tournament_account), 600);
			assert_eq!(
				TournamentRankings::<Test, Instance1>::get((
					SEASON_ID_1,
					tournament_id,
					MockRankCategory::A
				)),
				PlayerTableFor::<Test, Instance1>::try_from(vec![(ALICE, 120), (EDWARD, 99),])
					.expect("Should build player_table")
			);

			run_to_block(10);

			// Ending tournament
			assert_ok!(TournamentAlpha::try_finish_active_tournament_for(&SEASON_ID_1));

			System::assert_last_event(mock::RuntimeEvent::TournamentAlpha(
				crate::Event::TournamentEnded { season_id: SEASON_ID_1, tournament_id },
			));

			assert_eq!(Balances::free_balance(ALICE), 1_200);
			assert_eq!(Balances::free_balance(EDWARD), 960);

			assert_eq!(ActiveTournaments::<Test, Instance1>::get(SEASON_ID_1), None);
			assert_eq!(LatestTournaments::<Test, Instance1>::get(SEASON_ID_1), Some(tournament_id));
		});
}
