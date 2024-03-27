use crate::{mock::*, *};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::{bounded_vec, testing::H256};

impl Default for TournamentConfig<BlockNumberFor<Test>, MockBalance> {
	fn default() -> Self {
		Self {
			start: 10,
			active_end: 50,
			claim_end: 70,
			initial_reward: Some(10),
			max_reward: None,
			take_fee_percentage: None,
			reward_distribution: bounded_vec![50, 30, 10],
			golden_duck_config: Default::default(),
			max_players: 3,
		}
	}
}

impl TournamentConfig<BlockNumberFor<Test>, MockBalance> {
	pub(crate) fn start(mut self, start: BlockNumberFor<Test>) -> Self {
		self.start = start;
		self
	}

	pub(crate) fn active_end(mut self, active_end: BlockNumberFor<Test>) -> Self {
		self.active_end = active_end;
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

	pub(crate) fn reward_distribution(
		mut self,
		reward_distribution: RewardDistributionTable,
	) -> Self {
		self.reward_distribution = reward_distribution;
		self
	}

	pub(crate) fn golden_duck_config(mut self, golden_duck_config: GoldenDuckConfig) -> Self {
		self.golden_duck_config = golden_duck_config;
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
		let tournament_config = TournamentConfigFor::<Test, Instance1>::default()
			.start(10)
			.active_end(20)
			.claim_end(30);
		ExtBuilder::default().build().execute_with(|| {
			assert_ok!(TournamentAlpha::try_create_new_tournament_for(
				&ALICE,
				&SEASON_ID_1,
				tournament_config.clone(),
			));

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
		let tournament_config = TournamentConfigFor::<Test, Instance1>::default()
			.start(10)
			.active_end(20)
			.claim_end(30);
		ExtBuilder::default().build().execute_with(|| {
			assert_ok!(TournamentAlpha::try_create_new_tournament_for(
				&ALICE,
				&SEASON_ID_1,
				tournament_config.clone(),
			));

			run_to_block(15);

			assert_eq!(TournamentBeta::get_active_tournament_config_for(&SEASON_ID_1), None);
		});
	}

	#[test]
	fn check_is_golden_duck_enabled_for() {
		ExtBuilder::default().build().execute_with(|| {
			// Golden duck tournament
			{
				let golden_duck_tournament = TournamentConfigFor::<Test, Instance1>::default()
					.start(10)
					.active_end(20)
					.claim_end(30)
					.reward_distribution(bounded_vec![40, 20, 10])
					.golden_duck_config(GoldenDuckConfig::Enabled(20));
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
					.reward_distribution(bounded_vec![50, 30, 20])
					.start(25)
					.active_end(40)
					.claim_end(60)
					.golden_duck_config(GoldenDuckConfig::Disabled);
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
				TournamentConfigFor::<Test, Instance1>::default().start(2).active_end(3);
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
				TournamentConfigFor::<Test, Instance1>::default().start(10).active_end(5);
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
				TournamentConfigFor::<Test, Instance1>::default().active_end(10).claim_end(5);
			assert_noop!(
				TournamentAlpha::try_create_new_tournament_for(&ALICE, &3, tournament_config),
				Error::<Test, Instance1>::InvalidTournamentConfig
			);

			let min_duration = MinimumTournamentPhaseDuration::get().saturating_sub(1);

			// The amount of blocks between start and end should be greater or equal than
			// 'MinimumTournamentPhaseDuration'
			let tournament_config = TournamentConfigFor::<Test, Instance1>::default()
				.start(10)
				.active_end(10 + min_duration);
			assert_noop!(
				TournamentAlpha::try_create_new_tournament_for(&ALICE, &4, tournament_config),
				Error::<Test, Instance1>::InvalidTournamentConfig
			);

			// The amount of blocks between end and claim_end should be greater or equal than
			// 'MinimumTournamentPhaseDuration'
			let tournament_config = TournamentConfigFor::<Test, Instance1>::default()
				.start(5)
				.active_end(10)
				.claim_end(10 + min_duration);
			assert_noop!(
				TournamentAlpha::try_create_new_tournament_for(&ALICE, &5, tournament_config),
				Error::<Test, Instance1>::InvalidTournamentConfig
			);

			// The starting block of a tournament should be greater than the previous tournament
			// claim_end block
			let tournament_config = TournamentConfigFor::<Test, Instance1>::default()
				.start(5)
				.active_end(10)
				.claim_end(15);
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
			let tournament_config = TournamentConfigFor::<Test, Instance1>::default()
				.reward_distribution(
					RewardDistributionTable::try_from(vec![90, 80, 20])
						.expect("Should create table"),
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
}

mod tournament_ranker {
	use super::*;

	#[test]
	fn tournament_ranker_works() {
		let tournament_config = TournamentConfigFor::<Test, Instance1>::default()
			.max_players(MAX_PLAYERS)
			.start(10)
			.active_end(50)
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

			run_to_block(15);

			assert_eq!(
				TournamentRankings::<Test, Instance1>::get(SEASON_ID_1, tournament_id),
				RankingTableFor::<Test, Instance1>::default()
			);

			assert_ok!(TournamentAlpha::try_rank_entity_in_tournament_for(
				&SEASON_ID_1,
				&H256::from_low_u64_be(7),
				&10_u32,
				&MockRanker
			));

			System::assert_last_event(mock::RuntimeEvent::TournamentAlpha(
				crate::Event::EntityEnteredRanking {
					season_id: SEASON_ID_1,
					tournament_id,
					entity_id: H256::from_low_u64_be(7),
					rank: 0_u32,
				},
			));

			assert_eq!(
				TournamentRankings::<Test, Instance1>::get(SEASON_ID_1, tournament_id),
				RankingTableFor::<Test, Instance1>::try_from(vec![(H256::from_low_u64_be(7), 10)])
					.expect("Should build player_table")
			);

			let rankings: [(MockEntityId, MockEntity); 5] = [
				(H256::from_low_u64_be(1), 5),
				(H256::from_low_u64_be(2), 12),
				(H256::from_low_u64_be(3), 3),
				(H256::from_low_u64_be(4), 50),
				(H256::from_low_u64_be(5), 17),
			];

			for (entity_id, entity) in rankings {
				assert_ok!(TournamentAlpha::try_rank_entity_in_tournament_for(
					&SEASON_ID_1,
					&entity_id,
					&entity,
					&MockRanker
				));
			}

			assert_eq!(
				TournamentRankings::<Test, Instance1>::get(SEASON_ID_1, tournament_id),
				RankingTableFor::<Test, Instance1>::try_from(vec![
					(H256::from_low_u64_be(4), 50),
					(H256::from_low_u64_be(5), 17),
					(H256::from_low_u64_be(2), 12),
					(H256::from_low_u64_be(7), 10),
					(H256::from_low_u64_be(1), 5),
					(H256::from_low_u64_be(3), 3)
				])
				.expect("Should build player_table")
			);

			let rankings: [(MockEntityId, MockEntity); 6] = [
				(H256::from_low_u64_be(14), 70),
				(H256::from_low_u64_be(11), 80),
				(H256::from_low_u64_be(23), 1),
				(H256::from_low_u64_be(156), 25),
				(H256::from_low_u64_be(67), 35),
				(H256::from_low_u64_be(9), 9),
			];

			for (entity_id, entity) in rankings {
				assert_ok!(TournamentAlpha::try_rank_entity_in_tournament_for(
					&SEASON_ID_1,
					&entity_id,
					&entity,
					&MockRanker
				));
			}

			assert_eq!(
				TournamentRankings::<Test, Instance1>::get(SEASON_ID_1, tournament_id),
				RankingTableFor::<Test, Instance1>::try_from(vec![
					(H256::from_low_u64_be(11), 80),
					(H256::from_low_u64_be(14), 70),
					(H256::from_low_u64_be(4), 50),
					(H256::from_low_u64_be(67), 35),
					(H256::from_low_u64_be(156), 25),
					(H256::from_low_u64_be(5), 17),
					(H256::from_low_u64_be(2), 12),
					(H256::from_low_u64_be(7), 10),
					(H256::from_low_u64_be(9), 9),
					(H256::from_low_u64_be(1), 5)
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
					&H256::from_low_u64_be(3),
					&10_u32,
					&MockRanker
				),
				Error::<Test, Instance1>::NoActiveTournamentForSeason
			);
		});
	}

	#[test]
	fn tournament_ranker_cannot_rank_same_avatar_twice() {
		let tournament_config = TournamentConfigFor::<Test, Instance1>::default()
			.max_players(MAX_PLAYERS)
			.start(10)
			.active_end(50)
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

			run_to_block(15);

			assert_eq!(
				TournamentRankings::<Test, Instance1>::get(SEASON_ID_1, tournament_id),
				RankingTableFor::<Test, Instance1>::default()
			);

			assert_ok!(TournamentAlpha::try_rank_entity_in_tournament_for(
				&SEASON_ID_1,
				&H256::from_low_u64_be(77),
				&20_u32,
				&MockRanker
			));

			assert_ok!(TournamentAlpha::try_rank_entity_in_tournament_for(
				&SEASON_ID_1,
				&H256::from_low_u64_be(77),
				&20_u32,
				&MockRanker
			));

			assert_eq!(
				TournamentRankings::<Test, Instance1>::get(SEASON_ID_1, tournament_id),
				RankingTableFor::<Test, Instance1>::try_from(vec![(H256::from_low_u64_be(77), 20)])
					.expect("Should build player_table")
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
				.active_end(50)
				.claim_end(90)
				.reward_distribution(
					RewardDistributionTable::try_from(vec![50, 30]).expect("Should create table"),
				)
				.golden_duck_config(GoldenDuckConfig::Enabled(20))
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

				run_to_block(15);

				assert_ok!(TournamentAlpha::try_rank_entity_in_tournament_for(
					&SEASON_ID_1,
					&H256::from_low_u64_be(3),
					&10_u32,
					&MockRanker
				));
				assert_ok!(TournamentAlpha::try_rank_entity_in_tournament_for(
					&SEASON_ID_1,
					&H256::from_low_u64_be(7),
					&15_u32,
					&MockRanker
				));

				assert_ok!(TournamentAlpha::try_rank_entity_for_golden_duck(
					&SEASON_ID_1,
					&H256::from_low_u64_be(10),
				));

				System::assert_last_event(mock::RuntimeEvent::TournamentAlpha(
					crate::Event::EntityBecameGoldenDuck {
						season_id: SEASON_ID_1,
						tournament_id,
						entity_id: H256::from_low_u64_be(10),
					},
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
					RankingTableFor::<Test, Instance1>::try_from(vec![
						(H256::from_low_u64_be(7), 15),
						(H256::from_low_u64_be(3), 10)
					])
					.expect("Should build player_table")
				);

				assert_ok!(TournamentAlpha::try_claim_tournament_reward_for(
					&SEASON_ID_1,
					&ALICE,
					&H256::from_low_u64_be(3),
				));

				System::assert_last_event(mock::RuntimeEvent::TournamentAlpha(
					crate::Event::RankingRewardClaimed {
						season_id: SEASON_ID_1,
						tournament_id,
						entity_id: H256::from_low_u64_be(3),
						account: ALICE,
					},
				));

				assert_ok!(TournamentAlpha::try_claim_tournament_reward_for(
					&SEASON_ID_1,
					&BOB,
					&H256::from_low_u64_be(7),
				));

				System::assert_last_event(mock::RuntimeEvent::TournamentAlpha(
					crate::Event::RankingRewardClaimed {
						season_id: SEASON_ID_1,
						tournament_id,
						entity_id: H256::from_low_u64_be(7),
						account: BOB,
					},
				));

				assert_eq!(Balances::free_balance(tournament_account), 20);
				assert_eq!(Balances::free_balance(ALICE), 930);
				assert_eq!(Balances::free_balance(BOB), 1_050);

				assert_ok!(TournamentAlpha::try_claim_golden_duck_for(
					&SEASON_ID_1,
					&ALICE,
					&H256::from_low_u64_be(10),
				));

				System::assert_last_event(mock::RuntimeEvent::TournamentAlpha(
					crate::Event::GoldenDuckRewardClaimed {
						season_id: SEASON_ID_1,
						tournament_id,
						entity_id: H256::from_low_u64_be(10),
						account: ALICE,
					},
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
			.active_end(50)
			.claim_end(90)
			.reward_distribution(
				RewardDistributionTable::try_from(vec![50, 30]).expect("Should create table"),
			)
			.golden_duck_config(GoldenDuckConfig::Enabled(20));
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

			run_to_block(15);

			assert_ok!(TournamentAlpha::try_rank_entity_in_tournament_for(
				&SEASON_ID_1,
				&H256::from_low_u64_be(3),
				&10_u32,
				&MockRanker
			));

			assert_ok!(TournamentAlpha::try_rank_entity_in_tournament_for(
				&SEASON_ID_1,
				&H256::from_low_u64_be(9),
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
				RankingTableFor::<Test, Instance1>::try_from(vec![
					(H256::from_low_u64_be(9), 15),
					(H256::from_low_u64_be(3), 10)
				])
				.expect("Should build player_table")
			);

			assert_ok!(TournamentAlpha::try_claim_tournament_reward_for(
				&SEASON_ID_1,
				&ALICE,
				&H256::from_low_u64_be(3),
			));

			assert_ok!(TournamentAlpha::try_claim_tournament_reward_for(
				&SEASON_ID_1,
				&BOB,
				&H256::from_low_u64_be(9),
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
				.active_end(50)
				.claim_end(90)
				.reward_distribution(
					RewardDistributionTable::try_from(vec![50, 30]).expect("Should create table"),
				)
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

				run_to_block(15);

				assert_ok!(TournamentAlpha::try_rank_entity_in_tournament_for(
					&SEASON_ID_1,
					&H256::from_low_u64_be(3),
					&10_u32,
					&MockRanker
				));
				assert_ok!(TournamentAlpha::try_rank_entity_in_tournament_for(
					&SEASON_ID_1,
					&H256::from_low_u64_be(6),
					&15_u32,
					&MockRanker
				));

				assert_ok!(TournamentAlpha::try_rank_entity_for_golden_duck(
					&SEASON_ID_1,
					&H256::from_low_u64_be(10),
				));

				// Trying to claim reward while still in active state
				assert_noop!(
					TournamentAlpha::try_claim_tournament_reward_for(
						&SEASON_ID_1,
						&ALICE,
						&H256::from_low_u64_be(3)
					),
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

				// Trying to claim reward when the tournament already ended
				assert_noop!(
					TournamentAlpha::try_claim_tournament_reward_for(
						&SEASON_ID_1,
						&ALICE,
						&H256::from_low_u64_be(3)
					),
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
			});
		});
	}

	#[test]
	fn try_claim_tournament_rewards_fails_with_non_winner_entities() {
		ExtBuilder::default().build().execute_with(|| {
			let tournament_config = TournamentConfigFor::<Test, Instance1>::default()
				.start(10)
				.active_end(50)
				.claim_end(90)
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

				run_to_block(15);

				assert_ok!(TournamentAlpha::try_rank_entity_in_tournament_for(
					&SEASON_ID_1,
					&H256::from_low_u64_be(3),
					&10_u32,
					&MockRanker
				));
				assert_ok!(TournamentAlpha::try_rank_entity_in_tournament_for(
					&SEASON_ID_1,
					&H256::from_low_u64_be(12),
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
					TournamentAlpha::try_claim_tournament_reward_for(
						&SEASON_ID_1,
						&ALICE,
						&H256::from_low_u64_be(45)
					),
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
		.active_end(50)
		.claim_end(100)
		.initial_reward(Some(300))
		.max_reward(Some(120))
		.reward_distribution(
			RewardDistributionTable::try_from(vec![40, 25, 10])
				.expect("Should created reward table"),
		)
		.golden_duck_config(GoldenDuckConfig::Enabled(25))
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
					&entity_id,
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
				RankingTableFor::<Test, Instance1>::try_from(vec![
					(H256::from_low_u64_be(10), 120),
					(H256::from_low_u64_be(45), 30),
					(H256::from_low_u64_be(3), 22)
				])
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
					&entity_id,
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
				RankingTableFor::<Test, Instance1>::try_from(vec![
					(H256::from_low_u64_be(10), 120),
					(H256::from_low_u64_be(26), 99),
					(H256::from_low_u64_be(71), 70)
				])
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

			assert_ok!(TournamentAlpha::try_claim_tournament_reward_for(
				&SEASON_ID_1,
				&ALICE,
				&H256::from_low_u64_be(10)
			));

			assert_noop!(
				TournamentAlpha::try_claim_tournament_reward_for(
					&SEASON_ID_1,
					&BOB,
					&H256::from_low_u64_be(45)
				),
				Error::<Test, Instance1>::RankingCandidateNotInWinnerTable
			);

			assert_ok!(TournamentAlpha::try_claim_tournament_reward_for(
				&SEASON_ID_1,
				&CHARLIE,
				&H256::from_low_u64_be(26)
			));

			assert_ok!(TournamentAlpha::try_claim_tournament_reward_for(
				&SEASON_ID_1,
				&DAVE,
				&H256::from_low_u64_be(71)
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
		});
}
