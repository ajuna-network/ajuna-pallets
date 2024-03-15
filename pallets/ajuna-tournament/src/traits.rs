use frame_support::{pallet_prelude::*, PalletId};
use sp_runtime::{traits::AtLeast32BitUnsigned, SaturatedConversion, TypeId};

pub const REWARD_TABLE_MAX_LENGTH: u32 = 11;

pub const MAX_PLAYERS: u32 = 10;

pub type TournamentId = u32;

pub type RewardTable = BoundedVec<u8, ConstU32<REWARD_TABLE_MAX_LENGTH>>;

pub type PlayerTable<T> = BoundedVec<T, ConstU32<MAX_PLAYERS>>;

#[derive(Decode, Encode, Clone)]
pub struct TournamentTreasuryAccount {
	// bytes representing the data
	pub bytes: Vec<u8>,
}

impl TournamentTreasuryAccount {
	pub fn new_with<SeasonId: AtLeast32BitUnsigned>(
		pallet_id: PalletId,
		season_id: SeasonId,
		tournament_id: TournamentId,
	) -> Self {
		let mut bytes = pallet_id.0.to_vec();
		bytes.extend_from_slice(b"/");
		bytes.extend(season_id.saturated_into::<u128>().to_string().bytes());
		bytes.extend(b"/");
		bytes.extend(tournament_id.to_string().bytes());
		Self { bytes }
	}
}

impl TypeId for TournamentTreasuryAccount {
	// I don't know yet the full implications of the TypeId.
	//
	// However, this is the same type that is used for the pallet id.
	// I believe this is used by indexers to identify accounts from pallet
	// from pallet instances, hence we should use the same identifier as the
	// PalletId.
	const TYPE_ID: [u8; 4] = *b"modl";
}

pub trait EntityRank {
	type Entity: Member + PartialOrd + Ord;
	fn rank_against(&self, entity: &Self::Entity, other: &Self::Entity) -> sp_std::cmp::Ordering;
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

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
pub enum GoldenDuckState<AccountId, EntityId> {
	#[default]
	Disabled,
	Enabled(Option<(AccountId, EntityId)>),
}

pub trait TournamentInspector<SeasonId, BlockNumber, Balance> {
	fn get_active_tournament_for(
		season_id: &SeasonId,
	) -> Option<TournamentConfig<BlockNumber, Balance>>;

	fn is_golden_duck_enabled_for(season_id: &SeasonId) -> bool;
}

pub trait TournamentMutator<AccountId, SeasonId, BlockNumber, Balance> {
	fn try_create_new_tournament_for(
		creator: &AccountId,
		season_id: &SeasonId,
		config: TournamentConfig<BlockNumber, Balance>,
	) -> Result<TournamentId, DispatchError>;

	fn try_start_next_tournament_for(season_id: &SeasonId) -> DispatchResult;

	fn try_finish_active_tournament_for(season_id: &SeasonId) -> DispatchResult;
}

pub trait TournamentRanker<AccountId, SeasonId, RankCategory, Entity, EntityId> {
	fn try_rank_entity_in_tournament_for<R>(
		account: &AccountId,
		season_id: &SeasonId,
		category: &RankCategory,
		entity: &Entity,
		ranker: &R,
	) -> DispatchResult
	where
		R: EntityRank<Entity = Entity>;

	fn try_rank_entity_for_golden_duck(
		account: &AccountId,
		season_id: &SeasonId,
		entity_id: &EntityId,
	) -> DispatchResult
	where
		EntityId: Member + PartialOrd + Ord;
}
