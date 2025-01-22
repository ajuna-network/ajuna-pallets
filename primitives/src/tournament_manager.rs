use frame_support::pallet_prelude::{DispatchError, DispatchResult};
use sp_std::cmp::Ordering;

pub trait EntityRanker {
	type EntityId;
	type Entity;

	fn can_rank(&self, entity: (&Self::EntityId, &Self::Entity)) -> bool;

	fn rank_against(
		&self,
		entity: (&Self::EntityId, &Self::Entity),
		other: (&Self::EntityId, &Self::Entity),
	) -> Ordering;
}

pub trait TournamentInspector {
	type CategoryId;
	type TournamentId;
	type TournamentConfig;
	type TournamentState;
	type AccountId;
	fn get_active_tournament_config_for(
		category_id: &Self::CategoryId,
	) -> Option<(Self::TournamentId, Self::TournamentConfig)>;

	fn get_active_tournament_state_for(category_id: &Self::CategoryId) -> Self::TournamentState;

	fn is_golden_duck_enabled_for(category_id: &Self::CategoryId) -> bool;

	fn get_treasury_account_for(category_id: &Self::CategoryId) -> Self::AccountId;
}

pub trait TournamentMutator: TournamentInspector {
	fn try_create_new_tournament_for(
		creator: &Self::AccountId,
		category_id: &Self::CategoryId,
		config: Self::TournamentConfig,
	) -> Result<Self::TournamentId, DispatchError>;

	fn try_remove_latest_tournament_for(category_id: &Self::CategoryId) -> DispatchResult;
}

pub trait TournamentRanker: TournamentInspector {
	type EntityId;
	type Entity;
	fn try_rank_entity_in_tournament_for(
		category_id: &Self::CategoryId,
		entity_id: &Self::EntityId,
		entity: &Self::Entity,
	) -> DispatchResult;

	fn try_rank_entity_for_golden_duck(
		category_id: &Self::CategoryId,
		entity_id: &Self::EntityId,
	) -> DispatchResult;
}

pub trait TournamentClaimer: TournamentRanker {
	fn try_claim_tournament_reward_for(
		category_id: &Self::CategoryId,
		account: &Self::AccountId,
		entity_id: &Self::EntityId,
	) -> DispatchResult;

	fn try_claim_golden_duck_for(
		category_id: &Self::CategoryId,
		account: &Self::AccountId,
		entity_id: &Self::EntityId,
	) -> DispatchResult;
}
pub trait TournamentBenchmarkHelper<CategoryId, TournamentConfig, AccountId, Assets> {
	fn create_category_id() -> CategoryId;

	fn create_config() -> TournamentConfig;

	fn create_entities(owner: &AccountId, count: usize) -> Vec<Assets>;
}
