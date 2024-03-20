use frame_support::{pallet_prelude::*, PalletId};
use parity_scale_codec::{Error, Input};
use sp_runtime::TypeId;

pub const REWARD_TABLE_MAX_LENGTH: u32 = 11;

pub const MAX_PLAYERS: u32 = 10;

pub const MAX_ENABLED_SEASONS: u32 = 15;

pub const MIN_TOURNAMENT_BLOCK_SPACING: u32 = 10_000;

pub type TournamentId = u32;

pub type RewardTable = BoundedVec<u8, ConstU32<REWARD_TABLE_MAX_LENGTH>>;

pub type PlayerTable<T> = BoundedVec<T, ConstU32<MAX_PLAYERS>>;

pub type SeasonSet<T> = BoundedBTreeSet<T, ConstU32<MAX_ENABLED_SEASONS>>;

#[derive(Clone, PartialEq, Eq)]
pub struct TournamentTreasuryAccount<SeasonId> {
	pub pallet_id: PalletId,
	pub season_id: SeasonId,
}

type TreasuryAccountEncodec<'a, SeasonId> = (&'a PalletId, &'a [u8; 1], &'a SeasonId);

type TreasuryAccountDecodec<SeasonId> = (PalletId, [u8; 1], SeasonId);

impl<SeasonId: Encode> Encode for TournamentTreasuryAccount<SeasonId> {
	fn encode(&self) -> Vec<u8> {
		// This codec will fit into the indexers rendering design such that we can
		// see the treasury accounts as "<pallet_id>/season_id".
		let data: TreasuryAccountEncodec<SeasonId> = (&self.pallet_id, b"/", &self.season_id);
		data.encode()
	}
}

impl<SeasonId: Decode> Decode for TournamentTreasuryAccount<SeasonId> {
	fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
		let tuple = TreasuryAccountDecodec::decode(input)?;
		Ok(Self::new(tuple.0, tuple.2))
	}
}

impl<SeasonId> TournamentTreasuryAccount<SeasonId> {
	pub fn new(pallet_id: PalletId, season_id: SeasonId) -> Self {
		Self { pallet_id, season_id }
	}
}

impl<SeasonId> TypeId for TournamentTreasuryAccount<SeasonId> {
	// I don't know yet the full implications of the TypeId.
	//
	// However, this is the same type that is used for the pallet id.
	// I believe this is used by indexers to identify accounts from pallet
	// instances, hence we should use the same identifier as the PalletId.
	const TYPE_ID: [u8; 4] = *b"modl";
}

pub trait EntityRank {
	type Entity: Member + PartialOrd + Ord;
	fn rank_against(&self, entity: &Self::Entity, other: &Self::Entity) -> sp_std::cmp::Ordering;
}

/// Describes the configuration of a given tournament
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, PartialEq)]
pub struct TournamentConfig<BlockNumber, Balance> {
	/// Block in which the tournament starts.
	pub start: BlockNumber,
	/// Block in which the tournament finishes.
	pub end: BlockNumber,
	/// Block in which the claiming period for tournament rewards end.
	pub claim_end: BlockNumber,
	/// Optional funds that will be transferred by the tournament creator
	/// to the tournament's treasury on success.
	pub initial_reward: Option<Balance>,
	/// Optional cap to the maximum amount of funds to be distributed among the winners.
	pub max_reward: Option<Balance>,
	/// TODO: Define use
	pub take_fee_percentage: Option<u8>,
	/// Distribution table that indicates how the reward should be split among the tournament
	/// winners in the form of [1st %, 2nd %, 3rd %, ....]
	pub reward_table: RewardTable,
	/// Maximum amount of players that can be ranked in the tournament
	pub max_players: u32,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
pub enum GoldenDuckState<EntityId> {
	#[default]
	Disabled,
	Enabled(u8, Option<EntityId>),
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq)]
pub enum TournamentState<Balance> {
	#[default]
	Inactive,
	ActivePeriod(TournamentId),
	ClaimPeriod(TournamentId, Balance),
}

pub trait TournamentInspector<SeasonId, BlockNumber, Balance, AccountId> {
	fn get_active_tournament_config_for(
		season_id: &SeasonId,
	) -> Option<TournamentConfig<BlockNumber, Balance>>;

	fn is_golden_duck_enabled_for(season_id: &SeasonId) -> bool;

	fn get_treasury_account_for(season_id: &SeasonId) -> AccountId;
}

pub trait TournamentMutator<AccountId, SeasonId, BlockNumber, Balance> {
	fn try_create_new_tournament_for(
		creator: &AccountId,
		season_id: &SeasonId,
		config: TournamentConfig<BlockNumber, Balance>,
	) -> Result<TournamentId, DispatchError>;

	fn try_enable_tournament_processing_for_season(season_id: &SeasonId) -> DispatchResult;

	fn try_disable_tournament_processing_for_season(season_id: &SeasonId) -> DispatchResult;
}

pub trait TournamentRanker<SeasonId, Entity, EntityId> {
	fn try_rank_entity_in_tournament_for<R>(
		season_id: &SeasonId,
		entity: &Entity,
		ranker: &R,
	) -> DispatchResult
	where
		R: EntityRank<Entity = Entity>;

	fn try_rank_entity_for_golden_duck(
		season_id: &SeasonId,
		entity_id: &EntityId,
	) -> DispatchResult
	where
		EntityId: Member + PartialOrd + Ord;
}

pub trait TournamentClaimer<SeasonId, AccountId, Entity, EntityId> {
	fn try_claim_tournament_reward_for(
		season_id: &SeasonId,
		account: &AccountId,
		entity: &Entity,
	) -> DispatchResult;

	fn try_claim_golden_duck_for(
		season_id: &SeasonId,
		account: &AccountId,
		entity_id: &EntityId,
	) -> DispatchResult;
}

#[cfg(test)]
mod tests {
	use crate::TournamentTreasuryAccount;
	use frame_support::PalletId;
	use parity_scale_codec::{Decode, Encode};

	#[test]
	fn tournament_treasury_account_codec_works() {
		let pallet_id = PalletId(*b"ajn/trsy");
		let tournament_account = TournamentTreasuryAccount::new(pallet_id, 2u32);

		let encoded = tournament_account.encode();
		let decoded = TournamentTreasuryAccount::<u32>::decode(&mut encoded.as_slice()).unwrap();

		// PalletId does not implement debug...
		assert_eq!(encoded, decoded.encode())
	}
}
