use super::*;
use sp_runtime::SaturatedConversion;
use sp_std::{
	cmp::Ordering,
	ops::{Div, Rem},
};

#[derive(Copy, Clone)]
pub enum AvatarAttr {
	ItemType,
	ItemSubType,
	ClassType1,
	ClassType2,
	CustomType1,
	CustomType2,
	RarityTier,
	Quantity,
}

#[derive(Copy, Clone)]
pub enum SpecIdx {
	Byte1,
	Byte2,
	Byte3,
	Byte4,
	Byte5,
	Byte6,
	Byte7,
	Byte8,
	#[allow(dead_code)]
	Byte9,
	#[allow(dead_code)]
	Byte10,
	#[allow(dead_code)]
	Byte11,
	#[allow(dead_code)]
	Byte12,
	#[allow(dead_code)]
	Byte13,
	#[allow(dead_code)]
	Byte14,
	#[allow(dead_code)]
	Byte15,
	#[allow(dead_code)]
	Byte16,
}

/// Struct to wrap DNA interactions with Avatars from V2 upwards.
/// Don't use with Avatars with V1.
pub(crate) struct DnaUtils<BlockNumber> {
	_marker: PhantomData<BlockNumber>,
}

impl<BlockNumber> DnaUtils<BlockNumber>
where
	BlockNumber: sp_runtime::traits::BlockNumber,
{
	fn read_strand(avatar: &Avatar<BlockNumber>, position: usize, byte_type: ByteType) -> u8 {
		Self::read_at(avatar.dna.as_slice(), position, byte_type)
	}

	fn write_strand(
		avatar: &mut Avatar<BlockNumber>,
		position: usize,
		byte_type: ByteType,
		value: u8,
	) {
		match byte_type {
			ByteType::Full => avatar.dna[position] = value,
			ByteType::High =>
				avatar.dna[position] =
					(avatar.dna[position] & (ByteType::High as u8)) | (value << 4),
			ByteType::Low =>
				avatar.dna[position] = (avatar.dna[position] & (ByteType::Low as u8)) |
					(value & (ByteType::High as u8)),
		}
	}

	fn read_at(dna: &[u8], position: usize, byte_type: ByteType) -> u8 {
		match byte_type {
			ByteType::Full => dna[position],
			ByteType::High => Self::high_nibble_of(dna[position]),
			ByteType::Low => Self::low_nibble_of(dna[position]),
		}
	}

	fn write_at(dna: &mut [u8], position: usize, byte_type: ByteType, value: u8) {
		match byte_type {
			ByteType::Full => dna[position] = value,
			ByteType::High =>
				dna[position] = (dna[position] & (ByteType::High as u8)) | (value << 4),
			ByteType::Low =>
				dna[position] =
					(dna[position] & (ByteType::Low as u8)) | (value & (ByteType::High as u8)),
		}
	}

	pub fn high_nibble_of(byte: u8) -> u8 {
		byte >> 4
	}

	pub fn low_nibble_of(byte: u8) -> u8 {
		byte & 0x0F
	}

	pub fn read_attribute<T>(avatar: &Avatar<BlockNumber>, attribute: AvatarAttr) -> T
	where
		T: ByteConvertible,
	{
		T::from_byte(Self::read_attribute_raw(avatar, attribute))
	}

	pub fn read_attribute_raw(avatar: &Avatar<BlockNumber>, attribute: AvatarAttr) -> u8 {
		match attribute {
			AvatarAttr::ItemType => Self::read_strand(avatar, 0, ByteType::High),
			AvatarAttr::ItemSubType => Self::read_strand(avatar, 0, ByteType::Low),
			AvatarAttr::ClassType1 => Self::read_strand(avatar, 1, ByteType::High),
			AvatarAttr::ClassType2 => Self::read_strand(avatar, 1, ByteType::Low),
			AvatarAttr::CustomType1 => Self::read_strand(avatar, 2, ByteType::High),
			AvatarAttr::CustomType2 => Self::read_strand(avatar, 4, ByteType::Full),
			AvatarAttr::RarityTier => Self::read_strand(avatar, 2, ByteType::Low),
			AvatarAttr::Quantity => Self::read_strand(avatar, 3, ByteType::Full),
		}
	}

	pub fn write_attribute<T>(avatar: &mut Avatar<BlockNumber>, attribute: AvatarAttr, value: &T)
	where
		T: ByteConvertible,
	{
		Self::write_attribute_raw(avatar, attribute, value.as_byte())
	}

	pub fn write_attribute_raw(avatar: &mut Avatar<BlockNumber>, attribute: AvatarAttr, value: u8) {
		match attribute {
			AvatarAttr::ItemType => Self::write_strand(avatar, 0, ByteType::High, value),
			AvatarAttr::ItemSubType => Self::write_strand(avatar, 0, ByteType::Low, value),
			AvatarAttr::ClassType1 => Self::write_strand(avatar, 1, ByteType::High, value),
			AvatarAttr::ClassType2 => Self::write_strand(avatar, 1, ByteType::Low, value),
			AvatarAttr::CustomType1 => Self::write_strand(avatar, 2, ByteType::High, value),
			AvatarAttr::CustomType2 => Self::write_strand(avatar, 4, ByteType::Full, value),
			AvatarAttr::RarityTier => Self::write_strand(avatar, 2, ByteType::Low, value),
			AvatarAttr::Quantity => Self::write_strand(avatar, 3, ByteType::Full, value),
		}
	}

	pub fn read_specs(avatar: &Avatar<BlockNumber>) -> [u8; 16] {
		let mut out = [0; 16];
		out.copy_from_slice(&avatar.dna[5..21]);
		out
	}

	pub fn read_spec_raw(avatar: &Avatar<BlockNumber>, index: SpecIdx) -> u8 {
		match index {
			SpecIdx::Byte1 => Self::read_strand(avatar, 5, ByteType::Full),
			SpecIdx::Byte2 => Self::read_strand(avatar, 6, ByteType::Full),
			SpecIdx::Byte3 => Self::read_strand(avatar, 7, ByteType::Full),
			SpecIdx::Byte4 => Self::read_strand(avatar, 8, ByteType::Full),
			SpecIdx::Byte5 => Self::read_strand(avatar, 9, ByteType::Full),
			SpecIdx::Byte6 => Self::read_strand(avatar, 10, ByteType::Full),
			SpecIdx::Byte7 => Self::read_strand(avatar, 11, ByteType::Full),
			SpecIdx::Byte8 => Self::read_strand(avatar, 12, ByteType::Full),
			SpecIdx::Byte9 => Self::read_strand(avatar, 13, ByteType::Full),
			SpecIdx::Byte10 => Self::read_strand(avatar, 14, ByteType::Full),
			SpecIdx::Byte11 => Self::read_strand(avatar, 15, ByteType::Full),
			SpecIdx::Byte12 => Self::read_strand(avatar, 16, ByteType::Full),
			SpecIdx::Byte13 => Self::read_strand(avatar, 17, ByteType::Full),
			SpecIdx::Byte14 => Self::read_strand(avatar, 18, ByteType::Full),
			SpecIdx::Byte15 => Self::read_strand(avatar, 19, ByteType::Full),
			SpecIdx::Byte16 => Self::read_strand(avatar, 20, ByteType::Full),
		}
	}

	pub fn read_spec<T>(avatar: &Avatar<BlockNumber>, spec_byte: SpecIdx) -> T
	where
		T: ByteConvertible,
	{
		T::from_byte(Self::read_spec_raw(avatar, spec_byte))
	}

	pub fn write_specs(avatar: &mut Avatar<BlockNumber>, value: [u8; 16]) {
		(avatar.dna[5..21]).copy_from_slice(&value);
	}

	pub fn write_spec(avatar: &mut Avatar<BlockNumber>, spec_byte: SpecIdx, value: u8) {
		match spec_byte {
			SpecIdx::Byte1 => Self::write_strand(avatar, 5, ByteType::Full, value),
			SpecIdx::Byte2 => Self::write_strand(avatar, 6, ByteType::Full, value),
			SpecIdx::Byte3 => Self::write_strand(avatar, 7, ByteType::Full, value),
			SpecIdx::Byte4 => Self::write_strand(avatar, 8, ByteType::Full, value),
			SpecIdx::Byte5 => Self::write_strand(avatar, 9, ByteType::Full, value),
			SpecIdx::Byte6 => Self::write_strand(avatar, 10, ByteType::Full, value),
			SpecIdx::Byte7 => Self::write_strand(avatar, 11, ByteType::Full, value),
			SpecIdx::Byte8 => Self::write_strand(avatar, 12, ByteType::Full, value),
			SpecIdx::Byte9 => Self::write_strand(avatar, 13, ByteType::Full, value),
			SpecIdx::Byte10 => Self::write_strand(avatar, 14, ByteType::Full, value),
			SpecIdx::Byte11 => Self::write_strand(avatar, 15, ByteType::Full, value),
			SpecIdx::Byte12 => Self::write_strand(avatar, 16, ByteType::Full, value),
			SpecIdx::Byte13 => Self::write_strand(avatar, 17, ByteType::Full, value),
			SpecIdx::Byte14 => Self::write_strand(avatar, 18, ByteType::Full, value),
			SpecIdx::Byte15 => Self::write_strand(avatar, 19, ByteType::Full, value),
			SpecIdx::Byte16 => Self::write_strand(avatar, 20, ByteType::Full, value),
		}
	}

	pub fn read_progress(avatar: &Avatar<BlockNumber>) -> [u8; 11] {
		let mut out = [0; 11];
		out.copy_from_slice(&avatar.dna[21..32]);
		out
	}

	pub fn read_progress_starting_at(avatar: &Avatar<BlockNumber>, index: usize) -> [u8; 11] {
		let mut out = [0; 11];
		let to_index = if avatar.dna.len() < 11 { avatar.dna.len() } else { 11 };

		for (i, p) in (index..(index + to_index)).enumerate() {
			out[i] = avatar.dna[p];
		}
		out
	}

	pub fn write_progress(avatar: &mut Avatar<BlockNumber>, value: [u8; 11]) {
		(avatar.dna[21..32]).copy_from_slice(&value);
	}

	pub fn is_progress_match(
		array_1: [u8; 11],
		array_2: [u8; 11],
		rarity_level: u8,
	) -> Option<Vec<u32>> {
		let (mirror, matches) = Self::match_progress(array_1, array_2, rarity_level);
		let match_count = matches.len() as u32;
		let mirror_count = mirror.len() as u32;

		(match_count > 0 && (((match_count * 2) + mirror_count) >= 6)).then_some(matches)
	}

	pub fn match_progress(
		array_1: [u8; 11],
		array_2: [u8; 11],
		rarity_level: u8,
	) -> (Vec<u32>, Vec<u32>) {
		let mut matches = Vec::<u32>::new();
		let mut mirrors = Vec::<u32>::new();

		let lowest_1 = Self::lowest_progress_byte(&array_1, ByteType::High);
		let lowest_2 = Self::lowest_progress_byte(&array_2, ByteType::High);

		if lowest_1 > lowest_2 {
			return (mirrors, matches)
		}

		for i in 0..array_1.len() {
			let rarity_1 = Self::read_at(&array_1, i, ByteType::High);
			let variation_1 = Self::read_at(&array_1, i, ByteType::Low);

			let rarity_2 = Self::read_at(&array_2, i, ByteType::High);
			let variation_2 = Self::read_at(&array_2, i, ByteType::Low);

			let have_same_rarity = rarity_1 == rarity_2 || rarity_2 == 0x0B;
			let is_maxed = rarity_1 > lowest_1;
			let byte_match = Self::match_progress_byte(variation_1, variation_2);

			if have_same_rarity &&
				!is_maxed && (rarity_1 < rarity_level || variation_2 == 0x0B || byte_match)
			{
				matches.push(i as u32);
			} else if is_maxed && ((variation_1 == variation_2) || variation_2 == 0x0B) {
				mirrors.push(i as u32);
			}
		}

		(mirrors, matches)
	}

	pub fn match_progress_byte(byte_1: u8, byte_2: u8) -> bool {
		let diff = if byte_1 >= byte_2 { byte_1 - byte_2 } else { byte_2 - byte_1 };
		diff == 1 || diff == (PROGRESS_VARIATIONS - 1)
	}

	pub fn create_pattern<T>(mut base_seed: usize, increase_seed: usize) -> Vec<T>
	where
		T: ByteConvertible + Ranged,
	{
		// Equivalent to "0X35AAB76B4482CADFF35BB3BD1C86648697B6F6833B47B939AECE95EDCD0347"
		let fixed_seed: [u8; 32] = [
			0x33, 0x35, 0xAA, 0xB7, 0x6B, 0x44, 0x82, 0xCA, 0xDF, 0xF3, 0x5B, 0xB3, 0xBD, 0x1C,
			0x86, 0x64, 0x86, 0x97, 0xB6, 0xF6, 0x83, 0x3B, 0x47, 0xB9, 0x39, 0xAE, 0xCE, 0x95,
			0xED, 0xCD, 0x03, 0x47,
		];

		let mut all_enum = T::range().map(|variant| variant as u8).collect::<Vec<_>>();
		let mut pattern = Vec::with_capacity(4);

		for _ in 0..4 {
			base_seed = base_seed.saturating_add(increase_seed);
			let rand_1 = fixed_seed[base_seed % 32];

			let enum_type = all_enum.remove(rand_1 as usize % all_enum.len());
			pattern.push(enum_type);
		}

		pattern.into_iter().map(|item| T::from_byte(item)).collect()
	}

	pub fn enums_to_bits<T>(enum_list: &[T]) -> u32
	where
		T: ByteConvertible + Ranged,
	{
		let range_mod = T::range().start as u8;
		enum_list
			.iter()
			.fold(0_u32, |acc, entry| acc | (1 << (entry.as_byte().saturating_sub(range_mod))))
	}

	pub fn enums_order_to_bits<T>(enum_list: &[T]) -> u32
	where
		T: Clone + Ord,
	{
		let mut sorted_list = Vec::with_capacity(enum_list.len());
		sorted_list.extend_from_slice(enum_list);
		sorted_list.sort();

		let mut byte_buff = 0;
		let fill_amount = usize::BITS - sorted_list.len().saturating_sub(1).leading_zeros();

		for entry in enum_list {
			if let Ok(index) = sorted_list.binary_search(entry) {
				byte_buff |= index as u32;
				byte_buff <<= fill_amount;
			}
		}

		byte_buff >> fill_amount
	}

	pub fn bits_to_enums<T>(bits: u32) -> Vec<T>
	where
		T: ByteConvertible + Ranged,
	{
		let mut enums = Vec::new();

		for (i, value) in T::range().enumerate() {
			if (bits & (1 << i)) != 0 {
				enums.push(T::from_byte(value as u8));
			}
		}

		enums
	}

	pub fn bits_order_to_enum<T>(bit_order: u32, step_count: usize, enum_list: Vec<T>) -> Vec<T>
	where
		T: Clone + Ord,
	{
		let mut sorted_enum_list = enum_list;
		sorted_enum_list.sort();

		let mut output_enums = Vec::new();

		let mask_width = step_count * 2;
		let bit_mask = 0b0000_0000_0000_0000_0000_0000_0000_0011 << mask_width.saturating_sub(2);

		for i in (0..mask_width).step_by(2) {
			let bit_segment = bit_order & (bit_mask >> i);
			let bit_position = (bit_segment >> (mask_width - (i + 2))) as usize;

			if sorted_enum_list.len() > bit_position {
				output_enums.push(sorted_enum_list[bit_position].clone());
			}
		}

		output_enums
	}

	pub fn generate_progress<T: Config>(
		rarity: &RarityTier,
		scale_factor: u32,
		probability: Option<u32>,
		hash_provider: &mut HashProvider<T, 32>,
	) -> [u8; 11] {
		let mut progress_bytes = [0; 11];

		let prob_value = probability.unwrap_or_default();

		for i in 0..progress_bytes.len() {
			let random_value = hash_provider.next();

			// Upcast random_value
			let new_rarity =
				if (random_value as u32).saturating_mul(scale_factor) < (prob_value * MAX_BYTE) {
					rarity.upgrade().as_byte()
				} else {
					rarity.as_byte()
				};

			Self::write_at(&mut progress_bytes, i, ByteType::High, new_rarity);
			Self::write_at(
				&mut progress_bytes,
				i,
				ByteType::Low,
				random_value % PROGRESS_VARIATIONS,
			);
		}

		Self::write_at(&mut progress_bytes, 10, ByteType::High, rarity.as_byte());

		progress_bytes
	}

	pub fn lowest_progress_byte(progress_bytes: &[u8; 11], byte_type: ByteType) -> u8 {
		let mut result = u8::MAX;

		for i in 0..progress_bytes.len() {
			let value = Self::read_at(progress_bytes, i, byte_type);
			if result > value {
				result = value;
			}
		}

		result
	}

	#[cfg(test)]
	pub fn lowest_progress_indexes(progress_bytes: &[u8; 11], byte_type: ByteType) -> Vec<usize> {
		let mut lowest = u8::MAX;

		let mut result = Vec::new();

		for i in 0..progress_bytes.len() {
			let value = Self::read_at(progress_bytes, i, byte_type);

			match lowest.cmp(&value) {
				Ordering::Greater => {
					lowest = value;
					result = Vec::new();
					result.push(i);
				},
				Ordering::Equal => result.push(i),
				_ => continue,
			}
		}

		result
	}

	pub fn indexes_of_max(byte_array: &[u8]) -> Vec<usize> {
		let mut max_value = 0;
		let mut max_indexes = Vec::new();

		for (i, byte) in byte_array.iter().enumerate() {
			match byte.cmp(&max_value) {
				Ordering::Greater => {
					max_value = *byte;
					max_indexes.clear();
					max_indexes.push(i);
				},
				Ordering::Equal => {
					max_indexes.push(i);
				},
				Ordering::Less => continue,
			}
		}

		max_indexes
	}

	pub fn current_period<T: Config>(
		current_phase: u32,
		total_phases: u32,
		block_number: BlockNumberFor<T>,
	) -> u32 {
		block_number
			.div(current_phase.into())
			.rem(total_phases.into())
			.saturated_into::<u32>()
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::mock::*;
	use hex;
	use std::ops::Range;

	#[derive(Copy, Clone, Debug, Default, Ord, PartialOrd, Eq, PartialEq)]
	enum Letters {
		#[default]
		A = 1,
		B = 2,
		C = 3,
		D = 4,
		E = 5,
		F = 6,
		G = 7,
		H = 8,
	}

	impl ByteConvertible for Letters {
		fn from_byte(byte: u8) -> Self {
			match byte {
				1 => Self::A,
				2 => Self::B,
				3 => Self::C,
				4 => Self::D,
				5 => Self::E,
				6 => Self::F,
				7 => Self::G,
				8 => Self::H,
				_ => Self::default(),
			}
		}

		fn as_byte(&self) -> u8 {
			*self as u8
		}
	}

	impl Ranged for Letters {
		fn range() -> Range<usize> {
			1..9
		}
	}

	#[test]
	fn test_bits_to_enums_consistency_1() {
		let bits = 0b_01_01_01_01;

		let result = DnaUtils::<BlockNumberFor<Test>>::bits_to_enums::<NibbleType>(bits);
		let expected = vec![NibbleType::X0, NibbleType::X2, NibbleType::X4, NibbleType::X6];

		assert_eq!(result, expected);
	}

	#[test]
	fn test_bits_to_enums_consistency_2() {
		let bits = 0b_11_01_10_01;

		let result = DnaUtils::<BlockNumberFor<Test>>::bits_to_enums::<Letters>(bits);
		let expected = vec![Letters::A, Letters::D, Letters::E, Letters::G, Letters::H];

		assert_eq!(result, expected);
	}

	#[test]
	fn test_bits_order_to_enums_consistency_1() {
		let bit_order = 0b_01_10_11_00;
		let enum_list = vec![NibbleType::X0, NibbleType::X2, NibbleType::X4, NibbleType::X6];

		let result = DnaUtils::<BlockNumberFor<Test>>::bits_order_to_enum(bit_order, 4, enum_list);
		let expected = vec![NibbleType::X2, NibbleType::X4, NibbleType::X6, NibbleType::X0];
		assert_eq!(result, expected);

		let bit_order_2 = 0b_01_11_00_10;
		let enum_list_2 = vec![NibbleType::X4, NibbleType::X5, NibbleType::X6, NibbleType::X7];

		let result_2 =
			DnaUtils::<BlockNumberFor<Test>>::bits_order_to_enum(bit_order_2, 4, enum_list_2);
		let expected_2 = vec![NibbleType::X5, NibbleType::X7, NibbleType::X4, NibbleType::X6];
		assert_eq!(result_2, expected_2);
	}

	#[test]
	fn test_bits_order_to_enums_consistency_2() {
		let bit_order = 0b_01_10_00_10;
		let enum_list = vec![Letters::A, Letters::C, Letters::E];

		let result = DnaUtils::<BlockNumberFor<Test>>::bits_order_to_enum(bit_order, 4, enum_list);
		let expected = vec![Letters::C, Letters::E, Letters::A, Letters::E];

		assert_eq!(result, expected);
	}

	#[test]
	fn test_enum_to_bits_consistency_1() {
		let pattern = vec![NibbleType::X2, NibbleType::X4, NibbleType::X1, NibbleType::X3];
		let expected = 0b_00_01_11_10;

		assert_eq!(DnaUtils::<BlockNumberFor<Test>>::enums_to_bits(&pattern), expected);
	}

	#[test]
	fn test_enum_to_bits_consistency_2() {
		let pattern = vec![Letters::A, Letters::E, Letters::D];
		let expected = 0b_00_01_10_01;

		assert_eq!(DnaUtils::<BlockNumberFor<Test>>::enums_to_bits(&pattern), expected);
	}

	#[test]
	fn test_enum_order_to_bits_consistency() {
		let pattern = vec![Letters::A, Letters::B, Letters::D, Letters::E, Letters::F];
		#[allow(clippy::unusual_byte_groupings)]
		// We group by 3 because the output is grouped by 3
		let expected = 0b_000_001_010_011_100;

		assert_eq!(DnaUtils::<BlockNumberFor<Test>>::enums_order_to_bits(&pattern), expected);
	}

	#[test]
	fn test_enum_to_bits_to_enum() {
		let pattern = vec![Letters::A, Letters::G, Letters::C];

		let expected = vec![Letters::A, Letters::C, Letters::G];

		let bits = DnaUtils::<BlockNumberFor<Test>>::enums_to_bits(&pattern);
		assert_eq!(bits, 0b_01_00_01_01);

		let enums = DnaUtils::<BlockNumberFor<Test>>::bits_to_enums::<Letters>(bits);
		assert_eq!(enums, expected);
	}

	#[test]
	fn test_create_pattern_consistency() {
		let base_seed = Letters::A.as_byte() as usize;
		let pattern = DnaUtils::<BlockNumberFor<Test>>::create_pattern::<NibbleType>(
			base_seed,
			Letters::D.as_byte() as usize,
		);

		let expected = vec![NibbleType::X4, NibbleType::X6, NibbleType::X5, NibbleType::X1];

		assert_eq!(pattern, expected);
	}

	#[test]
	fn tests_pattern_and_order() {
		let base_seed = (Letters::A.as_byte() + Letters::G.as_byte()) as usize;

		let pattern_1 = DnaUtils::<BlockNumberFor<Test>>::create_pattern::<NibbleType>(
			base_seed,
			Letters::E.as_byte() as usize,
		);
		let p10 = DnaUtils::<BlockNumberFor<Test>>::enums_to_bits(&pattern_1);
		let p11 = DnaUtils::<BlockNumberFor<Test>>::enums_order_to_bits(&pattern_1);

		assert_eq!(p10, 0b_10_01_10_01);
		assert_eq!(p11, 0b_10_00_11_01);

		// Decode Blueprint
		let unordered_1 = DnaUtils::<BlockNumberFor<Test>>::bits_to_enums::<NibbleType>(p10);
		let pattern_1_check =
			DnaUtils::<BlockNumberFor<Test>>::bits_order_to_enum(p11, 4, unordered_1);
		assert_eq!(pattern_1_check, pattern_1);

		// Pattern number and enum number only match if they are according to the index in the list
		let unordered_material = DnaUtils::<BlockNumberFor<Test>>::bits_to_enums::<Letters>(p10);
		assert_eq!(
			DnaUtils::<BlockNumberFor<Test>>::bits_order_to_enum(p11, 4, unordered_material)[0],
			Letters::E
		);

		let test_set: Vec<(Letters, u32, u32)> = vec![
			(Letters::A, 0b_11_00_10_01, 0b_01_00_11_10),
			(Letters::C, 0b_00_01_11_10, 0b_10_00_01_11),
			(Letters::D, 0b_10_10_10_10, 0b_10_00_11_01),
		];

		for (component, enum_to_bits, enum_order_to_bits) in test_set {
			let pattern_base = DnaUtils::<BlockNumberFor<Test>>::create_pattern::<NibbleType>(
				base_seed,
				component.as_byte() as usize,
			);
			let p_enum_to_bits = DnaUtils::<BlockNumberFor<Test>>::enums_to_bits(&pattern_base);
			let p_enum_order_to_bits =
				DnaUtils::<BlockNumberFor<Test>>::enums_order_to_bits(&pattern_base);

			assert_eq!(p_enum_to_bits, enum_to_bits);
			assert_eq!(p_enum_order_to_bits, enum_order_to_bits);
			// Decode Blueprint
			let unordered_base =
				DnaUtils::<BlockNumberFor<Test>>::bits_to_enums::<NibbleType>(p_enum_to_bits);
			let pattern_base_check = DnaUtils::<BlockNumberFor<Test>>::bits_order_to_enum(
				p_enum_order_to_bits,
				4,
				unordered_base,
			);
			assert_eq!(pattern_base_check, pattern_base);
		}
	}

	#[test]
	fn test_match_progress_array_consistency() {
		let empty_vec = Vec::<u32>::with_capacity(0);

		let arr_1 = [0x00; 11];
		let arr_2 = [0x00; 11];
		let (mirrors, matches) = DnaUtils::<BlockNumberFor<Test>>::match_progress(arr_1, arr_2, 0);
		assert_eq!(matches, empty_vec);
		assert_eq!(mirrors, empty_vec);

		let arr_1 = [0x10; 11];
		let arr_2 = [0x00; 11];
		let (mirrors, matches) = DnaUtils::<BlockNumberFor<Test>>::match_progress(arr_1, arr_2, 0);
		assert_eq!(matches, empty_vec);
		assert_eq!(mirrors, empty_vec);

		let arr_1 = [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x00];
		let arr_2 = [0x00; 11];
		let (mirrors, matches) = DnaUtils::<BlockNumberFor<Test>>::match_progress(arr_1, arr_2, 0);
		let expected_mirrors: Vec<u32> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
		assert_eq!(matches, empty_vec);
		assert_eq!(mirrors, expected_mirrors);

		let arr_1 = [0x00; 11];
		let arr_2 = [0x10; 11];
		let (mirrors, matches) = DnaUtils::<BlockNumberFor<Test>>::match_progress(arr_1, arr_2, 0);
		assert_eq!(matches, empty_vec);
		assert_eq!(mirrors, empty_vec);

		let arr_1 = [0x10; 11];
		let arr_2 = [0x10; 11];
		let (mirrors, matches) = DnaUtils::<BlockNumberFor<Test>>::match_progress(arr_1, arr_2, 0);
		assert_eq!(matches, empty_vec);
		assert_eq!(mirrors, empty_vec);

		let arr_1 = [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x00];
		let arr_2 = [0x10; 11];
		let (mirrors, matches) = DnaUtils::<BlockNumberFor<Test>>::match_progress(arr_1, arr_2, 0);
		let expected_mirrors: Vec<u32> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
		assert_eq!(matches, empty_vec);
		assert_eq!(mirrors, expected_mirrors);

		let arr_1 = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x04, 0x03, 0x02, 0x01, 0x00];
		let arr_2 = [0x01, 0x02, 0x03, 0x04, 0x05, 0x04, 0x03, 0x02, 0x01, 0x00, 0x05];
		let (mirrors, matches) = DnaUtils::<BlockNumberFor<Test>>::match_progress(arr_1, arr_2, 0);
		let expected_matches: Vec<u32> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
		assert_eq!(matches, expected_matches);
		assert_eq!(mirrors, empty_vec);

		let arr_1 = [0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x14, 0x13, 0x12, 0x11, 0x10];
		let arr_2 = [0x01, 0x02, 0x03, 0x04, 0x05, 0x04, 0x03, 0x02, 0x01, 0x00, 0x05];
		let (mirrors, matches) = DnaUtils::<BlockNumberFor<Test>>::match_progress(arr_1, arr_2, 0);
		assert_eq!(matches, empty_vec);
		assert_eq!(mirrors, empty_vec);

		let arr_1 = [0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x14, 0x13, 0x12, 0x11, 0x10];
		let arr_2 = [0x11, 0x12, 0x13, 0x14, 0x15, 0x14, 0x13, 0x12, 0x11, 0x10, 0x15];
		let (mirrors, matches) = DnaUtils::<BlockNumberFor<Test>>::match_progress(arr_1, arr_2, 0);
		let expected_matches: Vec<u32> = vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
		assert_eq!(matches, expected_matches);
		assert_eq!(mirrors, empty_vec);

		let arr_1 = [0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x14, 0x13, 0x12, 0x11, 0x00];
		let arr_2 = [0x11, 0x12, 0x13, 0x14, 0x15, 0x14, 0x13, 0x12, 0x11, 0x10, 0x15];
		let (mirrors, matches) = DnaUtils::<BlockNumberFor<Test>>::match_progress(arr_1, arr_2, 0);
		assert_eq!(matches, empty_vec);
		assert_eq!(mirrors, empty_vec);

		let arr_1 = [0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x04, 0x03, 0x02, 0x01, 0x00];
		let arr_2 = [0x11, 0x12, 0x13, 0x14, 0x15, 0x14, 0x13, 0x12, 0x11, 0x10, 0x15];
		let (mirrors, matches) = DnaUtils::<BlockNumberFor<Test>>::match_progress(arr_1, arr_2, 0);
		assert_eq!(matches, empty_vec);
		assert_eq!(mirrors, empty_vec);

		let arr_1 = [0x00, 0x11, 0x02, 0x13, 0x04, 0x15, 0x04, 0x13, 0x02, 0x11, 0x00];
		let arr_2 = [0x01, 0x01, 0x12, 0x13, 0x04, 0x04, 0x13, 0x12, 0x01, 0x01, 0x15];
		let (mirrors, matches) = DnaUtils::<BlockNumberFor<Test>>::match_progress(arr_1, arr_2, 0);
		let expected_matches: Vec<u32> = vec![0, 8];
		let expected_mirrors: Vec<u32> = vec![1, 3, 9];
		assert_eq!(matches, expected_matches);
		assert_eq!(mirrors, expected_mirrors);
	}

	#[test]
	fn test_match_progress_array_consistency_multiple() {
		let empty_vec = Vec::<u32>::with_capacity(0);

		let arr_1 = [0x14, 0x12, 0x10, 0x11, 0x20, 0x21, 0x10, 0x15, 0x11, 0x25, 0x13];
		let arr_2 = [0x12, 0x13, 0x14, 0x13, 0x14, 0x11, 0x22, 0x10, 0x14, 0x22, 0x11];
		let (mirrors, matches) = DnaUtils::<BlockNumberFor<Test>>::match_progress(arr_1, arr_2, 0);
		let expected_matches: Vec<u32> = vec![1, 7];
		let expected_mirrors: Vec<u32> = vec![5];
		assert_eq!(matches, expected_matches);
		assert_eq!(mirrors, expected_mirrors);

		let arr_1 = [0x14, 0x12, 0x10, 0x11, 0x20, 0x21, 0x10, 0x15, 0x11, 0x25, 0x13];
		let arr_2 = [0x10, 0x10, 0x10, 0x14, 0x14, 0x13, 0x13, 0x12, 0x15, 0x14, 0x14];
		let (mirrors, matches) = DnaUtils::<BlockNumberFor<Test>>::match_progress(arr_1, arr_2, 0);
		let expected_matches: Vec<u32> = vec![10];
		assert_eq!(matches, expected_matches);
		assert_eq!(mirrors, empty_vec);

		let arr_1 = [0x14, 0x12, 0x10, 0x11, 0x20, 0x21, 0x10, 0x15, 0x11, 0x25, 0x13];
		let arr_2 = [0x15, 0x10, 0x14, 0x13, 0x13, 0x11, 0x10, 0x14, 0x12, 0x20, 0x11];
		let (mirrors, matches) = DnaUtils::<BlockNumberFor<Test>>::match_progress(arr_1, arr_2, 0);
		let expected_matches: Vec<u32> = vec![0, 7, 8];
		let expected_mirrors: Vec<u32> = vec![5];
		assert_eq!(matches, expected_matches);
		assert_eq!(mirrors, expected_mirrors);

		let arr_1 = [0x14, 0x12, 0x10, 0x11, 0x20, 0x21, 0x10, 0x15, 0x11, 0x25, 0x13];
		let arr_2 = [0x11, 0x11, 0x11, 0x10, 0x15, 0x12, 0x11, 0x11, 0x13, 0x12, 0x14];
		let (mirrors, matches) = DnaUtils::<BlockNumberFor<Test>>::match_progress(arr_1, arr_2, 0);
		let expected_matches: Vec<u32> = vec![1, 2, 3, 6, 10];
		assert_eq!(matches, expected_matches);
		assert_eq!(mirrors, empty_vec);
	}

	#[test]
	fn test_match_progress_consistency_on_level() {
		let empty_vec = Vec::<u32>::with_capacity(0);

		let arr_1 = [0x42, 0x40, 0x40, 0x44, 0x43, 0x42, 0x41, 0x44, 0x44, 0x42, 0x45];
		let arr_2 = [0x41, 0x51, 0x52, 0x53, 0x44, 0x52, 0x45, 0x41, 0x40, 0x41, 0x43];
		let (mirrors, matches) = DnaUtils::<BlockNumberFor<Test>>::match_progress(arr_1, arr_2, 0);
		let expected_matches: Vec<u32> = vec![0, 4, 9];
		assert_eq!(matches, expected_matches);
		assert_eq!(mirrors, empty_vec);

		let arr_1 = [0x42, 0x40, 0x40, 0x44, 0x43, 0x42, 0x41, 0x44, 0x44, 0x42, 0x45];
		let arr_2 = [0x52, 0x41, 0x43, 0x41, 0x53, 0x45, 0x43, 0x44, 0x52, 0x43, 0x43];
		let (mirrors, matches) = DnaUtils::<BlockNumberFor<Test>>::match_progress(arr_1, arr_2, 0);
		let expected_matches: Vec<u32> = vec![1, 9];
		assert_eq!(matches, expected_matches);
		assert_eq!(mirrors, empty_vec);

		let arr_1 = [0x42, 0x40, 0x40, 0x44, 0x43, 0x42, 0x41, 0x54, 0x44, 0x42, 0x53];
		let arr_2 = [0x52, 0x40, 0x43, 0x41, 0x53, 0x45, 0x41, 0x44, 0x52, 0x43, 0x43];
		let (mirrors, matches) = DnaUtils::<BlockNumberFor<Test>>::match_progress(arr_1, arr_2, 0);
		let expected_matches: Vec<u32> = vec![9];
		let expected_mirrors: Vec<u32> = vec![7, 10];
		assert_eq!(matches, expected_matches);
		assert_eq!(mirrors, expected_mirrors);

		let arr_1 = [0x45, 0x45, 0x45, 0x45, 0x45, 0x45, 0x45, 0x30, 0x30, 0x30, 0x30];
		let arr_2 = [0x45, 0x45, 0x45, 0x45, 0x45, 0x35, 0x45, 0x31, 0x30, 0x45, 0x45];
		let (mirrors, matches) = DnaUtils::<BlockNumberFor<Test>>::match_progress(arr_1, arr_2, 0);
		let expected_matches: Vec<u32> = vec![7];
		let expected_mirrors: Vec<u32> = vec![0, 1, 2, 3, 4, 5, 6];
		assert_eq!(matches, expected_matches);
		assert_eq!(mirrors, expected_mirrors);

		let arr_1 = [0x31, 0x30, 0x35, 0x33, 0x30, 0x33, 0x31, 0x32, 0x32, 0x32, 0x34];
		let arr_2 = [0x21, 0x21, 0x35, 0x34, 0x24, 0x33, 0x23, 0x22, 0x22, 0x22, 0x22];
		let (mirrors, matches) = DnaUtils::<BlockNumberFor<Test>>::match_progress(arr_1, arr_2, 0);
		assert_eq!(matches, empty_vec);
		assert_eq!(mirrors, empty_vec);
	}

	#[test]
	fn test_match_progress_consistency_hex() {
		let empty_vec = Vec::<u32>::with_capacity(0);

		let arr_1: [u8; 11] =
			hex::decode("3130353330333132323234").expect("Decode").try_into().unwrap();
		let arr_2: [u8; 11] =
			hex::decode("2121353424332322222222").expect("Decode").try_into().unwrap();
		let (mirrors, matches) = DnaUtils::<BlockNumberFor<Test>>::match_progress(arr_1, arr_2, 0);
		assert_eq!(matches, empty_vec);
		assert_eq!(mirrors, empty_vec);
	}

	#[test]
	fn test_indexes_of_max() {
		ExtBuilder::default().build().execute_with(|| {
			assert_eq!(DnaUtils::<BlockNumberFor<Test>>::indexes_of_max(&[0, 2, 1, 1]), vec![1]);
			assert_eq!(
				DnaUtils::<BlockNumberFor<Test>>::indexes_of_max(&[9, 5, 3, 9, 7, 2, 1]),
				vec![0, 3]
			);
			assert_eq!(
				DnaUtils::<BlockNumberFor<Test>>::indexes_of_max(&[0, 0, 0, 0, 0]),
				vec![0, 1, 2, 3, 4]
			);
			assert_eq!(
				DnaUtils::<BlockNumberFor<Test>>::indexes_of_max(&[
					1, 4, 9, 2, 3, 11, 10, 11, 0, 1
				]),
				vec![5, 7]
			);
		});
	}

	#[test]
	fn test_current_period() {
		ExtBuilder::default().build().execute_with(|| {
			assert_eq!(DnaUtils::<BlockNumberFor<Test>>::current_period::<Test>(10, 14, 0), 0);
			assert_eq!(DnaUtils::<BlockNumberFor<Test>>::current_period::<Test>(10, 14, 9), 0);
			assert_eq!(DnaUtils::<BlockNumberFor<Test>>::current_period::<Test>(10, 14, 10), 1);
			assert_eq!(DnaUtils::<BlockNumberFor<Test>>::current_period::<Test>(10, 14, 19), 1);
			assert_eq!(DnaUtils::<BlockNumberFor<Test>>::current_period::<Test>(10, 14, 130), 13);
			assert_eq!(DnaUtils::<BlockNumberFor<Test>>::current_period::<Test>(10, 14, 139), 13);
			assert_eq!(DnaUtils::<BlockNumberFor<Test>>::current_period::<Test>(10, 14, 140), 0);
		});
	}
}

pub(crate) const PROGRESS_VARIATIONS: u8 = 6;
pub(crate) const MAX_BYTE: u32 = u8::MAX as u32;
