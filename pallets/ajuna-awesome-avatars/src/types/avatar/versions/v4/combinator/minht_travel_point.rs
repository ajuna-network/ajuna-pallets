use super::*;

impl<T: Config> AvatarCombinator<T> {
	pub(super) fn mint_travel_point(
		input_moon: WrappedForgeItem<T>,
		input_captain: WrappedForgeItem<T>,
		input_map: WrappedForgeItem<T>,
	) -> Result<(LeaderForgeOutput<T>, Vec<ForgeOutput<T>>), DispatchError> {
		todo!()
	}
}
