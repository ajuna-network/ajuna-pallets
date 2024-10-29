use super::{TournamentConfig, TournamentId, TournamentState};
use sp_runtime::{traits::Member, DispatchError, DispatchResult};

pub trait EntityRank {
	type EntityId: Member;
	type Entity: Member;

	fn can_rank(&self, entity: (&Self::EntityId, &Self::Entity)) -> bool;

	fn rank_against(
		&self,
		entity: (&Self::EntityId, &Self::Entity),
		other: (&Self::EntityId, &Self::Entity),
	) -> sp_std::cmp::Ordering;
}
pub trait TournamentInspector<CategoryId, BlockNumber, Balance, AccountId, Ranker> {
	fn get_active_tournament_config_for(
		category_id: &CategoryId,
	) -> Option<(TournamentId, TournamentConfig<BlockNumber, Balance, Ranker>)>;

	fn get_active_tournament_state_for(category_id: &CategoryId) -> TournamentState<Balance>;

	fn is_golden_duck_enabled_for(category_id: &CategoryId) -> bool;

	fn get_treasury_account_for(category_id: &CategoryId) -> AccountId;
}

pub trait TournamentMutator<AccountId, CategoryId, BlockNumber, Balance, Ranker> {
	fn try_create_new_tournament_for(
		creator: &AccountId,
		category_id: &CategoryId,
		config: TournamentConfig<BlockNumber, Balance, Ranker>,
	) -> Result<TournamentId, DispatchError>;

	fn try_remove_latest_tournament_for(category_id: &CategoryId) -> DispatchResult;
}

pub trait TournamentRanker<CategoryId, Entity, EntityId> {
	fn try_rank_entity_in_tournament_for(
		category_id: &CategoryId,
		entity_id: &EntityId,
		entity: &Entity,
	) -> DispatchResult;

	fn try_rank_entity_for_golden_duck(
		category_id: &CategoryId,
		entity_id: &EntityId,
	) -> DispatchResult
	where
		EntityId: Member + PartialOrd + Ord;
}

pub trait TournamentClaimer<CategoryId, AccountId, EntityId> {
	fn try_claim_tournament_reward_for(
		category_id: &CategoryId,
		account: &AccountId,
		entity_id: &EntityId,
	) -> DispatchResult;

	fn try_claim_golden_duck_for(
		category_id: &CategoryId,
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
