use crate::{
	types::avatar::versions::v2::{ByteType, DnaUtils},
	*,
};
use sp_runtime::{traits::Zero, DispatchError, Saturating};
use sp_std::{collections::btree_set::BTreeSet, marker::PhantomData, vec::Vec};

pub(crate) struct AttributeMapperV4;

impl<BlockNumber> AttributeMapper<BlockNumber> for AttributeMapperV4 {
	fn rarity(target: &Avatar<BlockNumber>) -> u8 {
		target.dna.iter().map(|x| *x >> 4).min().unwrap_or_default()
	}

	fn force(target: &Avatar<BlockNumber>) -> u8 {
		(target.dna.last().unwrap_or(&0) & 0b0000_1111).saturating_add(1)
	}
}

pub(crate) struct MinterV4<T: Config>(pub PhantomData<T>);

impl<T: Config> Minter<T> for MinterV4<T> {
	fn mint(
		player: &T::AccountId,
		season_id: &SeasonId,
		mint_option: &MintOption,
	) -> Result<Vec<AvatarIdOf<T>>, DispatchError> {
		let is_batched = mint_option.pack_size.is_batched();
		let season = Seasons::<T>::get(season_id).ok_or(Error::<T>::UnknownSeason)?;
		(0..mint_option.pack_size.as_mint_count())
			.map(|_| {
				let avatar_id = Pallet::<T>::random_hash(b"avatar_minter_v4", player);
				let dna = Self::random_dna(&avatar_id, &season, is_batched)?;
				let souls = (dna.iter().map(|x| *x as SoulCount).sum::<SoulCount>() % 100) + 1;
				let current_block = <frame_system::Pallet<T>>::block_number();
				let avatar = Avatar {
					season_id: *season_id,
					encoding: DnaEncoding::V4,
					dna,
					souls,
					minted_at: current_block,
				};
				Avatars::<T>::insert(avatar_id, (player, avatar));
				Owners::<T>::try_append(&player, &season_id, avatar_id)
					.map_err(|_| Error::<T>::MaxOwnershipReached)?;
				Ok(avatar_id)
			})
			.collect()
	}
}

impl<T: Config> MinterV4<T> {
	fn random_dna(
		hash: &T::Hash,
		season: &SeasonOf<T>,
		batched_mint: bool,
	) -> Result<Dna, DispatchError> {
		let dna = (0..season.max_components)
			.map(|i| {
				let (random_tier, random_variation) =
					Self::random_component(season, hash, i as usize * 2, batched_mint);
				((random_tier << 4) | random_variation) as u8
			})
			.collect::<Vec<_>>();
		Dna::try_from(dna).map_err(|_| Error::<T>::IncorrectDna.into())
	}

	fn random_component(
		season: &SeasonOf<T>,
		hash: &T::Hash,
		index: usize,
		batched_mint: bool,
	) -> (u8, u8) {
		let hash = hash.as_ref();
		let random_tier = {
			let random_prob = hash[index] % MAX_PERCENTAGE;
			let probs =
				if batched_mint { &season.batch_mint_probs } else { &season.single_mint_probs };
			let mut cumulative_sum = 0;
			let mut random_tier = &season.tiers[0];
			for i in 0..probs.len() {
				let new_cumulative_sum = cumulative_sum + probs[i];
				if random_prob >= cumulative_sum && random_prob < new_cumulative_sum {
					random_tier = &season.tiers[i];
					break
				}
				cumulative_sum = new_cumulative_sum;
			}
			random_tier
		};
		let random_variation = hash[index + 1] % season.max_variations;
		(random_tier.as_byte(), random_variation)
	}
}

pub(crate) struct ForgerV4<T: Config>(pub PhantomData<T>);

impl<T: Config> Forger<T> for ForgerV4<T> {
	fn forge(
		player: &T::AccountId,
		_season_id: SeasonId,
		season: &SeasonOf<T>,
		input_leader: ForgeItem<T>,
		input_sacrifices: Vec<ForgeItem<T>>,
		_restricted: bool,
	) -> Result<(LeaderForgeOutput<T>, Vec<ForgeOutput<T>>), DispatchError> {
		let (leader_id, mut leader) = input_leader;

		let leader_rarity = leader.rarity();
		let any_sacrifice_higher_rarity =
			input_sacrifices.iter().any(|(_, sacrifice)| sacrifice.rarity() > leader_rarity);

		// If any sacrifice has a higher rarity than the leader we forbid forging
		// All sacrifices should have the same or lower rarity than the leader
		if any_sacrifice_higher_rarity {
			return Ok((
				LeaderForgeOutput::Unchanged((leader_id, leader)),
				input_sacrifices
					.into_iter()
					.map(|sacrifice| ForgeOutput::Unchanged(sacrifice))
					.collect(),
			));
		}

		let current_block = <frame_system::Pallet<T>>::block_number();

		let (sacrifice_ids, sacrifice_avatars): (Vec<AvatarIdOf<T>>, Vec<AvatarOf<T>>) =
			input_sacrifices.into_iter().unzip();

		let (mut unique_matched_indexes, matches, soul_count) =
			Self::compare_all(&leader, sacrifice_avatars.as_slice(), 0)?;

		if matches > 0 {
			leader.souls += soul_count;
		}

		let mut upgraded_components = 0;
		let prob = Self::forge_probability(&leader, season, &current_block, matches);
		let rolls = sacrifice_avatars.len();
		let random_hash = Pallet::<T>::random_hash(b"avatar_forger_v4", player);

		for hash in random_hash.as_ref().iter().take(rolls) {
			let roll = hash % MAX_PERCENTAGE;
			if roll <= prob {
				if let Some(first_matched_index) = unique_matched_indexes.pop_first() {
					let nucleotide = leader.dna[first_matched_index];
					let current_tier_index = season
						.tiers
						.clone()
						.into_iter()
						.position(|tier| tier as u8 == nucleotide >> 4)
						.ok_or(Error::<T>::UnknownTier)?;

					let already_maxed_out = current_tier_index == (season.tiers.len() - 1);
					if !already_maxed_out {
						let next_tier = season.tiers[current_tier_index + 1].clone() as u8;
						let upgraded_nucleotide = (next_tier << 4) | (nucleotide & 0b0000_1111);
						leader.dna[first_matched_index] = upgraded_nucleotide;
						upgraded_components += 1;
					}
				}
			}
		}

		Ok((
			LeaderForgeOutput::Forged((leader_id, leader), upgraded_components),
			sacrifice_ids
				.into_iter()
				.map(|sacrifice_id| ForgeOutput::Consumed(sacrifice_id))
				.collect(),
		))
	}
}

