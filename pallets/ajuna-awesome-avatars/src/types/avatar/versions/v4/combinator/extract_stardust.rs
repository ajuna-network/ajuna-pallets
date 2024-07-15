use super::*;

impl<T: Config> AvatarCombinator<T> {
	pub(super) fn extract_stardust(
		input_moon: WrappedForgeItem<T>,
		input_captain: WrappedForgeItem<T>,
		input_map: WrappedForgeItem<T>,
	) -> Result<(LeaderForgeOutput<T>, Vec<ForgeOutput<T>>), DispatchError> {
		let (moon_id, mut moon) = input_moon;
		let (captain_id, mut captain) = input_captain;
		let (map_id, map) = input_map;

		let mut moon_interpreter = UnprospectedMoonInterpreter::from_wrapper(&mut moon);
		let mut captain_interpreter = CaptainInterpreter::from_wrapper(&mut captain);

		// Extract all 'stardustOnMoon' from moon
		let stardust_amt = moon_interpreter.get_stardust_on_moon();
		moon_interpreter.set_stardust_on_moon(0);

		// Add the extracted stardust to the captain 'stardustOnAccount'
		let current_stardust_on_acc = captain_interpreter.get_stardust_on_account();
		let new_stardust_on_acc = current_stardust_on_acc.saturating_add(stardust_amt as u16);
		captain_interpreter.set_stardust_on_account(new_stardust_on_acc);

		// Increase 'stardustGatheredAllTime'
		let current_stardust_all_time = captain_interpreter.get_stardust_gathered_all_time();
		let new_stardust_all_time = current_stardust_all_time.saturating_add(stardust_amt as u16);
		captain_interpreter.set_stardust_gathered_all_time(new_stardust_all_time);

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

#[cfg(test)]
mod test {
	use super::*;
	use crate::mock::*;

	#[test]
	fn test_extract_stardust() {
		ExtBuilder::default().build().execute_with(|| {
			let stardust_amt = 200;
			let moon = create_random_unprospected_moon(&ALICE, stardust_amt, (0, 0));
			let captain = create_random_captain(&ALICE, 1, 100);
			let map = create_random_cluster_map(&ALICE, &[], &[], (0, 0));

			let (leader_output, mut sacrifice_output) =
				AvatarCombinator::<Test>::extract_stardust(moon, captain, map)
					.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 2);
			assert_eq!(sacrifice_output.iter().filter(|output| is_forged(output)).count(), 1);
			assert_eq!(sacrifice_output.iter().filter(|output| is_unchanged(output)).count(), 1);

			if let LeaderForgeOutput::Forged((_, leader_avatar), _) = leader_output {
				let mut wrapper = WrappedAvatar::new(leader_avatar);
				let interpreter = UnprospectedMoonInterpreter::from_wrapper(&mut wrapper);

				assert_eq!(interpreter.get_stardust_on_moon(), 0);

				let _map = sacrifice_output.pop().expect("Should contain element");
				let captain = sacrifice_output.pop().expect("Should contain element");
				if let ForgeOutput::Forged((_, item), _) = captain {
					let mut wrapper = WrappedAvatar::new(item);
					let interpreter = CaptainInterpreter::from_wrapper(&mut wrapper);

					assert_eq!(interpreter.get_stardust_on_account(), stardust_amt as u16);
					assert_eq!(interpreter.get_stardust_gathered_all_time(), stardust_amt as u16);
				} else {
					panic!("Captain should have been forged!");
				}
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}
		})
	}
}
