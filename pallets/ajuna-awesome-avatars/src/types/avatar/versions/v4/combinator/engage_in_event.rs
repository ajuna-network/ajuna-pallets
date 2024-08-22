use super::*;

impl<T: Config> AvatarCombinator<T> {
	pub(super) fn engage_in_event(
		input_moon: WrappedForgeItem<T>,
		input_captain: WrappedForgeItem<T>,
		input_map: WrappedForgeItem<T>,
		hash_provider: &mut HashProvider<T, 32>,
	) -> Result<(LeaderForgeOutput<T>, Vec<ForgeOutput<T>>), DispatchError> {
		let (moon_id, mut moon) = input_moon;
		let (captain_id, mut captain) = input_captain;
		let (map_id, map) = input_map;

		let mut moon_interpreter = UnprospectedMoonInterpreter::from_wrapper(&mut moon);
		let mut captain_interpreter = CaptainInterpreter::from_wrapper(&mut captain);

		// TODO: How de we send the questions and answers as parameters? Is it even possible?

		let prospecting_minutes_left = moon_interpreter.get_prospecting_minutes_left();
		moon_interpreter.set_prospecting_minutes_left(prospecting_minutes_left.saturating_sub(5));

		// nextEventAt
		// eventType
		// eventLength

		// TODO: How de we send the decisions as parameters? Is it even possible?
		let num_good_decisions = moon_interpreter.get_number_of_decisions(DecisionType::Good);
		moon_interpreter.set_number_of_decisions(DecisionType::Good, num_good_decisions);
		let num_bad_decisions = moon_interpreter.get_number_of_decisions(DecisionType::Bad);
		moon_interpreter.set_number_of_decisions(DecisionType::Bad, num_bad_decisions);

		let output_vec: Vec<ForgeOutput<T>> = [
			ForgeOutput::Unchanged((captain_id, captain.unwrap())),
			ForgeOutput::Unchanged((map_id, map.unwrap())),
		]
		.into_iter()
		.collect();

		Ok((LeaderForgeOutput::Forged((moon_id, moon.unwrap()), 0), output_vec))
	}
}
