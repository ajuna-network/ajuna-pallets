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
		let season_id = 1;
		let other_season_id = 2;
		let tournament_id = 3;
		let tournament_config = TournamentConfigFor::<Test, Instance1>::default();
		ExtBuilder::default().build().execute_with(|| {
			ActiveTournaments::<Test, Instance1>::insert(season_id, tournament_id);
			Tournaments::<Test, Instance1>::insert(
				season_id,
				tournament_id,
				tournament_config.clone(),
			);

			assert_eq!(
				TournamentAlpha::get_active_tournament_for(&season_id),
				Some(tournament_config)
			);
			assert_eq!(TournamentAlpha::get_active_tournament_for(&other_season_id), None);
		});
	}

	#[test]
	fn get_active_tournament_doesnt_apply_to_different_instance() {
		let season_id = 1;
		let tournament_id = 3;
		let tournament_config = TournamentConfigFor::<Test, Instance1>::default();
		ExtBuilder::default().build().execute_with(|| {
			ActiveTournaments::<Test, Instance1>::insert(season_id, tournament_id);
			Tournaments::<Test, Instance1>::insert(
				season_id,
				tournament_id,
				tournament_config.clone(),
			);

			assert_eq!(TournamentBeta::get_active_tournament_for(&season_id), None);
		});
	}
}

mod tournament_mutator {
	use super::*;

	#[test]
	fn try_create_new_tournament_works() {
		let season_id = 1;
		let tournament_config = TournamentConfigFor::<Test, Instance1>::default();
		ExtBuilder::default().build().execute_with(|| {
			let result = TournamentAlpha::try_create_new_tournament_for(
				&season_id,
				tournament_config.clone(),
			);
			assert_ok!(result);
			let tournament_id = result.unwrap();

			System::assert_last_event(mock::RuntimeEvent::TournamentAlpha(
				crate::Event::TournamentCreated { season_id, tournament_id },
			));

			assert_eq!(
				Tournaments::<Test, Instance1>::get(season_id, tournament_id),
				Some(tournament_config)
			);
			assert_eq!(NextTournamentIds::<Test, Instance1>::get(season_id), tournament_id);
		});
	}

	#[test]
	fn try_create_new_tournament_fails_for_invalid_configurations() {
		let season_id = 1;
		ExtBuilder::default().build().execute_with(|| {
			let tournament_config_1 =
				TournamentConfigFor::<Test, Instance1>::default().start(10).end(5);

			assert_noop!(
				TournamentAlpha::try_create_new_tournament_for(&season_id, tournament_config_1),
				Error::<Test, Instance1>::InvalidTournamentConfig
			);

			let tournament_config_2 =
				TournamentConfigFor::<Test, Instance1>::default().max_players(0);

			assert_noop!(
				TournamentAlpha::try_create_new_tournament_for(&season_id, tournament_config_2),
				Error::<Test, Instance1>::InvalidTournamentConfig
			);

			let tournament_config_3 =
				TournamentConfigFor::<Test, Instance1>::default().max_players(MAX_PLAYERS + 1);

			assert_noop!(
				TournamentAlpha::try_create_new_tournament_for(&season_id, tournament_config_3),
				Error::<Test, Instance1>::InvalidTournamentConfig
			);

			let tournament_config_4 = TournamentConfigFor::<Test, Instance1>::default()
				.reward_table(bounded_vec![80, 30, 20]);

			assert_noop!(
				TournamentAlpha::try_create_new_tournament_for(&season_id, tournament_config_4),
				Error::<Test, Instance1>::InvalidTournamentConfig
			);
		});
	}

	#[test]
	fn try_start_new_tournament_works() {
		let season_id = 1;
		let tournament_config = TournamentConfigFor::<Test, Instance1>::default();
		ExtBuilder::default().build().execute_with(|| {
			let result = TournamentAlpha::try_create_new_tournament_for(
				&season_id,
				tournament_config.clone(),
			);
			assert_ok!(result);
			let tournament_id_1 = result.unwrap();

			assert_ok!(TournamentAlpha::try_create_new_tournament_for(
				&season_id,
				tournament_config.clone(),
			));

			assert_ok!(TournamentAlpha::try_start_next_tournament_for(&season_id));

			System::assert_last_event(mock::RuntimeEvent::TournamentAlpha(
				crate::Event::TournamentStarted { season_id, tournament_id: tournament_id_1 },
			));

			assert_eq!(ActiveTournaments::<Test, Instance1>::get(season_id), Some(tournament_id_1));
			assert_eq!(LatestTournaments::<Test, Instance1>::get(season_id), None);
		});
	}

	#[test]
	fn try_start_new_tournament_fails_with_already_active_tournament() {
		let season_id = 1;
		let tournament_config = TournamentConfigFor::<Test, Instance1>::default();
		ExtBuilder::default().build().execute_with(|| {
			let result = TournamentAlpha::try_create_new_tournament_for(
				&season_id,
				tournament_config.clone(),
			);
			assert_ok!(result);
			let tournament_id_1 = result.unwrap();

			assert_ok!(TournamentAlpha::try_start_next_tournament_for(&season_id));

			let result = TournamentAlpha::try_create_new_tournament_for(
				&season_id,
				tournament_config.clone(),
			);
			assert_ok!(result);
			let tournament_id_2 = result.unwrap();

			assert_noop!(
				TournamentAlpha::try_start_next_tournament_for(&season_id),
				Error::<Test, Instance1>::AnotherTournamentAlreadyActiveForSeason
			);
		});
	}

	#[test]
	fn try_finish_active_tournament_works() {
		let season_id = 1;
		let tournament_config = TournamentConfigFor::<Test, Instance1>::default().start(1).end(5);
		ExtBuilder::default().build().execute_with(|| {
			let result = TournamentAlpha::try_create_new_tournament_for(
				&season_id,
				tournament_config.clone(),
			);
			assert_ok!(result);
			let tournament_id = result.unwrap();

			assert_ok!(TournamentAlpha::try_start_next_tournament_for(&season_id));

			run_to_block(6);

			assert_ok!(TournamentAlpha::try_finish_active_tournament_for(&season_id));

			System::assert_last_event(mock::RuntimeEvent::TournamentAlpha(
				crate::Event::TournamentEnded { season_id, tournament_id },
			));

			assert_eq!(ActiveTournaments::<Test, Instance1>::get(season_id), None);
			assert_eq!(LatestTournaments::<Test, Instance1>::get(season_id), Some(tournament_id));
		});
	}

	#[test]
	fn try_finish_active_tournament_fails_when_non_active_tournament() {
		let season_id = 1;
		ExtBuilder::default().build().execute_with(|| {
			assert_noop!(
				TournamentAlpha::try_finish_active_tournament_for(&season_id),
				Error::<Test, Instance1>::NoActiveTournamentForSeason
			);
		});
	}
}

mod tournament_ranker {
	use super::*;

	#[test]
	fn tournament_ranker_works() {
		ExtBuilder::default().build().execute_with(|| {});
	}
}

#[test]
fn test_full_tournament_workflow() {

}
