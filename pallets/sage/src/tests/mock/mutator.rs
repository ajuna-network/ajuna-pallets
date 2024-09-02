use super::*;

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, Default, PartialEq, Eq)]
pub struct MockAssetData {
	pub(crate) asset_type: u8,
	pub(crate) asset_subtype: u8,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, PartialEq, Eq)]
pub enum MockMutationError {
	MutationError,
}
impl From<MockMutationError> for DispatchError {
	fn from(_value: MockMutationError) -> Self {
		Self::Other("Mutation error!")
	}
}

pub struct MockStateMutator;

impl StateMutator<H256, MockBlockNumber> for MockStateMutator {
	type AccountId = MockAccountId;
	type Randomness = Randomness;
	type MutationId = u16;
	type AssetId = u16;
	type AssetData = MockAssetData;
	type MutationError = MockMutationError;

	fn try_mutate_state(
		_account: &Self::AccountId,
		mutation_id: &Self::MutationId,
		input_assets: Vec<MutatorInput<Self::AssetId, Self::AssetData, MockBlockNumber>>,
	) -> Result<
		Vec<MutatorOutput<Self::AssetId, Self::AssetData, MockBlockNumber>>,
		Self::MutationError,
	> {
		if *mutation_id == 0 {
			Ok(input_assets
				.into_iter()
				.map(|(input_id, _)| MutatorOutput::Unchanged(input_id))
				.collect())
		} else {
			Ok(input_assets
				.into_iter()
				.map(|(input_id, _)| MutatorOutput::Consumed(input_id))
				.collect())
		}
	}
}
