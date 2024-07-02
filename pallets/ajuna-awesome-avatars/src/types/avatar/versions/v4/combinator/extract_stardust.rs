use super::*;

impl<T: Config> AvatarCombinator<T> {
	pub(super) fn extract_stardust(
		input_leader: WrappedForgeItem<T>,
		input_sacrifices: Vec<WrappedForgeItem<T>>,
		season_id: SeasonId,
		hash_provider: &mut HashProvider<T, 32>,
	) -> Result<(LeaderForgeOutput<T>, Vec<ForgeOutput<T>>), DispatchError> {
		todo!()
	}
}
