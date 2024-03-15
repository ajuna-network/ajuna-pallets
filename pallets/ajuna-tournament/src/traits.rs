use frame_support::{pallet_prelude::*, PalletId};
use parity_scale_codec::{Error, Input};
use sp_runtime::TypeId;

pub const REWARD_TABLE_MAX_LENGTH: u32 = 11;

pub const MAX_PLAYERS: u32 = 10;

pub type TournamentId = u32;

pub type RewardTable = BoundedVec<u8, ConstU32<REWARD_TABLE_MAX_LENGTH>>;

pub type PlayerTable<T> = BoundedVec<T, ConstU32<MAX_PLAYERS>>;

#[derive(Clone, PartialEq, Eq)]
pub struct TournamentTreasuryAccount<SeasonId> {
	pub pallet_id: PalletId,
	pub season_id: SeasonId,
	pub tournament_id: TournamentId,
}

type TreasuryAccountEncodec<'a, SeasonId> =
	(&'a PalletId, &'a [u8; 1], &'a SeasonId, &'a [u8; 1], &'a TournamentId);

type TreasuryAccountDecodec<SeasonId> = (PalletId, [u8; 1], SeasonId, [u8; 1], TournamentId);

impl<SeasonId: Encode> Encode for TournamentTreasuryAccount<SeasonId> {
	fn encode(&self) -> Vec<u8> {
		// This codec will fit into the indexers rendering design such that we can
		// see the treasury accounts as "<pallet_id>/season_id/tournament_id".
		let data: TreasuryAccountEncodec<SeasonId> =
			(&self.pallet_id, b"/", &self.season_id, &b"/", &self.tournament_id);
		data.encode()
	}
}

impl<SeasonId: Decode> Decode for TournamentTreasuryAccount<SeasonId> {
	fn decode<I: Input>(input: &mut I) -> Result<Self, Error> {
		let tuple = TreasuryAccountDecodec::decode(input)?;
		Ok(Self::new(tuple.0, tuple.2, tuple.4))
	}
}

impl<SeasonId> TournamentTreasuryAccount<SeasonId> {
	pub fn new(pallet_id: PalletId, season_id: SeasonId, tournament_id: TournamentId) -> Self {
		Self { pallet_id, season_id, tournament_id }
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

#[cfg(test)]
mod tests {
	use crate::TournamentTreasuryAccount;
	use frame_support::PalletId;
	use parity_scale_codec::{Decode, Encode};

	#[test]
	fn tournament_treasury_account_codec_works() {
		let pallet_id = PalletId(*b"ajn/trsy");
		let tournament_account = TournamentTreasuryAccount::new(pallet_id, 2u32, 4u32);

		let encoded = tournament_account.encode();
		let decoded = TournamentTreasuryAccount::<u32>::decode(&mut encoded.as_slice()).unwrap();

		// PalletId does not implement debug...
		assert_eq!(encoded, decoded.encode())
	}
}
