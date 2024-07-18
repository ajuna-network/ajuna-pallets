use super::*;

impl<T: Config> AvatarCombinator<T> {
	pub(super) fn upgrade_ship(
		input_ship: WrappedForgeItem<T>,
		input_resource: WrappedForgeItem<T>,
		input_captain: WrappedForgeItem<T>,
		input_map: WrappedForgeItem<T>,
	) -> Result<(LeaderForgeOutput<T>, Vec<ForgeOutput<T>>), DispatchError> {
		todo!()
	}
}
