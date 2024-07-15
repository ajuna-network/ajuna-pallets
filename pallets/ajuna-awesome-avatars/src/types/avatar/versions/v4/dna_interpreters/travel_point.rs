use super::*;

use core::ops::{Deref, DerefMut};

pub(crate) struct TravelPointInterpreter<'a, BlockNumber> {
	inner: &'a mut WrappedAvatar<BlockNumber>,
}

impl<'a, BlockNumber> DnaInterpreter<'a, BlockNumber, TravelPointInterpreter<'a, BlockNumber>>
	for TravelPointInterpreter<'a, BlockNumber>
{
	fn from_wrapper(
		wrap: &'a mut WrappedAvatar<BlockNumber>,
	) -> TravelPointInterpreter<'a, BlockNumber> {
		Self { inner: wrap }
	}
}

impl<'a, BlockNumber> Deref for TravelPointInterpreter<'a, BlockNumber> {
	type Target = WrappedAvatar<BlockNumber>;

	fn deref(&self) -> &Self::Target {
		self.inner
	}
}

impl<'a, BlockNumber> DerefMut for TravelPointInterpreter<'a, BlockNumber> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.inner
	}
}

impl<'a, BlockNumber> TravelPointInterpreter<'a, BlockNumber>
where
	BlockNumber: sp_runtime::traits::BlockNumber,
{
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
