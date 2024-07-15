use super::*;

use core::{
	cmp::min,
	ops::{Deref, DerefMut},
};

pub(crate) struct TempNebulaInterpreter<'a, BlockNumber> {
	inner: &'a mut WrappedAvatar<BlockNumber>,
}

impl<'a, BlockNumber> DnaInterpreter<'a, BlockNumber, TempNebulaInterpreter<'a, BlockNumber>>
	for TempNebulaInterpreter<'a, BlockNumber>
{
	fn from_wrapper(
		wrap: &'a mut WrappedAvatar<BlockNumber>,
	) -> TempNebulaInterpreter<'a, BlockNumber> {
		Self { inner: wrap }
	}
}

impl<'a, BlockNumber> Deref for TempNebulaInterpreter<'a, BlockNumber> {
	type Target = WrappedAvatar<BlockNumber>;

	fn deref(&self) -> &Self::Target {
		self.inner
	}
}

impl<'a, BlockNumber> DerefMut for TempNebulaInterpreter<'a, BlockNumber> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.inner
	}
}

impl<'a, BlockNumber> TempNebulaInterpreter<'a, BlockNumber>
where
	BlockNumber: sp_runtime::traits::BlockNumber,
{
	/// resourceType --> [(5, 0, 6)]
	pub fn get_resource_type(&self) -> u8 {
		self.inner.get_segmented_attribute_of_one(5, 0, 6)
	}

	/// resourceType --> [(5, 0, 6)]
	pub fn set_resource_type(&mut self, value: u8) {
		// Only 6 bits max for resourceType
		let value = min(value, 0b0011_1111);
		self.inner.set_segmented_attribute_of_one(5, 0, 6, value)
	}

	/// resourceAmount --> [(5, 6, 2), (6, 0, 5)]
	pub fn get_resource_amount(&self) -> u8 {
		self.inner.get_segmented_attribute_of_two(5, &[2, 5]) as u8
	}

	/// resourceAmount --> [(5, 6, 2), (6, 0, 5)]
	pub fn set_resource_amount(&mut self, value: u8) {
		// Only 7 bits max for resourceAmount
		let value = min(value, 0b0111_1111);
		self.inner.set_segmented_attribute_of_two(5, &[2, 5], value as u16)
	}

	/// isDepleted --> [(7, 0, 1)]
	pub fn get_is_depleted(&self) -> bool {
		self.inner.get_segmented_attribute_of_one(7, 0, 1) != 0
	}

	/// isDepleted --> [(7, 0, 1)]
	pub fn set_is_depleted(&mut self, value: bool) {
		self.inner.set_segmented_attribute_of_one(7, 0, 1, value as u8);
	}

	/// isActive --> [(7, 1, 1)]
	pub fn get_is_active(&self) -> bool {
		self.inner.get_segmented_attribute_of_one(7, 1, 1) != 0
	}

	/// isActive --> [(7, 1, 1)]
	pub fn set_is_active(&mut self, value: bool) {
		self.inner.set_segmented_attribute_of_one(7, 1, 1, value as u8);
	}

	/// isActive --> [(7, 2, 1)]
	pub fn get_is_ship_present(&self) -> bool {
		self.inner.get_segmented_attribute_of_one(7, 2, 1) != 0
	}

	/// isActive --> [(7, 2, 1)]
	pub fn set_is_ship_present(&mut self, value: bool) {
		self.inner.set_segmented_attribute_of_one(7, 2, 1, value as u8);
	}

	/// isShipHarvesting --> [(7, 3, 1)]
	pub fn get_is_ship_harvesting(&self) -> bool {
		self.inner.get_segmented_attribute_of_one(7, 3, 1) != 0
	}

	/// isShipHarvesting --> [(7, 3, 1)]
	pub fn set_is_ship_harvesting(&mut self, value: bool) {
		self.inner.set_segmented_attribute_of_one(7, 3, 1, value as u8);
	}

	/// isFinishedHarvesting --> [(7, 4, 1)]
	pub fn get_is_finished_harvesting(&self) -> bool {
		self.inner.get_segmented_attribute_of_one(7, 4, 1) != 0
	}

	/// isFinishedHarvesting --> [(7, 4, 1)]
	pub fn set_is_finished_harvesting(&mut self, value: bool) {
		self.inner.set_segmented_attribute_of_one(7, 4, 1, value as u8);
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