impl<T: Config> ForgerV4<T> {
	fn compare_all(
		target: &AvatarOf<T>,
		others: &[AvatarOf<T>],
		max_tier: u8,
	) -> Result<(BTreeSet<usize>, u8, SoulCount), DispatchError> {
		let leader_tier = AttributeMapperV1::rarity(target);
		others.iter().try_fold(
			(BTreeSet::<usize>::new(), 0, SoulCount::zero()),
			|(mut matched_components, mut matches, mut souls), other| {
				let sacrifice_tier = AttributeMapperV1::rarity(other);
				if sacrifice_tier >= leader_tier {
					let (is_match, matching_components) = Self::compare(target, other, max_tier);

					if is_match {
						matches += 1;
						matched_components.extend(matching_components.iter());
					}
				}

				souls.saturating_accrue(other.souls);

				Ok((matched_components, matches, souls))
			},
		)
	}

	fn compare(target: &AvatarOf<T>, other: &AvatarOf<T>, max_tier: u8) -> (bool, BTreeSet<usize>) {
		let array_1 = DnaUtils::<BlockNumberFor<T>>::read_progress_starting_at(target, 0);
		let array_2 = DnaUtils::<BlockNumberFor<T>>::read_progress_starting_at(other, 0);

		let lowest_1 =
			DnaUtils::<BlockNumberFor<T>>::lowest_progress_byte(&array_1, ByteType::High);
		let lowest_2 =
			DnaUtils::<BlockNumberFor<T>>::lowest_progress_byte(&array_2, ByteType::High);

		if lowest_1 > lowest_2 {
			return (false, BTreeSet::new())
		}

		let (matching_indexes, match_count, mirror_count) =
			array_1.into_iter().zip(array_2).enumerate().fold(
				(BTreeSet::new(), 0, 0),
				|(mut matching_indexes, mut match_count, mut mirror_count), (i, (lhs, rhs))| {
					// Gene for avatar_1
					let rarity_1 = lhs >> 4;
					let variation_1 = lhs & 0b0000_1111;

					// Gene for avatar_2
					let rarity_2 = rhs >> 4;
					let variation_2 = rhs & 0b0000_1111;

					let have_same_rarity = rarity_1 == rarity_2;
					// check if current gene is lower than the lowest gen rarity or if gene already
					// has the highest rarity
					let is_maxed = rarity_1 > lowest_1;
					let byte_match = DnaUtils::<BlockNumberFor<T>>::match_progress_byte(
						variation_1,
						variation_2,
					);

					if have_same_rarity &&
						!is_maxed && (rarity_1 < max_tier || variation_2 == 0x0B || byte_match)
					{
						// if same rarity and not maxed, check if rarity is below max_tier or
						// there's a byte match
						match_count += 1;
						matching_indexes.insert(i);
					} else if is_maxed && ((variation_1 == variation_2) || variation_2 == 0x0B) {
						// if the genes are same rarity not on lowest rarity and same components,
						// then they count as mirrored
						mirror_count += 1;
					}

					(matching_indexes, match_count, mirror_count)
				},
			);

		let is_match = match_count > 0 && (((match_count * 2) + mirror_count) >= 6);
		(is_match, matching_indexes)
	}

	fn forge_probability(
		target: &AvatarOf<T>,
		season: &SeasonOf<T>,
		now: &BlockNumberFor<T>,
		matches: u8,
	) -> u8 {
		let period_multiplier = Self::forge_multiplier(target, season, now);
		// p = base_prob + (1 - base_prob) * (matches / max_sacrifices) * (1 / period_multiplier)
		season.base_prob +
			(((MAX_PERCENTAGE - season.base_prob) / season.max_sacrifices) * matches) /
				period_multiplier
	}

