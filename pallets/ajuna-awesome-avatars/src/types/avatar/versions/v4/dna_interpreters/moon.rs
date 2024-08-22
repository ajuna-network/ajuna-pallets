use super::*;

use core::{
	cmp::min,
	ops::{Deref, DerefMut},
};

pub(crate) struct MoonInterpreter<'a, BlockNumber> {
	inner: &'a mut WrappedAvatar<BlockNumber>,
}

impl<'a, BlockNumber> DnaInterpreter<'a, BlockNumber, MoonInterpreter<'a, BlockNumber>>
	for MoonInterpreter<'a, BlockNumber>
{
	fn from_wrapper(wrap: &'a mut WrappedAvatar<BlockNumber>) -> MoonInterpreter<'a, BlockNumber> {
		Self { inner: wrap }
	}
}

impl<'a, BlockNumber> Deref for MoonInterpreter<'a, BlockNumber> {
	type Target = WrappedAvatar<BlockNumber>;

	fn deref(&self) -> &Self::Target {
		self.inner
	}
}

impl<'a, BlockNumber> DerefMut for MoonInterpreter<'a, BlockNumber> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.inner
	}
}

impl<'a, BlockNumber> MoonInterpreter<'a, BlockNumber>
where
	BlockNumber: sp_runtime::traits::BlockNumber,
{
	/// visualMoonType --> [(5, 0, 4)]
	pub fn get_visual_moon_type(&self) -> u8 {
		self.inner.get_segmented_attribute_of_one(5, 0, 4)
	}

	/// visualMoonType --> [(5, 0, 4)]
	pub fn set_visual_moon_type(&mut self, value: u8) {
		// Only 4 bits max for visualMoonType
		let value = min(value, 0b_1111);
		self.inner.set_segmented_attribute_of_one(5, 0, 4, value);
	}

	/// moonType --> [(5, 4, 4)]
	pub fn get_moon_type(&self) -> u8 {
		self.inner.get_segmented_attribute_of_one(5, 4, 4)
	}

	/// moonType --> [(5, 4, 4)]
	pub fn set_moon_type(&mut self, value: u8) {
		// Only 4 bits max for moonType
		let value = min(value, 0b_1111);
		self.inner.set_segmented_attribute_of_one(5, 4, 4, value);
	}

	/// temperature --> [(6, 0, 4)]
	pub fn get_temperature(&self) -> u8 {
		self.inner.get_segmented_attribute_of_one(6, 0, 4)
	}

	/// temperature --> [(6, 0, 4)]
	pub fn set_temperature(&mut self, value: u8) {
		// Only 4 bits max for temperature
		let value = min(value, 0b_1111);
		self.inner.set_segmented_attribute_of_one(6, 0, 4, value);
	}

	/// humidity --> [(6, 4, 4)]
	pub fn get_humidity(&self) -> u8 {
		self.inner.get_segmented_attribute_of_one(6, 4, 4)
	}

	/// humidity --> [(6, 4, 4)]
	pub fn set_humidity(&mut self, value: u8) {
		// Only 4 bits max for humidity
		let value = min(value, 0b_1111);
		self.inner.set_segmented_attribute_of_one(6, 4, 4, value);
	}

	/// moonState --> [(7, 0, 4)]
	pub fn get_moon_state(&self) -> u8 {
		self.inner.get_segmented_attribute_of_one(7, 0, 4)
	}

	/// moonState --> [(7, 0, 4)]
	pub fn set_moon_state(&mut self, value: u8) {
		// Only 4 bits max for moonState
		let value = min(value, 0b1111);
		self.inner.set_segmented_attribute_of_one(7, 0, 4, value);
	}

	/// isPrimeMoon --> [(7, 4, 1)]
	pub fn get_is_prime_moon(&self) -> bool {
		self.inner.get_segmented_attribute_of_one(7, 4, 1) != 0
	}

	/// isPrimeMoon --> [(7, 4, 1)]
	pub fn set_is_prime_moon(&mut self, value: bool) {
		self.inner.set_segmented_attribute_of_one(7, 4, 1, value as u8);
	}

	/// resource_(1...50) --> [(8, 0, 1)], ..., [(14, 1, 1)]
	pub fn get_resource(&self, resource_idx: u8) -> u8 {
		let resource_idx = min(resource_idx, 49);
		let (byte_idx, bit_idx) = InterpreterUtils::get_indices(resource_idx, 8);

		self.inner.get_segmented_attribute_of_one(byte_idx as usize, bit_idx, 1)
	}

	/// resource_(1...50) --> [(8, 0, 1)], ..., [(14, 1, 1)]
	pub fn set_resource(&mut self, resource_idx: u8, value: u8) {
		let resource_idx = min(resource_idx, 49);
		let (byte_idx, bit_idx) = InterpreterUtils::get_indices(resource_idx, 8);

		// Only 1 bit max for resource_(1...50)
		let value = min(value, 0b1);
		self.inner.set_segmented_attribute_of_one(byte_idx as usize, bit_idx, 1, value);
	}

	/// perk_(1...50) --> [(14, 2, 1)], ..., [(20, 3, 1)]
	pub fn get_perk(&self, perk_idx: u8) -> u8 {
		let perk_idx = min(perk_idx, 49) + 2;
		let (byte_idx, bit_idx) = InterpreterUtils::get_indices(perk_idx, 14);

		self.inner.get_segmented_attribute_of_one(byte_idx as usize, bit_idx, 1)
	}

	/// perk_(1...50) --> [(14, 2, 1)], ..., [(20, 3, 1)]
	pub fn set_perk(&mut self, perk_idx: u8, value: u8) {
		let perk_idx = min(perk_idx, 49) + 2;
		let (byte_idx, bit_idx) = InterpreterUtils::get_indices(perk_idx, 14);

		// Only 1 bit max for perk_(1...50)
		let value = min(value, 0b1);
		self.inner.set_segmented_attribute_of_one(byte_idx as usize, bit_idx, 1, value);
	}

	/// isShipHarvesting --> [(20, 5, 1)]
	pub fn get_is_ship_harvesting(&self) -> bool {
		self.inner.get_segmented_attribute_of_one(20, 5, 1) != 0
	}

	/// isShipHarvesting --> [(20, 5, 1)]
	pub fn set_is_ship_harvesting(&mut self, value: bool) {
		self.inner.set_segmented_attribute_of_one(20, 5, 1, value as u8);
	}

	/// isFinishedHarvesting --> [(20, 6, 1)]
	pub fn get_is_finished_harvesting(&self) -> bool {
		self.inner.get_segmented_attribute_of_one(20, 6, 1) != 0
	}

	/// isFinishedHarvesting --> [(20, 6, 1)]
	pub fn set_is_finished_harvesting(&mut self, value: bool) {
		self.inner.set_segmented_attribute_of_one(20, 6, 1, value as u8);
	}

	/// isShipPresent --> [(20, 7, 1)]
	pub fn get_is_ship_present(&self) -> bool {
		self.inner.get_segmented_attribute_of_one(20, 7, 1) != 0
	}

	/// isShipPresent --> [(20, 7, 1)]
	pub fn set_is_ship_present(&mut self, value: bool) {
		self.inner.set_segmented_attribute_of_one(20, 7, 1, value as u8);
	}

	/// isTravelPointMintable --> [(21, 0, 1)]
	pub fn get_is_travel_point_mintable(&self) -> bool {
		self.inner.get_segmented_attribute_of_one(21, 0, 1) != 0
	}

	/// isTravelPointMintable --> [(21, 0, 1)]
	pub fn set_is_travel_point_mintable(&mut self, value: bool) {
		self.inner.set_segmented_attribute_of_one(21, 0, 1, value as u8);
	}

	/// mintedTravelPoints --> [(21, 1, 4)]
	pub fn get_minted_travel_points(&self) -> u8 {
		self.inner.get_segmented_attribute_of_one(21, 1, 4)
	}

	/// mintedTravelPoints --> [(21, 1, 4)]
	pub fn set_minted_travel_points(&mut self, value: u8) {
		self.inner.set_segmented_attribute_of_one(21, 1, 4, value);
	}

	/// blockMintsPeriod --> [(21, 5, 3), ..., (24, 0, 3)]
	pub fn get_block_mints_period(&self) -> u32 {
		self.inner.get_segmented_attribute_of_four(21, &[3, 3])
	}

	/// blockMintsPeriod --> [(21, 5, 3), ..., (24, 0, 3)]
	pub fn set_block_mints_period(&mut self, value: u32) {
		// Only 22 bits max for blockMintsPeriod
		let value = min(value, 0b0011_1111_1111_1111_1111_1111);
		self.inner.set_segmented_attribute_of_four(21, &[3, 3], value);
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
