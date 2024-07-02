mod extract_stardust;

use super::{utils::*, *};

pub(super) struct AvatarCombinator<T: Config>(pub PhantomData<T>);

impl<T: Config> AvatarCombinator<T> {
	pub(super) fn combine_avatars_in(
		forge_type: ForgeType,
		season_id: SeasonId,
		_season: &SeasonOf<T>,
		leader: WrappedForgeItem<T>,
		sacrifices: Vec<WrappedForgeItem<T>>,
		hash_provider: &mut HashProvider<T, 32>,
	) -> Result<(LeaderForgeOutput<T>, Vec<ForgeOutput<T>>), DispatchError> {
		match forge_type {
			ForgeType::ExtractStardust =>
				Self::extract_stardust(leader, sacrifices, season_id, hash_provider),
			ForgeType::None => Self::forge_none(leader, sacrifices),
		}
	}

	fn forge_none(
		input_leader: WrappedForgeItem<T>,
		input_sacrifices: Vec<WrappedForgeItem<T>>,
	) -> Result<(LeaderForgeOutput<T>, Vec<ForgeOutput<T>>), DispatchError> {
		Ok((
			LeaderForgeOutput::Unchanged((input_leader.0, input_leader.1.unwrap())),
			input_sacrifices
				.into_iter()
				.map(|sacrifice| ForgeOutput::Unchanged((sacrifice.0, sacrifice.1.unwrap())))
				.collect(),
		))
	}
}