	fn forge_multiplier(target: &AvatarOf<T>, season: &SeasonOf<T>, now: &BlockNumberFor<T>) -> u8 {
		let current_period = season.current_period(now).saturating_add(1);
		let last_variation = AttributeMapperV1::force(target) as u16;
		let max_variations = season.max_variations as u16;
		let is_in_period = if last_variation == max_variations {
			(current_period % max_variations).is_zero()
		} else {
			(current_period % max_variations) == last_variation
		};

		if (current_period == last_variation) || is_in_period {
			1
		} else {
			2
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::mock::*;
	use frame_support::assert_ok;

	#[test]
	fn forge_probability_works() {
		// | variation |  period |
		// + --------- + ------- +
		// |         1 |   1,  7 |
		// |         2 |   2,  8 |
		// |         3 |   3,  9 |
		// |         4 |   4, 10 |
		// |         5 |   5, 11 |
		// |         6 |   6, 12 |
		let per_period = 2;
		let periods = 6;
		let max_variations = 6;
		let max_sacrifices = 4;

		let season = Season::default()
			.per_period(per_period)
			.periods(periods)
			.max_variations(max_variations)
			.max_sacrifices(max_sacrifices)
			.base_prob(0);

		let avatar = Avatar::default().dna(&[1, 3, 3, 7, 0]);

		// in period
		let now = 1;
		assert_eq!(ForgerV4::<Test>::forge_probability(&avatar, &season, &now, 1), 25);
		assert_eq!(ForgerV4::<Test>::forge_probability(&avatar, &season, &now, 2), 50);
		assert_eq!(ForgerV4::<Test>::forge_probability(&avatar, &season, &now, 3), 75);
		assert_eq!(ForgerV4::<Test>::forge_probability(&avatar, &season, &now, 4), 100);

		// not in period
		let now = 2;
		assert_eq!(ForgerV4::<Test>::forge_probability(&avatar, &season, &now, 1), 12);
		assert_eq!(ForgerV4::<Test>::forge_probability(&avatar, &season, &now, 2), 25);
		assert_eq!(ForgerV4::<Test>::forge_probability(&avatar, &season, &now, 3), 37);
		assert_eq!(ForgerV4::<Test>::forge_probability(&avatar, &season, &now, 4), 50);

		// increase base_prob to 10
		let season = season.base_prob(10);
		// in period
		let now = 1;
		assert_eq!(ForgerV4::<Test>::forge_probability(&avatar, &season, &now, 1), 32);
		assert_eq!(ForgerV4::<Test>::forge_probability(&avatar, &season, &now, 2), 54);
		assert_eq!(ForgerV4::<Test>::forge_probability(&avatar, &season, &now, 3), 76);
		assert_eq!(ForgerV4::<Test>::forge_probability(&avatar, &season, &now, 4), 98);

		// not in period
		let now = 2;
		assert_eq!(ForgerV4::<Test>::forge_probability(&avatar, &season, &now, 1), 21);
		assert_eq!(ForgerV4::<Test>::forge_probability(&avatar, &season, &now, 2), 32);
		assert_eq!(ForgerV4::<Test>::forge_probability(&avatar, &season, &now, 3), 43);
		assert_eq!(ForgerV4::<Test>::forge_probability(&avatar, &season, &now, 4), 54);
	}

	#[test]
	fn forge_multiplier_works() {
		// | variation |      period |
		// + --------- + ----------- +
		// |         1 | 1, 4, 7, 10 |
		// |         2 | 2, 5, 8, 11 |
		// |         3 | 3, 6, 9, 12 |
		let per_period = 4;
		let periods = 3;
		let max_variations = 3;

		let season = Season::default()
			.per_period(per_period)
			.periods(periods)
			.max_variations(max_variations);

		#[allow(clippy::erasing_op, clippy::identity_op)]
		for (range, dna, expected_period, expected_multiplier) in [
			// cycle 0, period 0, last_variation must be 0
			((0 * per_period)..((0 + 1) * per_period), [7, 3, 5, 7, 0], 0, 1),
			((0 * per_period)..((0 + 1) * per_period), [7, 3, 5, 7, 1], 0, 2),
			((0 * per_period)..((0 + 1) * per_period), [7, 3, 5, 7, 2], 0, 2),
			// cycle 0, period 1, last_variation must be 1
			((1 * per_period)..((1 + 1) * per_period), [7, 3, 5, 7, 0], 1, 2),
			((1 * per_period)..((1 + 1) * per_period), [7, 3, 5, 7, 1], 1, 1),
			((1 * per_period)..((1 + 1) * per_period), [7, 3, 5, 7, 2], 1, 2),
			// cycle 0, period 2, last_variation must be 2
			((2 * per_period)..((2 + 1) * per_period), [7, 3, 5, 7, 0], 2, 2),
			((2 * per_period)..((2 + 1) * per_period), [7, 3, 5, 7, 1], 2, 2),
			((2 * per_period)..((2 + 1) * per_period), [7, 3, 5, 7, 2], 2, 1),
			// cycle 1, period 0, last_variation must be 0
			((3 * per_period)..((3 + 1) * per_period), [7, 3, 5, 7, 0], 0, 1),
			((3 * per_period)..((3 + 1) * per_period), [7, 3, 5, 7, 1], 0, 2),
			((3 * per_period)..((3 + 1) * per_period), [7, 3, 5, 7, 2], 0, 2),
			// cycle 1, period 1, last_variation must be 1
			((4 * per_period)..((4 + 1) * per_period), [7, 3, 5, 7, 0], 1, 2),
			((4 * per_period)..((4 + 1) * per_period), [7, 3, 5, 7, 1], 1, 1),
			((4 * per_period)..((4 + 1) * per_period), [7, 3, 5, 7, 2], 1, 2),
			// cycle 1, period 2, last_variation must be 2
			((5 * per_period)..((5 + 1) * per_period), [7, 3, 5, 7, 0], 2, 2),
			((5 * per_period)..((5 + 1) * per_period), [7, 3, 5, 7, 1], 2, 2),
			((5 * per_period)..((5 + 1) * per_period), [7, 3, 5, 7, 2], 2, 1),
		] {
			for now in range {
				assert_eq!(season.current_period(&now), expected_period);

				let avatar = Avatar::default().dna(&dna);
				assert_eq!(
					ForgerV4::<Test>::forge_multiplier(&avatar, &season, &now),
					expected_multiplier
				);
			}
		}
	}

	#[test]
	fn compare_easy_works() {
		let season = Season::default();

		let leader = Avatar::default()
			.dna(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
		let other = Avatar::default()
			.dna(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

		assert_eq!(
			ForgerV4::<Test>::compare(&leader, &other, season.max_tier() as u8,),
			(true, BTreeSet::from([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]))
		);

		let leader = Avatar::default()
			.dna(&[0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10]);
		let other = Avatar::default()
			.dna(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

		assert_eq!(
			ForgerV4::<Test>::compare(&leader, &other, season.max_tier() as u8,),
			(false, BTreeSet::from([]))
		);

		let leader = Avatar::default()
			.dna(&[0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x00]);
		let other = Avatar::default()
			.dna(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

		assert_eq!(
			ForgerV4::<Test>::compare(&leader, &other, season.max_tier() as u8,),
			(true, BTreeSet::from([10]))
		);

		let leader = Avatar::default()
			.dna(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
		let other = Avatar::default()
			.dna(&[0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10]);

		assert_eq!(
			ForgerV4::<Test>::compare(&leader, &other, season.max_tier() as u8,),
			(false, BTreeSet::from([]))
		);

		let leader = Avatar::default()
			.dna(&[0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10]);
		let other = Avatar::default()
			.dna(&[0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10]);

		assert_eq!(
			ForgerV4::<Test>::compare(&leader, &other, season.max_tier() as u8,),
			(true, BTreeSet::from([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]))
		);

		let leader = Avatar::default()
			.dna(&[0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x00]);
		let other = Avatar::default()
			.dna(&[0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10]);

		assert_eq!(
			ForgerV4::<Test>::compare(&leader, &other, season.max_tier() as u8,),
			(false, BTreeSet::from([]))
		);

		let leader = Avatar::default()
			.dna(&[0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x04, 0x03, 0x02, 0x01, 0x00]);
		let other = Avatar::default()
			.dna(&[0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x03, 0x02, 0x01, 0x00, 0x05]);

		assert_eq!(
			ForgerV4::<Test>::compare(&leader, &other, season.max_tier() as u8,),
			(true, BTreeSet::from([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]))
		);
	}

	#[test]
	fn compare_sample_works() {
		let leader = Avatar::default()
			.dna(&[0x34, 0x34, 0x30, 0x30, 0x35, 0x31, 0x31, 0x34, 0x14, 0x35, 0x14]);

		let other = Avatar::default()
			.dna(&[0x11, 0x35, 0x30, 0x10, 0x14, 0x31, 0x33, 0x14, 0x32, 0x11, 0x15]);
		assert_eq!(ForgerV4::<Test>::compare(&leader, &other, 0,), (true, BTreeSet::from([10])));

		let other = Avatar::default()
			.dna(&[0x14, 0x15, 0x13, 0x10, 0x35, 0x15, 0x11, 0x32, 0x10, 0x30, 0x13]);
		assert_eq!(ForgerV4::<Test>::compare(&leader, &other, 0,), (true, BTreeSet::from([10])));

		let other = Avatar::default()
			.dna(&[0x11, 0x12, 0x13, 0x14, 0x15, 0x15, 0x11, 0x14, 0x13, 0x35, 0x15]);
		assert_eq!(ForgerV4::<Test>::compare(&leader, &other, 0,), (true, BTreeSet::from([8, 10])));

		let other = Avatar::default()
			.dna(&[0x11, 0x33, 0x12, 0x10, 0x15, 0x13, 0x11, 0x14, 0x15, 0x34, 0x13]);
		assert_eq!(ForgerV4::<Test>::compare(&leader, &other, 0,), (true, BTreeSet::from([8, 10])));
	}

	#[test]
	fn compare_sample_2_works() {
		let leader = Avatar::default()
			.dna(&[0x12, 0x12, 0x13, 0x14, 0x10, 0x15, 0x15, 0x13, 0x34, 0x30, 0x13]);

		let other = Avatar::default()
			.dna(&[0x11, 0x10, 0x15, 0x10, 0x15, 0x12, 0x12, 0x10, 0x11, 0x12, 0x13]);
		assert_eq!(ForgerV4::<Test>::compare(&leader, &other, 0,), (false, BTreeSet::from([0, 4])));

		let other = Avatar::default()
			.dna(&[0x10, 0x12, 0x10, 0x10, 0x13, 0x12, 0x12, 0x14, 0x15, 0x12, 0x13]);
		assert_eq!(ForgerV4::<Test>::compare(&leader, &other, 0,), (false, BTreeSet::from([7])));

		let other = Avatar::default()
			.dna(&[0x15, 0x12, 0x11, 0x15, 0x30, 0x13, 0x11, 0x13, 0x10, 0x11, 0x13]);
		assert_eq!(ForgerV4::<Test>::compare(&leader, &other, 0,), (false, BTreeSet::from([3])));

		let other = Avatar::default()
			.dna(&[0x10, 0x10, 0x10, 0x10, 0x35, 0x12, 0x31, 0x15, 0x14, 0x13, 0x11]);
		assert_eq!(ForgerV4::<Test>::compare(&leader, &other, 0,), (false, BTreeSet::from([])));
	}

	#[test]
	fn compare_simple_works() {
		let leader = Avatar::default()
			.dna(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
		let other = Avatar::default()
			.dna(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

		assert_eq!(ForgerV4::<Test>::compare(&leader, &other, 0,), (false, BTreeSet::from([])));

		let leader = Avatar::default()
			.dna(&[0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10]);
		let other = Avatar::default()
			.dna(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

		assert_eq!(ForgerV4::<Test>::compare(&leader, &other, 0,), (false, BTreeSet::from([])));

		let leader = Avatar::default()
			.dna(&[0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x00]);
		let other = Avatar::default()
			.dna(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

		assert_eq!(ForgerV4::<Test>::compare(&leader, &other, 0,), (false, BTreeSet::from([])));

		let leader = Avatar::default()
			.dna(&[0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);
		let other = Avatar::default()
			.dna(&[0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10]);

		assert_eq!(ForgerV4::<Test>::compare(&leader, &other, 0,), (false, BTreeSet::from([])));

		let leader = Avatar::default()
			.dna(&[0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10]);
		let other = Avatar::default()
			.dna(&[0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10]);

		assert_eq!(ForgerV4::<Test>::compare(&leader, &other, 0,), (false, BTreeSet::from([])));

		let leader = Avatar::default()
			.dna(&[0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x00]);
		let other = Avatar::default()
			.dna(&[0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10]);

		assert_eq!(ForgerV4::<Test>::compare(&leader, &other, 0,), (false, BTreeSet::from([])));

		let leader = Avatar::default()
			.dna(&[0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x04, 0x03, 0x02, 0x01, 0x00]);
		let other = Avatar::default()
			.dna(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x04, 0x03, 0x02, 0x01, 0x00, 0x05]);

		assert_eq!(
			ForgerV4::<Test>::compare(&leader, &other, 0,),
			(true, BTreeSet::from([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]))
		);

		let leader = Avatar::default()
			.dna(&[0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x14, 0x13, 0x12, 0x11, 0x10]);
		let other = Avatar::default()
			.dna(&[0x01, 0x02, 0x03, 0x04, 0x05, 0x04, 0x03, 0x02, 0x01, 0x00, 0x05]);

		assert_eq!(ForgerV4::<Test>::compare(&leader, &other, 0,), (false, BTreeSet::from([])));

		let leader = Avatar::default()
			.dna(&[0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x14, 0x13, 0x12, 0x11, 0x10]);
		let other = Avatar::default()
			.dna(&[0x11, 0x12, 0x13, 0x14, 0x15, 0x14, 0x13, 0x12, 0x11, 0x10, 0x15]);

		assert_eq!(
			ForgerV4::<Test>::compare(&leader, &other, 0,),
			(true, BTreeSet::from([0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10]))
		);

		let leader = Avatar::default()
			.dna(&[0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x14, 0x13, 0x12, 0x11, 0x00]);
		let other = Avatar::default()
			.dna(&[0x11, 0x12, 0x13, 0x14, 0x15, 0x14, 0x13, 0x12, 0x11, 0x10, 0x15]);

		assert_eq!(ForgerV4::<Test>::compare(&leader, &other, 0,), (false, BTreeSet::from([])));

		let leader = Avatar::default()
			.dna(&[0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x04, 0x03, 0x02, 0x01, 0x00]);
		let other = Avatar::default()
			.dna(&[0x11, 0x12, 0x13, 0x14, 0x15, 0x14, 0x13, 0x12, 0x11, 0x10, 0x15]);

		assert_eq!(ForgerV4::<Test>::compare(&leader, &other, 0,), (false, BTreeSet::from([])));

		let leader = Avatar::default()
			.dna(&[0x00, 0x11, 0x02, 0x13, 0x04, 0x15, 0x04, 0x13, 0x02, 0x11, 0x00]);
		let other = Avatar::default()
			.dna(&[0x01, 0x01, 0x12, 0x13, 0x04, 0x04, 0x13, 0x12, 0x01, 0x01, 0x15]);

		assert_eq!(ForgerV4::<Test>::compare(&leader, &other, 0,), (true, BTreeSet::from([0, 8])));
	}

	#[test]
	fn forge_should_work_for_matches() {
		let tiers = &[RarityTier::Common, RarityTier::Legendary];
		let season_id = 1;
		let season = Season::default()
			.tiers(tiers)
			.batch_mint_probs(&[100])
			.max_components(5)
			.max_variations(3)
			.min_sacrifices(1)
			.max_sacrifices(2);
		let season_schedule = SeasonSchedule::default();

		ExtBuilder::default()
			.seasons(&[(season_id, season.clone())])
			.schedules(&[(season_id, season_schedule.clone())])
			.mint_cooldown(1)
			.free_mints(&[(BOB, 10)])
			.build()
			.execute_with(|| {
				// prepare avatars to forge
				run_to_block(season_schedule.start);
				assert_ok!(AAvatars::mint(
					RuntimeOrigin::signed(BOB),
					MintOption {
						pack_size: MintPackSize::Six,
						payment: MintPayment::Free,
						pack_type: PackType::default(),
					}
				));

				// forge
				let owned_avatar_ids = Owners::<Test>::get(BOB, season_id);
				let leader_id = owned_avatar_ids[0];
				let sacrifice_ids = &owned_avatar_ids[1..3];

				let original_leader: AvatarOf<Test> = Avatars::<Test>::get(leader_id).unwrap().1;
				let original_sacrifices = sacrifice_ids
					.iter()
					.map(|id| Avatars::<Test>::get(id).unwrap().1)
					.collect::<Vec<_>>();

				assert_ok!(AAvatars::forge(
					RuntimeOrigin::signed(BOB),
					leader_id,
					sacrifice_ids.to_vec()
				));
				let forged_leader = Avatars::<Test>::get(leader_id).unwrap().1;

				for (sacrifice, result) in original_sacrifices
					.iter()
					.zip([(false, BTreeSet::from([])), (false, BTreeSet::from([]))])
				{
					assert_eq!(ForgerV4::<Test>::compare(&original_leader, sacrifice, 0,), result)
				}

				// check all sacrifices are burned
				for sacrifice_id in sacrifice_ids {
					assert!(!Avatars::<Test>::contains_key(sacrifice_id));
				}
				// check for souls accumulation
				assert_eq!(
					forged_leader.souls,
					original_leader.souls +
						original_sacrifices.iter().map(|x| x.souls).sum::<SoulCount>(),
				);

				// check for the upgraded DNA
				assert_ne!(original_leader.dna[0..=1], forged_leader.dna[0..=1]);
				assert_eq!(original_leader.dna.to_vec()[0] >> 4, RarityTier::Common as u8);
				assert_eq!(original_leader.dna.to_vec()[1] >> 4, RarityTier::Common as u8);
				assert_eq!(forged_leader.dna.to_vec()[0] >> 4, RarityTier::Legendary as u8);
				assert_eq!(forged_leader.dna.to_vec()[1] >> 4, RarityTier::Common as u8);
				System::assert_last_event(mock::RuntimeEvent::AAvatars(
					crate::Event::AvatarsForged { avatar_ids: vec![(leader_id, 1)] },
				));

				// variations remain the same
				assert_eq!(
					original_leader.dna[0..=1].iter().map(|x| x & 0b0000_1111).collect::<Vec<_>>(),
					forged_leader.dna[0..=1].iter().map(|x| x & 0b0000_1111).collect::<Vec<_>>(),
				);
				// other components remain the same
				assert_eq!(
					original_leader.dna[2..season.max_components as usize],
					forged_leader.dna[2..season.max_components as usize]
				);
			});
	}

	#[test]
	fn forge_should_work_for_non_matches() {
		let tiers =
			&[RarityTier::Common, RarityTier::Uncommon, RarityTier::Rare, RarityTier::Legendary];
		let season_id = 1;
		let season = Season::default()
			.tiers(tiers)
			.batch_mint_probs(&[33, 33, 34])
			.max_components(10)
			.max_variations(12)
			.min_sacrifices(1)
			.max_sacrifices(5);
		let season_schedule = SeasonSchedule::default();

		ExtBuilder::default()
			.seasons(&[(season_id, season.clone())])
			.schedules(&[(season_id, season_schedule.clone())])
			.mint_cooldown(1)
			.free_mints(&[(BOB, 10)])
			.build()
			.execute_with(|| {
				// prepare avatars to forge
				run_to_block(season_schedule.start);
				assert_ok!(AAvatars::mint(
					RuntimeOrigin::signed(BOB),
					MintOption {
						pack_size: MintPackSize::Six,
						payment: MintPayment::Free,
						pack_type: PackType::default(),
					}
				));

				// forge
				let owned_avatar_ids = Owners::<Test>::get(BOB, season_id);
				let leader_id = owned_avatar_ids[0];
				let sacrifice_id = owned_avatar_ids[1];

				let original_leader: AvatarOf<Test> = Avatars::<Test>::get(leader_id).unwrap().1;
				let original_sacrifice = Avatars::<Test>::get(sacrifice_id).unwrap().1;

				assert_ok!(AAvatars::forge(
					RuntimeOrigin::signed(BOB),
					leader_id,
					vec![sacrifice_id]
				));
				let forged_leader = Avatars::<Test>::get(leader_id).unwrap().1;

				assert_eq!(
					ForgerV4::<Test>::compare(&original_leader, &original_sacrifice, 0,),
					(false, BTreeSet::from([]))
				);
				// check all sacrifices are burned
				assert!(!Avatars::<Test>::contains_key(sacrifice_id));
				// check for souls accumulation
				assert_eq!(forged_leader.souls, original_leader.souls + original_sacrifice.souls);

				// check DNAs are the same
				assert_eq!(original_leader.dna, forged_leader.dna);
				System::assert_last_event(mock::RuntimeEvent::AAvatars(
					crate::Event::AvatarsForged { avatar_ids: vec![(leader_id, 0)] },
				));
			});
	}

	fn create_avatar_from_rarity(
		owner: MockAccountId,
		season_id: SeasonId,
		rarity_tier: &RarityTier,
		with_souls: SoulCount,
	) -> (AvatarIdOf<Test>, AvatarOf<Test>) {
		let dna = [rarity_tier.as_byte() << 4; 8];
		let mut avatar = Avatar::default().season_id(season_id).dna(&dna);
		avatar.souls = with_souls;

		let avatar_id = H256::random();
		Avatars::<Test>::insert(avatar_id, (owner, avatar.clone()));
		Owners::<Test>::try_append(owner, season_id, avatar_id).unwrap();

		(avatar_id, avatar)
	}

	#[test]
	fn forge_should_not_allow_higher_tiers_into_lower_tier() {
		let tiers = &[
			RarityTier::Common,
			RarityTier::Uncommon,
			RarityTier::Rare,
			RarityTier::Epic,
			RarityTier::Legendary,
		];
		let season_id = 1;
		let season = Season::default()
			.tiers(tiers)
			.batch_mint_probs(&[33, 33, 34])
			.max_components(10)
			.max_variations(12)
			.min_sacrifices(1)
			.max_sacrifices(5);

		ExtBuilder::default().build().execute_with(|| {
			for leader_rarity in tiers {
				let (leader_id, leader) =
					create_avatar_from_rarity(CHARLIE, season_id, leader_rarity, 0);
				assert_eq!(leader.rarity(), leader_rarity.as_byte());
				for s_rarity_1 in tiers {
					let (s_id_1, sac_1) =
						create_avatar_from_rarity(CHARLIE, season_id, s_rarity_1, 0);
					assert_eq!(sac_1.rarity(), s_rarity_1.as_byte());
					for s_rarity_2 in tiers {
						let (s_id_2, sac_2) =
							create_avatar_from_rarity(CHARLIE, season_id, s_rarity_2, 0);
						assert_eq!(sac_2.rarity(), s_rarity_2.as_byte());

						let (leader_output, sacrifice_output) = ForgerV4::<Test>::forge(
							&CHARLIE,
							season_id,
							&season,
							(leader_id, leader.clone()),
							vec![(s_id_1, sac_1.clone()), (s_id_2, sac_2.clone())],
							false,
						)
						.expect("Forge should succeed");

						// If any of the sacrifices has greater rarity than the leader the forge
						// goes through without changing any of the inputs.
						if s_rarity_1 > leader_rarity || s_rarity_2 > leader_rarity {
							assert!(matches!(leader_output, LeaderForgeOutput::Unchanged(_)));
							assert!(matches!(sacrifice_output[0], ForgeOutput::Unchanged(_)));
							assert!(matches!(sacrifice_output[1], ForgeOutput::Unchanged(_)));
						} else {
							assert!(matches!(leader_output, LeaderForgeOutput::Forged(_, _)));
							assert!(matches!(sacrifice_output[0], ForgeOutput::Consumed(_)));
							assert!(matches!(sacrifice_output[1], ForgeOutput::Consumed(_)));
						}
					}
				}
			}
		});
	}

	#[test]
	fn forge_should_only_add_sp_if_there_is_matching_sacrifices() {
		let season_id = 1;
		let season = Season::default()
			.batch_mint_probs(&[33, 33, 34])
			.max_components(10)
			.max_variations(12)
			.min_sacrifices(1)
			.max_sacrifices(5);

		ExtBuilder::default().build().execute_with(|| {
			let leader_souls = 150;
			let (leader_id, mut leader) =
				create_avatar_from_rarity(CHARLIE, season_id, &RarityTier::Rare, leader_souls);
			leader.dna = Dna::try_from(vec![
				0x12, 0x12, 0x13, 0x14, 0x10, 0x15, 0x15, 0x13, 0x34, 0x30, 0x13,
			])
			.expect("Valid Dna");
			let matching_souls = 50;
			let (matching_id, mut matching_avatar) =
				create_avatar_from_rarity(CHARLIE, season_id, &RarityTier::Common, matching_souls);
			matching_avatar.dna = Dna::try_from(vec![
				0x11, 0x11, 0x13, 0x14, 0x10, 0x15, 0x15, 0x12, 0x33, 0x30, 0x12,
			])
			.expect("Valid Dna");
			let non_matching_souls = 200;
			let (non_matching_id, mut non_matching_avatar) = create_avatar_from_rarity(
				CHARLIE,
				season_id,
				&RarityTier::Common,
				non_matching_souls,
			);
			non_matching_avatar.dna = Dna::try_from(vec![
				0x12, 0x12, 0x13, 0x14, 0x10, 0x15, 0x15, 0x13, 0x34, 0x30, 0x13,
			])
			.expect("Valid Dna");

			// Forging with a matching valid sacrifice should add its SP to the leader
			let (_, matches, _) =
				ForgerV4::<Test>::compare_all(&leader, &[matching_avatar.clone()], 0)
					.expect("Compare should succeed");
			assert_eq!(matches, 1);

			let (leader_output, sacrifice_output) = ForgerV4::<Test>::forge(
				&CHARLIE,
				season_id,
				&season,
				(leader_id, leader.clone()),
				vec![(matching_id, matching_avatar.clone())],
				false,
			)
			.expect("Forge should succeed");
			assert!(matches!(sacrifice_output[0], ForgeOutput::Consumed(_)));
			match leader_output {
				LeaderForgeOutput::Forged((_, forged_leader), _) =>
					assert_eq!(forged_leader.souls, leader_souls + matching_souls),
				_ => panic!("Forge output should be Forged"),
			}

			// Forging with a non-matching valid sacrifice should not add its SP to the leader
			let (_, matches, _) =
				ForgerV4::<Test>::compare_all(&leader, &[non_matching_avatar.clone()], 0)
					.expect("Compare should succeed");
			assert_eq!(matches, 0);

			let (leader_output, sacrifice_output) = ForgerV4::<Test>::forge(
				&CHARLIE,
				season_id,
				&season,
				(leader_id, leader.clone()),
				vec![(non_matching_id, non_matching_avatar.clone())],
				false,
			)
			.expect("Forge should succeed");
			assert!(matches!(sacrifice_output[0], ForgeOutput::Consumed(_)));
			match leader_output {
				LeaderForgeOutput::Forged((_, forged_leader), _) =>
					assert_eq!(forged_leader.souls, leader_souls),
				_ => panic!("Forge output should be Forged"),
			}

			// Forging with a both a non-matching and a matching sacrifice should all sacrifice SP
			// to the leader
			let (leader_output, sacrifice_output) = ForgerV4::<Test>::forge(
				&CHARLIE,
				season_id,
				&season,
				(leader_id, leader.clone()),
				vec![(non_matching_id, non_matching_avatar), (matching_id, matching_avatar)],
				false,
			)
			.expect("Forge should succeed");
			assert!(matches!(sacrifice_output[0], ForgeOutput::Consumed(_)));
			assert!(matches!(sacrifice_output[1], ForgeOutput::Consumed(_)));
			match leader_output {
				LeaderForgeOutput::Forged((_, forged_leader), _) => assert_eq!(
					forged_leader.souls,
					leader_souls + matching_souls + non_matching_souls
				),
				_ => panic!("Forge output should be Forged"),
			}
		});
	}

	#[test]
	fn forging_avatars_doesnt_go_above_max_rarity_in_season() {
		let tiers = &[RarityTier::Common, RarityTier::Rare, RarityTier::Legendary];
		let season_id = 1;
		let season = Season::default().tiers(tiers);
		let season_schedule = SeasonSchedule::default();

		ExtBuilder::default()
			.seasons(&[(season_id, season.clone())])
			.schedules(&[(season_id, season_schedule.clone())])
			.mint_cooldown(1)
			.free_mints(&[(BOB, 10)])
			.build()
			.execute_with(|| {
				let legendary_1_dna = Dna::try_from(vec![
					0x52, 0x52, 0x53, 0x54, 0x50, 0x55, 0x55, 0x53, 0x54, 0x50, 0x53,
				])
				.expect("Valid Dna");
				let (legendary_id_1, mut legendary_1) =
					create_avatar_from_rarity(BOB, season_id, &RarityTier::Legendary, 100);
				legendary_1.dna = legendary_1_dna.clone();
				assert_eq!(legendary_1.rarity(), RarityTier::Legendary.as_byte());
				let (legendary_id_2, mut legendary_2) =
					create_avatar_from_rarity(BOB, season_id, &RarityTier::Legendary, 100);
				legendary_2.dna = Dna::try_from(vec![
					0x53, 0x52, 0x54, 0x53, 0x51, 0x55, 0x55, 0x54, 0x54, 0x51, 0x53,
				])
				.expect("Valid Dna");
				assert_eq!(legendary_2.rarity(), RarityTier::Legendary.as_byte());
				let (legendary_id_3, mut legendary_3) =
					create_avatar_from_rarity(BOB, season_id, &RarityTier::Legendary, 100);
				legendary_3.dna = Dna::try_from(vec![
					0x53, 0x52, 0x54, 0x53, 0x51, 0x55, 0x55, 0x54, 0x54, 0x51, 0x53,
				])
				.expect("Valid Dna");
				assert_eq!(legendary_3.rarity(), RarityTier::Legendary.as_byte());
				let (legendary_id_4, mut legendary_4) =
					create_avatar_from_rarity(BOB, season_id, &RarityTier::Legendary, 100);
				legendary_4.dna = Dna::try_from(vec![
					0x53, 0x52, 0x54, 0x53, 0x51, 0x55, 0x55, 0x54, 0x54, 0x51, 0x53,
				])
				.expect("Valid Dna");
				assert_eq!(legendary_4.rarity(), RarityTier::Legendary.as_byte());
				let (legendary_id_5, mut legendary_5) =
					create_avatar_from_rarity(BOB, season_id, &RarityTier::Legendary, 100);
				legendary_5.dna = Dna::try_from(vec![
					0x53, 0x52, 0x54, 0x53, 0x51, 0x55, 0x55, 0x54, 0x54, 0x51, 0x53,
				])
				.expect("Valid Dna");
				assert_eq!(legendary_5.rarity(), RarityTier::Legendary.as_byte());

				// Forging with a matching valid sacrifice should add its SP to the leader
				let (_, matches, _) = ForgerV4::<Test>::compare_all(
					&legendary_1,
					&[
						legendary_2.clone(),
						legendary_3.clone(),
						legendary_4.clone(),
						legendary_5.clone(),
					],
					0,
				)
				.expect("Compare should succeed");
				assert_eq!(matches, 4);

				let (leader_output, sacrifice_output) = ForgerV4::<Test>::forge(
					&CHARLIE,
					season_id,
					&season,
					(legendary_id_1, legendary_1),
					vec![
						(legendary_id_2, legendary_2),
						(legendary_id_3, legendary_3),
						(legendary_id_4, legendary_4),
						(legendary_id_5, legendary_5),
					],
					false,
				)
				.expect("Forge should succeed");
				assert!(matches!(sacrifice_output[0], ForgeOutput::Consumed(_)));
				assert!(matches!(sacrifice_output[1], ForgeOutput::Consumed(_)));
				assert!(matches!(sacrifice_output[2], ForgeOutput::Consumed(_)));
				assert!(matches!(sacrifice_output[3], ForgeOutput::Consumed(_)));
				match leader_output {
					LeaderForgeOutput::Forged((_, forged_leader), _) => {
						assert_eq!(forged_leader.rarity(), RarityTier::Legendary.as_byte());
						assert_eq!(forged_leader.dna.to_vec(), legendary_1_dna.to_vec());
					},

					_ => panic!("Forge output should be Forged"),
				}
			});
	}
}
