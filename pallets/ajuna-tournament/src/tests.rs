use crate::{mock::*, *};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::{bounded_vec, testing::H256};

impl Default for TournamentConfig<BlockNumberFor<Test>, MockBalance> {
	fn default() -> Self {
		Self {
			start: 10,
			end: 50,
			claim_end: 70,
			initial_reward: Some(10),
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

	pub(crate) fn claim_end(mut self, claim_end: BlockNumberFor<Test>) -> Self {
		self.claim_end = claim_end;
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
	fn get_active_tournament_config_works() {
		let tournament_id = 1;
		let tournament_config = TournamentConfigFor::<Test, Instance1>::default()
			.start(10)
			.end(20)
			.claim_end(30);
		ExtBuilder::default().build().execute_with(|| {
			ActiveTournaments::<Test, Instance1>::insert(SEASON_ID_1, TournamentState::Inactive);
			EnabledSeasons::<Test, Instance1>::mutate(|season_set| {
				season_set.try_insert(SEASON_ID_1).expect("Should insert")
			});
			Tournaments::<Test, Instance1>::insert(
				SEASON_ID_1,
				tournament_id,
				tournament_config.clone(),
			);

			run_to_block(15);

			assert_eq!(
				TournamentAlpha::get_active_tournament_config_for(&SEASON_ID_1),
				Some(tournament_config)
			);
			assert_eq!(TournamentAlpha::get_active_tournament_config_for(&SEASON_ID_2), None);
		});
	}

	#[test]
	fn get_active_tournament_config_doesnt_apply_to_different_pallet_instances() {
		let tournament_id = 1;
		let tournament_config = TournamentConfigFor::<Test, Instance1>::default()
			.start(10)
			.end(20)
			.claim_end(30);
		ExtBuilder::default().build().execute_with(|| {
			ActiveTournaments::<Test, Instance1>::insert(SEASON_ID_1, TournamentState::Inactive);
			EnabledSeasons::<Test, Instance1>::mutate(|season_set| {
				season_set.try_insert(SEASON_ID_1).expect("Should insert")
			});
			Tournaments::<Test, Instance1>::insert(
				SEASON_ID_1,
				tournament_id,
				tournament_config.clone(),
			);

			run_to_block(15);

			assert_eq!(TournamentBeta::get_active_tournament_config_for(&SEASON_ID_1), None);
		});
	}

	#[test]
	fn check_is_golden_duck_enabled_for() {
		ExtBuilder::default().build().execute_with(|| {
			EnabledSeasons::<Test, Instance1>::mutate(|season_set| {
				season_set.try_insert(SEASON_ID_1).expect("Should insert")
			});

			// Golden duck tournament
			{
				let golden_duck_tournament = TournamentConfigFor::<Test, Instance1>::default()
					.start(10)
					.end(20)
					.claim_end(30)
					.reward_table(bounded_vec![40, 20, 10]);
				assert_ok!(TournamentAlpha::try_create_new_tournament_for(
					&ALICE,
					&SEASON_ID_1,
					golden_duck_tournament.clone(),
				));

				run_to_block(15);

				assert!(TournamentAlpha::is_golden_duck_enabled_for(&SEASON_ID_1));
			};

			// Non-Golden duck tournament
			{
				let non_golden_duck_tournament = TournamentConfigFor::<Test, Instance1>::default()
					.reward_table(bounded_vec![50, 30, 20])
					.start(25)
					.end(40)
					.claim_end(60);
				assert_ok!(TournamentAlpha::try_create_new_tournament_for(
					&ALICE,
					&SEASON_ID_2,
					non_golden_duck_tournament.clone(),
				));

				run_to_block(30);

				assert!(!TournamentAlpha::is_golden_duck_enabled_for(&SEASON_ID_2));
			};
		});
	}

	#[test]
	fn check_get_treasury_account_for() {
		ExtBuilder::default().build().execute_with(|| {
			assert_ne!(
				TournamentAlpha::get_treasury_account_for(&SEASON_ID_1),
				TournamentAlpha::get_treasury_account_for(&SEASON_ID_2)
			);
		});
	}
}

mod tournament_mutator {
	use super::*;

	#[test]
	fn try_create_new_tournament_works() {
		let tournament_config = TournamentConfigFor::<Test, Instance1>::default();
		ExtBuilder::default().build().execute_with(|| {
			let tournament_id = {
				let result = TournamentAlpha::try_create_new_tournament_for(
					&ALICE,
					&SEASON_ID_1,
					tournament_config.clone(),
				);
				assert_ok!(result);
				result.unwrap()
			};

			System::assert_last_event(mock::RuntimeEvent::TournamentAlpha(
				crate::Event::TournamentCreated { season_id: SEASON_ID_1, tournament_id },
			));

			assert_eq!(
				Tournaments::<Test, Instance1>::get(SEASON_ID_1, tournament_id),
				Some(tournament_config)
			);
			assert_eq!(NextTournamentIds::<Test, Instance1>::get(SEASON_ID_1), tournament_id + 1);
		});
	}

	#[test]
	fn try_create_new_tournament_with_initial_reward_takes_funds_from_creator() {
		let tournament_config =
			TournamentConfigFor::<Test, Instance1>::default().initial_reward(Some(330));
		ExtBuilder::default().balances(&[(ALICE, 1_000)]).build().execute_with(|| {
			assert_eq!(Balances::free_balance(ALICE), 1_000);

			assert_ok!(TournamentAlpha::try_create_new_tournament_for(
				&ALICE,
				&SEASON_ID_1,
				tournament_config.clone(),
			));

			assert_eq!(Balances::free_balance(ALICE), 670);
			assert_eq!(
				Balances::free_balance(TournamentAlpha::tournament_treasury_account_id(
					SEASON_ID_1
				)),
				330
			);
		});
	}

	#[test]
	fn try_create_new_tournament_fails_for_invalid_configurations() {
		ExtBuilder::default().build().execute_with(|| {
			run_to_block(3);

			// Tournament starting block should not be lower than the current block
			let tournament_config =
				TournamentConfigFor::<Test, Instance1>::default().start(2).end(3);
			assert_noop!(
				TournamentAlpha::try_create_new_tournament_for(
					&ALICE,
					&SEASON_ID_1,
					tournament_config
				),
				Error::<Test, Instance1>::InvalidTournamentConfig
			);

			// Tournament starting block should not be lower than the active end block
			let tournament_config =
				TournamentConfigFor::<Test, Instance1>::default().start(10).end(5);
			assert_noop!(
				TournamentAlpha::try_create_new_tournament_for(
					&ALICE,
					&SEASON_ID_2,
					tournament_config
				),
				Error::<Test, Instance1>::InvalidTournamentConfig
			);

			// Tournament end block should not be lower than the claim end block
			let tournament_config =
				TournamentConfigFor::<Test, Instance1>::default().end(10).claim_end(5);
			assert_noop!(
				TournamentAlpha::try_create_new_tournament_for(&ALICE, &3, tournament_config),
				Error::<Test, Instance1>::InvalidTournamentConfig
			);

			let min_duration = MinimumTournamentDuration::get().saturating_sub(1);

			// The amount of blocks between start and end should be greater or equal than
			// 'MinimumTournamentDuration'
			let tournament_config = TournamentConfigFor::<Test, Instance1>::default()
				.start(10)
				.end(10 + min_duration);
			assert_noop!(
				TournamentAlpha::try_create_new_tournament_for(&ALICE, &4, tournament_config),
				Error::<Test, Instance1>::InvalidTournamentConfig
			);

			// The amount of blocks between end and claim_end should be greater or equal than
			// 'MinimumTournamentDuration'
			let tournament_config = TournamentConfigFor::<Test, Instance1>::default()
				.start(5)
				.end(10)
				.claim_end(10 + min_duration);
			assert_noop!(
				TournamentAlpha::try_create_new_tournament_for(&ALICE, &5, tournament_config),
				Error::<Test, Instance1>::InvalidTournamentConfig
			);

			// The starting block of a tournament should be greater than the previous tournament
			// claim_end block
			let tournament_config =
				TournamentConfigFor::<Test, Instance1>::default().start(5).end(10).claim_end(15);
			assert_ok!(TournamentAlpha::try_create_new_tournament_for(
				&ALICE,
				&6,
				tournament_config.clone()
			));
			assert_noop!(
				TournamentAlpha::try_create_new_tournament_for(&ALICE, &6, tournament_config),
				Error::<Test, Instance1>::InvalidTournamentConfig
			);

			// The tournament config should have either initial_reward or take_reward_fee filled
			let tournament_config = TournamentConfigFor::<Test, Instance1>::default()
				.initial_reward(None)
				.take_fee_percentage(None);
			assert_noop!(
				TournamentAlpha::try_create_new_tournament_for(&ALICE, &7, tournament_config),
				Error::<Test, Instance1>::InvalidTournamentConfig
			);

			// initial_reward should be greater than 0
			let tournament_config =
				TournamentConfigFor::<Test, Instance1>::default().initial_reward(Some(0));
			assert_noop!(
				TournamentAlpha::try_create_new_tournament_for(&ALICE, &8, tournament_config),
				Error::<Test, Instance1>::InvalidTournamentConfig
			);

			// max_reward should be greater than 0
			let tournament_config =
				TournamentConfigFor::<Test, Instance1>::default().max_reward(Some(0));
			assert_noop!(
				TournamentAlpha::try_create_new_tournament_for(&ALICE, &9, tournament_config),
				Error::<Test, Instance1>::InvalidTournamentConfig
			);

			// max_reward should be greater than 0
			let tournament_config =
				TournamentConfigFor::<Test, Instance1>::default().max_reward(Some(0));
			assert_noop!(
				TournamentAlpha::try_create_new_tournament_for(&ALICE, &10, tournament_config),
				Error::<Test, Instance1>::InvalidTournamentConfig
			);

			// take_fee_percentage should be smaller or equal than 100
			let tournament_config =
				TournamentConfigFor::<Test, Instance1>::default().take_fee_percentage(Some(120));
			assert_noop!(
				TournamentAlpha::try_create_new_tournament_for(&ALICE, &11, tournament_config),
				Error::<Test, Instance1>::InvalidTournamentConfig
			);

			// reward_table percentages should add up to a maximum of 100
			let tournament_config = TournamentConfigFor::<Test, Instance1>::default().reward_table(
				RewardTable::try_from(vec![90, 80, 20]).expect("Should create table"),
			);
			assert_noop!(
				TournamentAlpha::try_create_new_tournament_for(&ALICE, &12, tournament_config),
				Error::<Test, Instance1>::InvalidTournamentConfig
			);

			// max_players should be greater than 0
			let tournament_config =
				TournamentConfigFor::<Test, Instance1>::default().max_players(0);
			assert_noop!(
				TournamentAlpha::try_create_new_tournament_for(&ALICE, &13, tournament_config),
				Error::<Test, Instance1>::InvalidTournamentConfig
			);

			// max_players should be lower than `MAX_PLAYERS` constant
			let tournament_config =
				TournamentConfigFor::<Test, Instance1>::default().max_players(MAX_PLAYERS + 1);
			assert_noop!(
				TournamentAlpha::try_create_new_tournament_for(&ALICE, &14, tournament_config),
				Error::<Test, Instance1>::InvalidTournamentConfig
			);
		});
	}

	#[test]
	fn try_enable_tournament_processing_works() {
		ExtBuilder::default().build().execute_with(|| {
			let tournament_config = TournamentConfigFor::<Test, Instance1>::default()
				.start(10)
				.end(20)
				.claim_end(30);
			let tournament_id = {
				let maybe_id = TournamentAlpha::try_create_new_tournament_for(
					&ALICE,
					&SEASON_ID_1,
					tournament_config.clone(),
				);

				assert_ok!(maybe_id);

				maybe_id.expect("Should get id")
			};
			assert_ok!(TournamentAlpha::try_create_new_tournament_for(
				&ALICE,
				&SEASON_ID_2,
				tournament_config
			));

			assert_ok!(TournamentAlpha::try_enable_tournament_processing_for_season(&SEASON_ID_1));

			run_to_block(15);

			System::assert_last_event(mock::RuntimeEvent::TournamentAlpha(
				crate::Event::TournamentActivePeriodStarted {
					season_id: SEASON_ID_1,
					tournament_id,
				},
			));

			assert_eq!(
				ActiveTournaments::<Test, Instance1>::get(SEASON_ID_1),
				TournamentState::ActivePeriod(tournament_id)
			);
			assert_eq!(
				ActiveTournaments::<Test, Instance1>::get(SEASON_ID_2),
				TournamentState::Inactive
			);
		});
	}

	#[test]
	fn try_disable_tournament_processing_works() {
		ExtBuilder::default().build().execute_with(|| {
			let tournament_config_1 = TournamentConfigFor::<Test, Instance1>::default()
				.start(10)
				.end(20)
				.claim_end(30);
			let tournament_id_1 = {
				let maybe_id = TournamentAlpha::try_create_new_tournament_for(
					&ALICE,
					&SEASON_ID_1,
					tournament_config_1,
				);

				assert_ok!(maybe_id);

				maybe_id.expect("Should get id")
			};

			assert_ok!(TournamentAlpha::try_enable_tournament_processing_for_season(&SEASON_ID_1));

			run_to_block(15);

			assert_eq!(
				ActiveTournaments::<Test, Instance1>::get(SEASON_ID_1),
				TournamentState::ActivePeriod(tournament_id_1)
			);

			run_to_block(100);

			System::assert_last_event(mock::RuntimeEvent::TournamentAlpha(
				crate::Event::TournamentEnded {
					season_id: SEASON_ID_1,
					tournament_id: tournament_id_1,
				},
			));

			let tournament_config_2 = TournamentConfigFor::<Test, Instance1>::default()
				.start(120)
				.end(180)
				.claim_end(300);
			let tournament_id_2 = {
				let maybe_id = TournamentAlpha::try_create_new_tournament_for(
					&ALICE,
					&SEASON_ID_1,
					tournament_config_2,
				);

				assert_ok!(maybe_id);

				maybe_id.expect("Should get id")
			};

			assert_ok!(TournamentAlpha::try_disable_tournament_processing_for_season(&SEASON_ID_1));

			run_to_block(130);

			assert_eq!(
				LatestTournaments::<Test, Instance1>::get(SEASON_ID_1),
				Some(tournament_id_1)
			);
			assert_eq!(
				ActiveTournaments::<Test, Instance1>::get(SEASON_ID_1),
				TournamentState::Inactive
			);

			assert_ok!(TournamentAlpha::try_enable_tournament_processing_for_season(&SEASON_ID_1));

			run_to_block(140);

			System::assert_last_event(mock::RuntimeEvent::TournamentAlpha(
				crate::Event::TournamentActivePeriodStarted {
					season_id: SEASON_ID_1,
					tournament_id: tournament_id_2,
				},
			));

			assert_eq!(
				ActiveTournaments::<Test, Instance1>::get(SEASON_ID_1),
				TournamentState::ActivePeriod(tournament_id_2)
			);
		});
	}
}

mod tournament_ranker {
	use super::*;

	#[test]
	fn tournament_ranker_works() {
		let tournament_config = TournamentConfigFor::<Test, Instance1>::default()
			.max_players(MAX_PLAYERS)
			.start(10)
			.end(50)
			.claim_end(90);
		ExtBuilder::default().build().execute_with(|| {
			let tournament_id = {
				let result = TournamentAlpha::try_create_new_tournament_for(
					&ALICE,
					&SEASON_ID_1,
					tournament_config.clone(),
				);
				assert_ok!(result);
				result.unwrap()
			};

			assert_ok!(TournamentAlpha::try_enable_tournament_processing_for_season(&SEASON_ID_1));

			run_to_block(15);

			assert_eq!(
				TournamentRankings::<Test, Instance1>::get(SEASON_ID_1, tournament_id),
				PlayerTableFor::<Test, Instance1>::default()
			);

			assert_ok!(TournamentAlpha::try_rank_entity_in_tournament_for(
				&SEASON_ID_1,
				&10_u32,
				&MockRanker
			));

			assert_eq!(
				TournamentRankings::<Test, Instance1>::get(SEASON_ID_1, tournament_id),
				PlayerTableFor::<Test, Instance1>::try_from(vec![10])
					.expect("Should build player_table")
			);

			let rankings: [MockEntity; 5] = [5, 12, 3, 50, 17];

			for entity in rankings {
				assert_ok!(TournamentAlpha::try_rank_entity_in_tournament_for(
					&SEASON_ID_1,
					&entity,
					&MockRanker
				));
			}

			assert_eq!(
				TournamentRankings::<Test, Instance1>::get(SEASON_ID_1, tournament_id),
				PlayerTableFor::<Test, Instance1>::try_from(vec![50, 17, 12, 10, 5, 3])
					.expect("Should build player_table")
			);

			let rankings: [MockEntity; 6] = [70, 80, 1, 25, 35, 9];

			for entity in rankings {
				assert_ok!(TournamentAlpha::try_rank_entity_in_tournament_for(
					&SEASON_ID_1,
					&entity,
					&MockRanker
				));
			}

			assert_eq!(
				TournamentRankings::<Test, Instance1>::get(SEASON_ID_1, tournament_id),
				PlayerTableFor::<Test, Instance1>::try_from(vec![
					80, 70, 50, 35, 25, 17, 12, 10, 9, 5,
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
					&SEASON_ID_1,
					&10_u32,
					&MockRanker
				),
				Error::<Test, Instance1>::NoActiveTournamentForSeason
			);
		});
	}
}

mod tournament_claimer {
	use super::*;

	#[test]
	fn try_claim_tournament_rewards_works() {
		ExtBuilder::default().build().execute_with(|| {
			let tournament_config = TournamentConfigFor::<Test, Instance1>::default()
				.initial_reward(Some(100))
				.start(10)
				.end(50)
				.claim_end(90)
				.reward_table(RewardTable::try_from(vec![50, 30]).expect("Should create table"))
				.max_players(2);
			ExtBuilder::default().build().execute_with(|| {
				let tournament_id = {
					let result = TournamentAlpha::try_create_new_tournament_for(
						&ALICE,
						&SEASON_ID_1,
						tournament_config,
					);
					assert_ok!(result);
					result.unwrap()
				};
				let tournament_account =
					TournamentAlpha::tournament_treasury_account_id(SEASON_ID_1);

				assert_ok!(TournamentAlpha::try_enable_tournament_processing_for_season(
					&SEASON_ID_1
				));

				run_to_block(15);

				assert_ok!(TournamentAlpha::try_rank_entity_in_tournament_for(
					&SEASON_ID_1,
					&10_u32,
					&MockRanker
				));
				assert_ok!(TournamentAlpha::try_rank_entity_in_tournament_for(
					&SEASON_ID_1,
					&15_u32,
					&MockRanker
				));

				assert_ok!(TournamentAlpha::try_rank_entity_for_golden_duck(
					&SEASON_ID_1,
					&H256::from_low_u64_be(10),
				));

				assert_eq!(Balances::free_balance(tournament_account), 100);
				assert_eq!(Balances::free_balance(ALICE), 900);
				assert_eq!(Balances::free_balance(BOB), 1_000);

				run_to_block(60);

				System::assert_last_event(mock::RuntimeEvent::TournamentAlpha(
					crate::Event::TournamentClaimPeriodStarted {
						season_id: SEASON_ID_1,
						tournament_id,
					},
				));

				assert_eq!(
					TournamentRankings::<Test, Instance1>::get(SEASON_ID_1, tournament_id),
					PlayerTableFor::<Test, Instance1>::try_from(vec![15, 10])
						.expect("Should build player_table")
				);

				assert_ok!(TournamentAlpha::try_claim_tournament_reward_for(
					&SEASON_ID_1,
					&ALICE,
					&10_u32
				));

				assert_ok!(TournamentAlpha::try_claim_tournament_reward_for(
					&SEASON_ID_1,
					&BOB,
					&15_u32
				));

				assert_eq!(Balances::free_balance(tournament_account), 20);
				assert_eq!(Balances::free_balance(ALICE), 930);
				assert_eq!(Balances::free_balance(BOB), 1_050);

				assert_ok!(TournamentAlpha::try_claim_golden_duck_for(
					&SEASON_ID_1,
					&ALICE,
					&H256::from_low_u64_be(10),
				));

				assert_eq!(Balances::free_balance(ALICE), 950);
				assert_eq!(Balances::free_balance(BOB), 1_050);
				assert_eq!(Balances::free_balance(tournament_account), 0);
			});
		});
	}

	#[test]
	fn try_claim_tournament_rewards_with_max_reward() {
		let tournament_config = TournamentConfigFor::<Test, Instance1>::default()
			.max_reward(Some(200))
			.initial_reward(Some(500))
			.start(10)
			.end(50)
			.claim_end(90)
			.reward_table(RewardTable::try_from(vec![50, 30]).expect("Should create table"));
		ExtBuilder::default().build().execute_with(|| {
			let tournament_id = {
				let result = TournamentAlpha::try_create_new_tournament_for(
					&ALICE,
					&SEASON_ID_1,
					tournament_config,
				);
				assert_ok!(result);
				result.unwrap()
			};

			let tournament_account = TournamentAlpha::tournament_treasury_account_id(SEASON_ID_1);

			assert_eq!(Balances::free_balance(tournament_account), 500);
			assert_eq!(Balances::free_balance(ALICE), 500);
			assert_eq!(Balances::free_balance(BOB), 1_000);

			assert_ok!(TournamentAlpha::try_enable_tournament_processing_for_season(&SEASON_ID_1));

			run_to_block(15);

			assert_ok!(TournamentAlpha::try_rank_entity_in_tournament_for(
				&SEASON_ID_1,
				&10_u32,
				&MockRanker
			));

			assert_ok!(TournamentAlpha::try_rank_entity_in_tournament_for(
				&SEASON_ID_1,
				&15_u32,
				&MockRanker
			));

			assert_ok!(TournamentAlpha::try_rank_entity_for_golden_duck(
				&SEASON_ID_1,
				&H256::from_low_u64_be(10),
			));

			run_to_block(60);

			System::assert_last_event(mock::RuntimeEvent::TournamentAlpha(
				crate::Event::TournamentClaimPeriodStarted {
					season_id: SEASON_ID_1,
					tournament_id,
				},
			));

			assert_eq!(
				TournamentRankings::<Test, Instance1>::get(SEASON_ID_1, tournament_id),
				PlayerTableFor::<Test, Instance1>::try_from(vec![15, 10])
					.expect("Should build player_table")
			);

			assert_ok!(TournamentAlpha::try_claim_tournament_reward_for(
				&SEASON_ID_1,
				&ALICE,
				&10_u32
			));

			assert_ok!(TournamentAlpha::try_claim_tournament_reward_for(
				&SEASON_ID_1,
				&BOB,
				&15_u32
			));

			assert_eq!(Balances::free_balance(tournament_account), 340);
			assert_eq!(Balances::free_balance(ALICE), 560);
			assert_eq!(Balances::free_balance(BOB), 1_100);

			assert_ok!(TournamentAlpha::try_claim_golden_duck_for(
				&SEASON_ID_1,
				&ALICE,
				&H256::from_low_u64_be(10),
			));

			assert_eq!(Balances::free_balance(tournament_account), 300);
			assert_eq!(Balances::free_balance(ALICE), 600);
			assert_eq!(Balances::free_balance(BOB), 1_100);
		});
	}

	#[test]
	fn try_claim_tournament_rewards_fails_outside_claim_state() {
		ExtBuilder::default().build().execute_with(|| {
			let tournament_config = TournamentConfigFor::<Test, Instance1>::default()
				.initial_reward(Some(100))
				.start(10)
				.end(50)
				.claim_end(90)
				.reward_table(RewardTable::try_from(vec![50, 30]).expect("Should create table"))
				.max_players(2);
			ExtBuilder::default().build().execute_with(|| {
				let tournament_id = {
					let result = TournamentAlpha::try_create_new_tournament_for(
						&ALICE,
						&SEASON_ID_1,
						tournament_config,
					);
					assert_ok!(result);
					result.unwrap()
				};
				assert_ok!(TournamentAlpha::try_enable_tournament_processing_for_season(
					&SEASON_ID_1
				));

				run_to_block(15);

				assert_ok!(TournamentAlpha::try_rank_entity_in_tournament_for(
					&SEASON_ID_1,
					&10_u32,
					&MockRanker
				));
				assert_ok!(TournamentAlpha::try_rank_entity_in_tournament_for(
					&SEASON_ID_1,
					&15_u32,
					&MockRanker
				));

				assert_ok!(TournamentAlpha::try_rank_entity_for_golden_duck(
					&SEASON_ID_1,
					&H256::from_low_u64_be(10),
				));

				// Trying to claim reward while still in active state
				assert_noop!(
					TournamentAlpha::try_claim_tournament_reward_for(&SEASON_ID_1, &ALICE, &10_u32),
					Error::<Test, Instance1>::TournamentNotInClaimPeriod
				);

				assert_noop!(
					TournamentAlpha::try_claim_golden_duck_for(
						&SEASON_ID_1,
						&ALICE,
						&H256::from_low_u64_be(10),
					),
					Error::<Test, Instance1>::TournamentNotInClaimPeriod
				);

				run_to_block(60);

				System::assert_last_event(mock::RuntimeEvent::TournamentAlpha(
					crate::Event::TournamentClaimPeriodStarted {
						season_id: SEASON_ID_1,
						tournament_id,
					},
				));

				run_to_block(100);

				System::assert_last_event(mock::RuntimeEvent::TournamentAlpha(
					crate::Event::TournamentEnded { season_id: SEASON_ID_1, tournament_id },
				));

				// Trying to claim reward while still in active state
				assert_noop!(
					TournamentAlpha::try_claim_tournament_reward_for(&SEASON_ID_1, &ALICE, &10_u32),
					Error::<Test, Instance1>::NoActiveTournamentForSeason
				);

				assert_noop!(
					TournamentAlpha::try_claim_golden_duck_for(
						&SEASON_ID_1,
						&ALICE,
						&H256::from_low_u64_be(10),
					),
					Error::<Test, Instance1>::NoActiveTournamentForSeason
				);
			});
		});
	}

	#[test]
	fn try_claim_tournament_rewards_fails_with_non_winner_entities() {
		ExtBuilder::default().build().execute_with(|| {
			let tournament_config = TournamentConfigFor::<Test, Instance1>::default()
				.initial_reward(Some(100))
				.start(10)
				.end(50)
				.claim_end(90)
				.reward_table(RewardTable::try_from(vec![50, 30]).expect("Should create table"))
				.max_players(2);
			ExtBuilder::default().build().execute_with(|| {
				let tournament_id = {
					let result = TournamentAlpha::try_create_new_tournament_for(
						&ALICE,
						&SEASON_ID_1,
						tournament_config,
					);
					assert_ok!(result);
					result.unwrap()
				};
				assert_ok!(TournamentAlpha::try_enable_tournament_processing_for_season(
					&SEASON_ID_1
				));

				run_to_block(15);

				assert_ok!(TournamentAlpha::try_rank_entity_in_tournament_for(
					&SEASON_ID_1,
					&10_u32,
					&MockRanker
				));
				assert_ok!(TournamentAlpha::try_rank_entity_in_tournament_for(
					&SEASON_ID_1,
					&15_u32,
					&MockRanker
				));

				assert_ok!(TournamentAlpha::try_rank_entity_for_golden_duck(
					&SEASON_ID_1,
					&H256::from_low_u64_be(10),
				));

				run_to_block(60);

				System::assert_last_event(mock::RuntimeEvent::TournamentAlpha(
					crate::Event::TournamentClaimPeriodStarted {
						season_id: SEASON_ID_1,
						tournament_id,
					},
				));

				// Trying to claim reward while still in active state
				assert_noop!(
					TournamentAlpha::try_claim_tournament_reward_for(&SEASON_ID_1, &ALICE, &13_u32),
					Error::<Test, Instance1>::RankingCandidateNotInWinnerTable
				);

				assert_noop!(
					TournamentAlpha::try_claim_golden_duck_for(
						&SEASON_ID_1,
						&ALICE,
						&H256::from_low_u64_be(13),
					),
					Error::<Test, Instance1>::GoldenDuckCandidateNotWinner
				);
			});
		});
	}
}

#[test]
fn test_full_tournament_workflow() {
	let tournament_config = TournamentConfigFor::<Test, Instance1>::default()
		.start(20)
		.end(50)
		.claim_end(100)
		.initial_reward(Some(300))
		.max_reward(Some(120))
		.reward_table(RewardTable::try_from(vec![40, 25, 10]).expect("Should created reward table"))
		.max_players(3);
	ExtBuilder::default()
		.balances(&[(ALICE, 1_000), (BOB, 1_000), (CHARLIE, 1_000), (DAVE, 1_000), (EDWARD, 1_000)])
		.build()
		.execute_with(|| {
			let tournament_id = {
				let result = TournamentAlpha::try_create_new_tournament_for(
					&ALICE,
					&SEASON_ID_1,
					tournament_config.clone(),
				);
				assert_ok!(result);
				result.unwrap()
			};

			System::assert_last_event(mock::RuntimeEvent::TournamentAlpha(
				crate::Event::TournamentCreated { season_id: SEASON_ID_1, tournament_id },
			));

			let tournament_account = TournamentAlpha::tournament_treasury_account_id(SEASON_ID_1);

			assert_eq!(Balances::free_balance(tournament_account), 300);
			assert_eq!(Balances::free_balance(ALICE), 700);
			assert_eq!(Balances::free_balance(BOB), 1_000);
			assert_eq!(Balances::free_balance(CHARLIE), 1_000);
			assert_eq!(Balances::free_balance(DAVE), 1_000);
			assert_eq!(Balances::free_balance(EDWARD), 1_000);

			System::assert_last_event(mock::RuntimeEvent::TournamentAlpha(
				crate::Event::TournamentCreated { season_id: SEASON_ID_1, tournament_id },
			));

			assert_ok!(TournamentAlpha::try_enable_tournament_processing_for_season(&SEASON_ID_1));

			run_to_block(25);

			System::assert_last_event(mock::RuntimeEvent::TournamentAlpha(
				crate::Event::TournamentActivePeriodStarted {
					season_id: SEASON_ID_1,
					tournament_id,
				},
			));

			assert_eq!(
				ActiveTournaments::<Test, Instance1>::get(SEASON_ID_1),
				TournamentState::ActivePeriod(tournament_id)
			);
			assert_eq!(LatestTournaments::<Test, Instance1>::get(SEASON_ID_1), None);
			assert_eq!(
				TournamentAlpha::get_active_tournament_config_for(&SEASON_ID_1),
				Some(tournament_config)
			);

			// Ranking some entities
			let rankings_1: [(MockEntity, MockEntityId); 3] = [
				(120, H256::from_low_u64_be(10)),
				(30, H256::from_low_u64_be(45)),
				(22, H256::from_low_u64_be(3)),
			];

			for (entity, entity_id) in rankings_1 {
				assert_ok!(TournamentAlpha::try_rank_entity_in_tournament_for(
					&SEASON_ID_1,
					&entity,
					&MockRanker
				));

				assert_ok!(TournamentAlpha::try_rank_entity_for_golden_duck(
					&SEASON_ID_1,
					&entity_id
				));
			}

			assert_eq!(
				TournamentRankings::<Test, Instance1>::get(SEASON_ID_1, tournament_id),
				PlayerTableFor::<Test, Instance1>::try_from(vec![120, 30, 22])
					.expect("Should build player_table")
			);
			assert_eq!(
				GoldenDucks::<Test, Instance1>::get(SEASON_ID_1, tournament_id),
				GoldenDuckStateFor::<Test, Instance1>::Enabled(25, Some(H256::from_low_u64_be(3)))
			);

			let rankings_2: [(MockEntity, MockEntityId); 3] = [
				(99, H256::from_low_u64_be(26)),
				(70, H256::from_low_u64_be(71)),
				(56, H256::from_low_u64_be(92)),
			];

			for (entity, entity_id) in rankings_2 {
				assert_ok!(TournamentAlpha::try_rank_entity_in_tournament_for(
					&SEASON_ID_1,
					&entity,
					&MockRanker
				));

				assert_ok!(TournamentAlpha::try_rank_entity_for_golden_duck(
					&SEASON_ID_1,
					&entity_id
				));
			}

			assert_eq!(
				TournamentRankings::<Test, Instance1>::get(SEASON_ID_1, tournament_id),
				PlayerTableFor::<Test, Instance1>::try_from(vec![120, 99, 70])
					.expect("Should build player_table")
			);
			assert_eq!(
				GoldenDucks::<Test, Instance1>::get(SEASON_ID_1, tournament_id),
				GoldenDuckStateFor::<Test, Instance1>::Enabled(25, Some(H256::from_low_u64_be(3)))
			);

			run_to_block(65);

			System::assert_last_event(mock::RuntimeEvent::TournamentAlpha(
				crate::Event::TournamentClaimPeriodStarted {
					season_id: SEASON_ID_1,
					tournament_id,
				},
			));

			assert_eq!(
				ActiveTournaments::<Test, Instance1>::get(SEASON_ID_1),
				TournamentState::ClaimPeriod(tournament_id, 120)
			);
			assert_eq!(LatestTournaments::<Test, Instance1>::get(SEASON_ID_1), None);

			assert_ok!(TournamentAlpha::try_claim_tournament_reward_for(
				&SEASON_ID_1,
				&ALICE,
				&120_u32
			));

			assert_noop!(
				TournamentAlpha::try_claim_tournament_reward_for(&SEASON_ID_1, &BOB, &30_u32),
				Error::<Test, Instance1>::RankingCandidateNotInWinnerTable
			);

			assert_ok!(TournamentAlpha::try_claim_tournament_reward_for(
				&SEASON_ID_1,
				&CHARLIE,
				&99_u32
			));

			assert_ok!(TournamentAlpha::try_claim_tournament_reward_for(
				&SEASON_ID_1,
				&DAVE,
				&70_u32
			));

			assert_eq!(Balances::free_balance(tournament_account), 210);
			assert_eq!(Balances::free_balance(ALICE), 748);
			assert_eq!(Balances::free_balance(BOB), 1_000);
			assert_eq!(Balances::free_balance(CHARLIE), 1_030);
			assert_eq!(Balances::free_balance(DAVE), 1_012);
			assert_eq!(Balances::free_balance(EDWARD), 1_000);

			assert_noop!(
				TournamentAlpha::try_claim_golden_duck_for(
					&SEASON_ID_1,
					&CHARLIE,
					&H256::from_low_u64_be(26),
				),
				Error::<Test, Instance1>::GoldenDuckCandidateNotWinner
			);

			assert_ok!(TournamentAlpha::try_claim_golden_duck_for(
				&SEASON_ID_1,
				&EDWARD,
				&H256::from_low_u64_be(3),
			));

			assert_eq!(Balances::free_balance(tournament_account), 180);
			assert_eq!(Balances::free_balance(ALICE), 748);
			assert_eq!(Balances::free_balance(BOB), 1_000);
			assert_eq!(Balances::free_balance(CHARLIE), 1_030);
			assert_eq!(Balances::free_balance(DAVE), 1_012);
			assert_eq!(Balances::free_balance(EDWARD), 1_030);

			run_to_block(120);

			System::assert_last_event(mock::RuntimeEvent::TournamentAlpha(
				crate::Event::TournamentEnded { season_id: SEASON_ID_1, tournament_id },
			));

			assert_eq!(
				ActiveTournaments::<Test, Instance1>::get(SEASON_ID_1),
				TournamentState::Inactive
			);
			assert_eq!(LatestTournaments::<Test, Instance1>::get(SEASON_ID_1), Some(tournament_id));
		});
}
