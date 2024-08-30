use super::*;
use parity_scale_codec::MaxEncodedLen;
use sp_runtime::DispatchError;

/// Represent an individual asset going into the mutation process
pub type MutatorInput<AssetId, AssetData, BlockNumber> =
	(AssetId, asset::Asset<AssetData, BlockNumber>);

/// Indicates the possible outcomes an asset may end up after a successful state mutation
pub enum MutatorOutput<AssetId, AssetData, BlockNumber>
where
	AssetData: Parameter + Member,
{
	/// The asset was mutated in some way, meaning its internal state has changed
	Mutated(MutatorInput<AssetId, AssetData, BlockNumber>),
	/// The has been created through the mutation process
	Created(asset::Asset<AssetData, BlockNumber>),
	/// The asset has been consumed, indicating that it no longer exists
	Consumed(asset::Asset<AssetData, BlockNumber>),
	/// The asset has remained unchanged through the mutation process
	Unchanged(MutatorInput<AssetId, AssetData, BlockNumber>),
}

pub trait StateMutator<BlockNumber> {
	type AccountId;
	type MutationId: Parameter;
	type AssetId: Parameter + Member + MaxEncodedLen;
	type AssetData: Parameter + Member + MaxEncodedLen;
	type MutationError: Into<DispatchError>;

	fn try_mutate_state(
		account: Self::AccountId,
		mutation_id: Self::MutationId,
		asset_ids: &[MutatorInput<Self::AssetId, Self::AssetData, BlockNumber>],
	) -> Result<Vec<MutatorOutput<Self::AssetId, Self::AssetData, BlockNumber>>, Self::MutationError>;
}
