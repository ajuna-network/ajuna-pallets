use super::*;

use core::{
	cmp::min,
	ops::{Deref, DerefMut},
};

pub(crate) struct NebulaInterpreter<'a, BlockNumber> {
	inner: &'a mut WrappedAvatar<BlockNumber>,
}

impl<'a, BlockNumber> DnaInterpreter<'a, BlockNumber, NebulaInterpreter<'a, BlockNumber>>
	for NebulaInterpreter<'a, BlockNumber>
{
	fn from_wrapper(
		wrap: &'a mut WrappedAvatar<BlockNumber>,
	) -> NebulaInterpreter<'a, BlockNumber> {
		Self { inner: wrap }
	}
}

impl<'a, BlockNumber> Deref for NebulaInterpreter<'a, BlockNumber> {
	type Target = WrappedAvatar<BlockNumber>;

	fn deref(&self) -> &Self::Target {
		self.inner
	}
}

impl<'a, BlockNumber> DerefMut for NebulaInterpreter<'a, BlockNumber> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.inner
	}
}

impl<'a, BlockNumber> NebulaInterpreter<'a, BlockNumber>
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
