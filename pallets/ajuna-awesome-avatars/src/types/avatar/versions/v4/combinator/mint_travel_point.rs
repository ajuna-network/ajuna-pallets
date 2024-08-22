use super::*;

impl<T: Config> AvatarCombinator<T> {
	pub(super) fn mint_travel_point(
		input_moon: WrappedForgeItem<T>,
		input_captain: WrappedForgeItem<T>,
		input_map: WrappedForgeItem<T>,
	) -> Result<(LeaderForgeOutput<T>, Vec<ForgeOutput<T>>), DispatchError> {
		let (moon_id, mut moon) = input_moon;
		let (captain_id, mut captain) = input_captain;
		let (map_id, map) = input_map;

		let mut moon_interpreter = UnprospectedMoonInterpreter::from_wrapper(&mut moon);
		let mut captain_interpreter = CaptainInterpreter::from_wrapper(&mut captain);

		// Decrease stardust by 3
		let current_stardust_on_acc = captain_interpreter.get_stardust_on_account();
		let new_stardust_on_acc = current_stardust_on_acc.saturating_sub(STARDUST_PER_TRAVEL_POINT);
		captain_interpreter.set_stardust_on_account(new_stardust_on_acc);

		// TODO: Something with the cluster

		let output_vec: Vec<ForgeOutput<T>> = [
			ForgeOutput::Forged((captain_id, captain.unwrap()), 0),
			ForgeOutput::Unchanged((map_id, map.unwrap())),
		]
			.into_iter()
			.collect();

		Ok((LeaderForgeOutput::Forged((moon_id, moon.unwrap()), 0), output_vec))
	}
}
