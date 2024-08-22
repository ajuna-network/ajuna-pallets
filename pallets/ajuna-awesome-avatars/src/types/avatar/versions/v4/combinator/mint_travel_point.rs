use super::*;

impl<T: Config> AvatarCombinator<T> {
	pub(super) fn mint_travel_point(
		input_moon: WrappedForgeItem<T>,
		input_captain: WrappedForgeItem<T>,
		input_map: WrappedForgeItem<T>,
		season_id: SeasonId,
		current_block: BlockNumberFor<T>,
	) -> Result<(LeaderForgeOutput<T>, Vec<ForgeOutput<T>>), DispatchError> {
		let (moon_id, mut moon) = input_moon;
		let (captain_id, mut captain) = input_captain;
		let (map_id, map) = input_map;

		let mut moon_interpreter = MoonInterpreter::from_wrapper(&mut moon);
		let mut captain_interpreter = CaptainInterpreter::from_wrapper(&mut captain);

		// Decrease stardustOnAccount by 3
		let current_stardust_on_acc = captain_interpreter.get_stardust_on_account();
		let new_stardust_on_acc = current_stardust_on_acc.saturating_sub(STARDUST_PER_TRAVEL_POINT);
		captain_interpreter.set_stardust_on_account(new_stardust_on_acc);

		// Mint TravelPoint
		let dna = MinterV4::<T>::generate_empty_dna::<32>()?;
		let coord_x = moon_interpreter.get_coord(Coord::X);
		let coord_y = moon_interpreter.get_coord(Coord::Y);
		let minted_travel_point = AvatarBuilder::with_dna(season_id, dna, current_block)
			.structured_into_travel_point(coord_x, coord_y)
			.build();

		/// TODO: Decrease blockMintedPeriod
		let current_block_minted_period = moon_interpreter.get_block_mints_period();

		// Decrease mintedTravelPoints by 1
		let current_minted_travel_points = moon_interpreter.get_minted_travel_points();
		let new_minted_travel_points =
			current_minted_travel_points.saturating_sub(MOON_MINTED_TRAVEL_POINTS_DEC);
		moon_interpreter.set_minted_travel_points(new_minted_travel_points);

		// Increase travelPointsMinted by 1
		let current_travel_points_minted = captain_interpreter.get_travel_points_minted();
		let new_travel_points_minted =
			current_travel_points_minted.saturating_add(CAPTAIN_MINTED_TRAVEL_POINTS_INC);
		captain_interpreter.set_travel_points_minted(new_travel_points_minted);

		// TODO: Something with the cluster

		let output_vec: Vec<ForgeOutput<T>> = [
			ForgeOutput::Forged((captain_id, captain.unwrap()), 0),
			ForgeOutput::Unchanged((map_id, map.unwrap())),
			ForgeOutput::Minted(minted_travel_point),
		]
		.into_iter()
		.collect();

		Ok((LeaderForgeOutput::Forged((moon_id, moon.unwrap()), 0), output_vec))
	}
}
