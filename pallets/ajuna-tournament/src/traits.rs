use super::{TournamentConfig, TournamentId};
use sp_runtime::{traits::Member, DispatchError, DispatchResult};

pub trait EntityRank {
	type EntityId: Member + PartialOrd + Ord;
	type Entity: Member + PartialOrd + Ord;

	fn rank_against(
		&self,
		entity: (&Self::EntityId, &Self::Entity),
		other: (&Self::EntityId, &Self::Entity),
	) -> sp_std::cmp::Ordering;
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
}

pub trait TournamentRanker<SeasonId, Entity, EntityId> {
	fn try_rank_entity_in_tournament_for<R>(
		season_id: &SeasonId,
		entity_id: &EntityId,
		entity: &Entity,
		ranker: &R,
	) -> DispatchResult
	where
		R: EntityRank<EntityId = EntityId, Entity = Entity>;

	fn try_rank_entity_for_golden_duck(
		season_id: &SeasonId,
		entity_id: &EntityId,
	) -> DispatchResult
	where
		EntityId: Member + PartialOrd + Ord;
}

pub trait TournamentClaimer<SeasonId, AccountId, EntityId> {
	fn try_claim_tournament_reward_for(
		season_id: &SeasonId,
		account: &AccountId,
		entity_id: &EntityId,
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
