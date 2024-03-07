use frame_support::pallet_prelude::*;
use frame_system::pallet_prelude::BlockNumberFor;

pub const REWARD_TABLE_MAX_LENGTH: u32 = 5;

pub const MAX_PLAYERS: u32 = 3;

pub type TournamentId = u32;

pub type RewardTable = BoundedVec<u8, ConstU32<REWARD_TABLE_MAX_LENGTH>>;

pub type PlayerTable<T> = BoundedVec<T, ConstU32<MAX_PLAYERS>>;

pub trait Identifier<Id> {
	fn get_id() -> Id;
}

pub trait Ranker<Id> {
	type Ordering: Ord;
	type Category: Member + Parameter + MaxEncodedLen + Copy;
	type Entity: Parameter + Member + Identifier<Id>;
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, PartialEq)]
pub struct TournamentConfig<BlockNumber, Balance> {
	pub start: BlockNumber,
	pub end: BlockNumber,
	pub initial_reward: Option<Balance>,
	pub max_reward: Option<Balance>,
	pub take_fee_percentage: Option<u8>,
	pub reward_table: RewardTable,
	pub max_players: u32,
}

pub trait TournamentInspector<SeasonId, BlockNumber, Balance> {
	fn get_active_tournament_for(
		season_id: &SeasonId,
	) -> Option<TournamentConfig<BlockNumber, Balance>>;
}

pub trait TournamentMutator<SeasonId, BlockNumber, Balance> {
	fn try_create_new_tournament_for(
		season_id: &SeasonId,
		config: TournamentConfig<BlockNumber, Balance>,
	) -> Result<TournamentId, DispatchError>;

	fn try_start_next_tournament_for(season_id: &SeasonId) -> DispatchResult;

	fn try_finish_active_tournament_for(season_id: &SeasonId) -> DispatchResult;
}

pub trait TournamentRanker<SeasonId, AccountId, RC, E, EI> {
	fn try_rank_entity_in_tournament_for<R>(
		season_id: &SeasonId,
		account: &AccountId,
		ranker: &R,
		entity: &E,
	) -> DispatchResult
	where
		R: Ranker<EI, Category = RC, Entity = E>;
}
