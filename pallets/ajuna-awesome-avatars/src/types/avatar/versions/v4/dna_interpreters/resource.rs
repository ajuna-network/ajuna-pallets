use super::*;

use core::{
	cmp::min,
	ops::{Deref, DerefMut},
};

pub(crate) struct ResourceInterpreter<'a, BlockNumber> {
	inner: &'a mut WrappedAvatar<BlockNumber>,
}

impl<'a, BlockNumber> DnaInterpreter<'a, BlockNumber, ResourceInterpreter<'a, BlockNumber>>
	for ResourceInterpreter<'a, BlockNumber>
{
	fn from_wrapper(
		wrap: &'a mut WrappedAvatar<BlockNumber>,
	) -> ResourceInterpreter<'a, BlockNumber> {
		Self { inner: wrap }
	}
}

impl<'a, BlockNumber> Deref for ResourceInterpreter<'a, BlockNumber> {
	type Target = WrappedAvatar<BlockNumber>;

	fn deref(&self) -> &Self::Target {
		self.inner
	}
}

impl<'a, BlockNumber> DerefMut for ResourceInterpreter<'a, BlockNumber> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.inner
	}
}

impl<'a, BlockNumber> ResourceInterpreter<'a, BlockNumber>
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
}
