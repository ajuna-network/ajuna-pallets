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

		// Extract all 'stardustOnMoon' from moon
		let stardust_amt = moon.get_spec::<u8>(SpecIdx::Byte7);
		moon.set_spec(SpecIdx::Byte7, 0);

		// Add the extracted stardust to the captain 'stardustOnAccount'
		// We need to write each bit at its proper location
		let current_stardust_on_acc = {
			let mut base = 0_u16;

			base |= (captain.get_spec::<u8>(SpecIdx::Byte6) & 0b_0000_0011) as u16;
			base <<= 8;
			base |= captain.get_spec::<u8>(SpecIdx::Byte7) as u16;
			base <<= 3;
			base |= ((captain.get_spec::<u8>(SpecIdx::Byte8) & 0b_1100_0000) >> 5) as u16;

			base
		};
		let new_stardust_on_acc = current_stardust_on_acc.saturating_add(stardust_amt as u16);
		// Write the new 'stardustOnAccount' value
		let spec_6 = ((new_stardust_on_acc >> 11) as u8) & 0b_0000_0011;
		captain.set_spec(
			SpecIdx::Byte6,
			(captain.get_spec::<u8>(SpecIdx::Byte6) & 0b_1111_1100) | spec_6,
		);
		let spec_7 = (new_stardust_on_acc >> 8) as u8;
		captain.set_spec(SpecIdx::Byte7, spec_7);
		let spec_8 = ((new_stardust_on_acc >> 3) as u8) & 0b_1110_0000;
		captain.set_spec(
			SpecIdx::Byte8,
			(captain.get_spec::<u8>(SpecIdx::Byte8) & 0b_0001_1111) | spec_8,
		);

		// Increase 'stardustGatheredAllTime'
		let current_stardust_all_time = {
			let mut base = 0_u16;

			base |= (captain.get_spec::<u8>(SpecIdx::Byte8) & 0b_0001_1111) as u16;
			base <<= 8;
			base |= captain.get_spec::<u8>(SpecIdx::Byte9) as u16;
			base <<= 1;
			base |= ((captain.get_spec::<u8>(SpecIdx::Byte8) & 0b_1000_0000) >> 7) as u16;

			base
		};
		let new_stardust_all_time = current_stardust_all_time.saturating_add(stardust_amt as u16);
		// Write the new 'stardustGatheredAllTime'
		let spec_8 = ((new_stardust_all_time >> 9) as u8) & 0b_0001_1111;
		captain.set_spec(
			SpecIdx::Byte8,
			(captain.get_spec::<u8>(SpecIdx::Byte8) & 0b_1110_0000) | spec_8,
		);
		let spec_9 = (new_stardust_all_time >> 8) as u8;
		captain.set_spec(SpecIdx::Byte9, spec_9);
		let spec_10 = ((new_stardust_all_time >> 1) as u8) & 0b_1000_0000;
		captain.set_spec(
			SpecIdx::Byte10,
			(captain.get_spec::<u8>(SpecIdx::Byte10) & 0b_0111_1111) | spec_10,
		);

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
			let stardust_amt = 300;
			let moon = create_random_unprospected_moon(&ALICE, stardust_amt);
			let captain = 1;
			let map = 2;

			/*let (leader_output, sacrifice_output) =
				AvatarCombinator::<Test>::extract_stardust(moon, captain, map)
					.expect("Should succeed in forging");

			assert_eq!(sacrifice_output.len(), 2);
			assert!(sacrifice_output.iter().all(|output| !is_forged(output)));
			assert_eq!(sacrifice_output.iter().filter(|output| is_minted(output)).count(), 1);

			if let LeaderForgeOutput::Forged((_, leader_avatar), _) = leader_output {
				assert_eq!(leader_avatar.souls, 16);
				assert_eq!(DnaUtils::read_attribute_raw(&leader_avatar, AvatarAttr::Quantity), 8);
			} else {
				panic!("LeaderForgeOutput should have been Forged!")
			}*/
		})
	}
}
