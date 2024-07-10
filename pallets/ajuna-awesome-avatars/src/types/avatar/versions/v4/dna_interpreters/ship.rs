use super::*;

use core::{
	cmp::min,
	ops::{Deref, DerefMut},
};

pub(crate) struct ShipInterpreter<'a, BlockNumber> {
	inner: &'a mut WrappedAvatar<BlockNumber>,
}

impl<'a, BlockNumber> DnaInterpreter<'a, BlockNumber, ShipInterpreter<'a, BlockNumber>>
	for ShipInterpreter<'a, BlockNumber>
{
	fn from_wrapper(wrap: &'a mut WrappedAvatar<BlockNumber>) -> ShipInterpreter<'a, BlockNumber> {
		Self { inner: wrap }
	}
}

impl<'a, BlockNumber> Deref for ShipInterpreter<'a, BlockNumber> {
	type Target = WrappedAvatar<BlockNumber>;

	fn deref(&self) -> &Self::Target {
		self.inner
	}
}

impl<'a, BlockNumber> DerefMut for ShipInterpreter<'a, BlockNumber> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.inner
	}
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) enum UpgradeType {
	Scanner = 0,
	Cargo = 1,
	Speed = 2,
}

impl<'a, BlockNumber> ShipInterpreter<'a, BlockNumber>
where
	BlockNumber: sp_runtime::traits::BlockNumber,
{
	/// resourceType_(1...8) --> [(5, 0, 6)], [(6, 5, 6)], ..., [(16, 3, 6)]
	pub fn get_resource_type(&self, resource_idx: u8) -> u8 {
		let resource_idx = min(resource_idx, 7) * 13;
		let (byte_idx, bit_idx) = InterpreterUtils::get_indices(resource_idx, 5);

		if bit_idx > 2 {
			let upper_bits = 8_u8.saturating_sub(bit_idx);
			let lower_bits = 6_u8.saturating_sub(upper_bits);
			self.inner
				.get_segmented_attribute_of_two(byte_idx as usize, &[upper_bits, lower_bits]) as u8
		} else {
			self.inner.get_segment_attribute_of_one(byte_idx as usize, bit_idx, 6)
		}
	}

	/// resourceType_(1...8) --> [(5, 0, 6)], [(6, 5, 6)], ..., [(16, 3, 6)]
	pub fn set_resource_type(&mut self, resource_idx: u8, value: u8) {
		let resource_idx = min(resource_idx, 7) * 13;
		let (byte_idx, bit_idx) = InterpreterUtils::get_indices(resource_idx, 5);

		// Only 6 bits max for resourceType_(1...8)
		let value = min(value, 0b0011_1111);

		if bit_idx > 2 {
			let upper_bits = 8_u8.saturating_sub(bit_idx);
			let lower_bits = 6_u8.saturating_sub(upper_bits);
			self.inner.set_segmented_attribute_of_two(
				byte_idx as usize,
				&[upper_bits, lower_bits],
				value as u16,
			)
		} else {
			self.inner.set_segment_attribute_of_one(byte_idx as usize, bit_idx, 6, value)
		}
	}

	/// resourceAmount_(1...8) --> [(5, 6, 2), (6, 0, 5)], [(7, 3, 5), (8, 0, 2)], ..., [(17, 1, 7)]
	pub fn get_resource_amount(&self, resource_idx: u8) -> u8 {
		let resource_idx = (min(resource_idx, 7) * 13) + 6;
		let (byte_idx, bit_idx) = InterpreterUtils::get_indices(resource_idx, 5);

		if bit_idx > 2 {
			let upper_bits = 8_u8.saturating_sub(bit_idx);
			let lower_bits = 6_u8.saturating_sub(upper_bits);
			self.inner
				.get_segmented_attribute_of_two(byte_idx as usize, &[upper_bits, lower_bits]) as u8
		} else {
			self.inner.get_segment_attribute_of_one(byte_idx as usize, bit_idx, 6)
		}
	}

	/// resourceAmount_(1...8) --> [(5, 6, 2), (6, 0, 5)], [(7, 3, 5), (8, 0, 2)], ..., [(17, 1, 7)]
	pub fn set_resource_amount(&mut self, resource_idx: u8, value: u8) {
		let resource_idx = (min(resource_idx, 7) * 13) + 6;
		let (byte_idx, bit_idx) = InterpreterUtils::get_indices(resource_idx, 5);

		// Only 7 bits max for resourceAmount_(1...8)
		let value = min(value, 0b0111_1111);

		if bit_idx > 2 {
			let upper_bits = 8_u8.saturating_sub(bit_idx);
			let lower_bits = 6_u8.saturating_sub(upper_bits);
			self.inner.set_segmented_attribute_of_two(
				byte_idx as usize,
				&[upper_bits, lower_bits],
				value as u16,
			)
		} else {
			self.inner.set_segment_attribute_of_one(byte_idx as usize, bit_idx, 6, value)
		}
	}

	/// amountOfCargoPlaceholders --> [(18, 0, 4)]
	pub fn get_amount_of_cargo_placeholders(&self) -> u8 {
		self.inner.get_segment_attribute_of_one(18, 0, 4)
	}

	/// amountOfCargoPlaceholders --> [(18, 0, 4)]
	pub fn set_amount_of_cargo_placeholders(&mut self, value: u8) {
		// Only 4 bits max for amountOfCargoPlaceholders
		let value = min(value, 0b1111);
		self.inner.set_segment_attribute_of_one(18, 0, 4, value);
	}

	/// (scanner|cargo|speed)UpgradeLvl --> [(18, 4, 2)], [(18, 6, 2)], [(19, 0, 2)]
	pub fn get_upgrade_lvl(&self, upgrade_type: UpgradeType) -> u8 {
		match upgrade_type {
			UpgradeType::Scanner => self.inner.get_segment_attribute_of_one(18, 4, 2),
			UpgradeType::Cargo => self.inner.get_segment_attribute_of_one(18, 6, 2),
			UpgradeType::Speed => self.inner.get_segment_attribute_of_one(19, 0, 2),
		}
	}

	/// scannerUpgradeLvl --> [(18, 4, 2)]
	pub fn set_scanner_upgrade_lvl(&mut self, upgrade_type: UpgradeType, value: u8) {
		// Only 2 bits max for (scanner|cargo|speed)UpgradeLvl
		let value = min(value, 0b0011);

		match upgrade_type {
			UpgradeType::Scanner => self.inner.set_segment_attribute_of_one(18, 4, 2, value),
			UpgradeType::Cargo => self.inner.set_segment_attribute_of_one(18, 6, 2, value),
			UpgradeType::Speed => self.inner.set_segment_attribute_of_one(19, 0, 2, value),
		}
	}

	/// isOnRoute --> [(19, 2, 1)]
	pub fn get_is_on_route(&self) -> bool {
		self.inner.get_segment_attribute_of_one(19, 2, 1) != 0
	}

	/// isOnRoute --> [(19, 2, 1)]
	pub fn set_is_on_route(&mut self, value: bool) {
		self.inner.set_segment_attribute_of_one(19, 2, 1, value as u8);
	}

	/// isHarvesting --> [(19, 3, 1)]
	pub fn get_is_harvesting(&self) -> bool {
		self.inner.get_segment_attribute_of_one(19, 3, 1) != 0
	}

	/// isHarvesting --> [(19, 3, 1)]
	pub fn set_is_harvesting(&mut self, value: bool) {
		self.inner.set_segment_attribute_of_one(19, 3, 1, value as u8);
	}

	/// isFinishedHarvesting --> [(19, 4, 1)]
	pub fn get_is_finished_harvesting(&self) -> bool {
		self.inner.get_segment_attribute_of_one(19, 4, 1) != 0
	}

	/// isFinishedHarvesting --> [(19, 4, 1)]
	pub fn set_is_finished_harvesting(&mut self, value: bool) {
		self.inner.set_segment_attribute_of_one(19, 4, 1, value as u8);
	}

	/// isOnTravelPoint --> [(19, 5, 1)]
	pub fn get_is_on_travel_point(&self) -> bool {
		self.inner.get_segment_attribute_of_one(19, 5, 1) != 0
	}

	/// isOnTravelPoint --> [(19, 5, 1)]
	pub fn set_is_on_travel_point(&mut self, value: bool) {
		self.inner.set_segment_attribute_of_one(19, 5, 1, value as u8);
	}

	/// freeResourceSlots --> [(19, 6, 2), (20, 0, 2)]
	pub fn get_free_resource_slots(&self) -> u8 {
		self.inner.get_segmented_attribute_of_two(19, &[2, 2]) as u8
	}

	/// freeResourceSlots --> [(19, 6, 2), (20, 0, 2)]
	pub fn set_free_resource_slots(&mut self, value: u8) {
		// Only 4 bits max for freeResourceSlots
		let value = min(value, 0b1111);
		self.inner.set_segmented_attribute_of_two(18, &[2, 2], value as u16);
	}

	/// distanceFlown --> [(20, 2, 6), (21, 0, 8)]
	pub fn get_distance_flown(&self) -> u16 {
		self.inner.get_segmented_attribute_of_two(20, &[6, 8])
	}

	/// distanceFlown --> [(20, 2, 6), (21, 0, 8)]
	pub fn set_distance_flown(&mut self, value: u16) {
		// Only 14 bits max for distanceFlown
		let value = min(value, 0b0011_1111_1111_1111);
		self.inner.set_segmented_attribute_of_two(20, &[6, 8], value);
	}

	/// TargetCoord(X/Y) --> [(22, 0, 8), (23, 0, 8), (24, 0, 8)], [(25, 0, 8), (26, 0, 8), (27, 0,
	/// 8)]
	pub fn get_target_coord(&self, coord: Coord) -> u32 {
		match coord {
			Coord::X => self.inner.get_segmented_attribute_of_three(22, &[8, 8]),
			Coord::Y => self.inner.get_segmented_attribute_of_three(25, &[8, 8]),
		}
	}

	/// TargetCoord(X/Y) --> [(22, 0, 8), (23, 0, 8), (24, 0, 8)], [(25, 0, 8), (26, 0, 8), (27, 0,
	/// 8)]
	pub fn set_target_coord(&mut self, coord: Coord, value: u32) {
		match coord {
			Coord::X => self.inner.set_segmented_attribute_of_three(22, &[8, 8], value),
			Coord::Y => self.inner.set_segmented_attribute_of_three(25, &[8, 8], value),
		}
	}

	/// timeFlownInBlocks --> [(28, 0, 8)]
	pub fn get_time_flown_in_blocks(&self) -> u8 {
		self.inner.get_segment_attribute_of_one(28, 0, 8)
	}

	/// timeFlownInBlocks --> [(28, 0, 8)]
	pub fn set_time_flown_in_blocks(&mut self, value: u8) {
		self.inner.set_segment_attribute_of_one(20, 0, 8, value);
	}

	/// Coord (X/Y) --> [(29, 0, 8), (30, 0, 8), (31, 0, 8)], [(32, 0, 8), (33, 0, 8), (34, 0, 8)]
	pub fn get_coord(&self, coord: Coord) -> u32 {
		match coord {
			Coord::X => self.inner.get_segmented_attribute_of_three(29, &[8, 8]),
			Coord::Y => self.inner.get_segmented_attribute_of_three(32, &[8, 8]),
		}
	}

	/// Coord (X/Y) --> [(29, 0, 8), (30, 0, 8), (31, 0, 8)], [(32, 0, 8), (33, 0, 8), (34, 0, 8)]
	pub fn set_coord(&mut self, coord: Coord, value: u32) {
		match coord {
			Coord::X => self.inner.set_segmented_attribute_of_three(29, &[8, 8], value),
			Coord::Y => self.inner.set_segmented_attribute_of_three(32, &[8, 8], value),
		}
	}
}
