use super::*;

use core::{
	cmp::min,
	ops::{Deref, DerefMut},
};

pub(crate) struct ClusterMapInterpreter<'a, BlockNumber> {
	inner: &'a mut WrappedAvatar<BlockNumber>,
}

impl<'a, BlockNumber> DnaInterpreter<'a, BlockNumber, ClusterMapInterpreter<'a, BlockNumber>>
	for ClusterMapInterpreter<'a, BlockNumber>
{
	fn from_wrapper(
		wrap: &'a mut WrappedAvatar<BlockNumber>,
	) -> ClusterMapInterpreter<'a, BlockNumber> {
		Self { inner: wrap }
	}
}

impl<'a, BlockNumber> Deref for ClusterMapInterpreter<'a, BlockNumber> {
	type Target = WrappedAvatar<BlockNumber>;

	fn deref(&self) -> &Self::Target {
		self.inner
	}
}

impl<'a, BlockNumber> DerefMut for ClusterMapInterpreter<'a, BlockNumber> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.inner
	}
}

impl<'a, BlockNumber> ClusterMapInterpreter<'a, BlockNumber>
where
	BlockNumber: sp_runtime::traits::BlockNumber,
{
	/// MainCluster(1...10) --> [(5, 0, 1)], [(5, 1, 1)], ..., [(6, 1, 1)]
	pub fn get_main_cluster(&self, main_cluster_idx: u8) -> u8 {
		let main_cluster_idx = min(main_cluster_idx, 9);
		let (byte_idx, bit_idx) = InterpreterUtils::get_indices(main_cluster_idx, 5);

		self.inner.get_segmented_attribute_of_one(byte_idx as usize, bit_idx, 1)
	}

	/// MainCluster(1...10) --> [(5, 0, 1)], [(5, 1, 1)], ..., [(6, 1, 1)]
	pub fn set_main_cluster(&mut self, main_cluster_idx: u8, value: u8) {
		let main_cluster_idx = min(main_cluster_idx, 9);
		let (byte_idx, bit_idx) = InterpreterUtils::get_indices(main_cluster_idx, 5);

		// Only 1 bit max for MainCluster(1...10)
		let value = min(value, 0b0000_0001);

		self.inner.set_segmented_attribute_of_one(byte_idx as usize, bit_idx, 1, value)
	}

	/// Cluster(1...50) --> [(6, 2, 1)], [(6, 3, 1)], ..., [(12, 3, 1)]
	pub fn get_cluster(&self, cluster_idx: u8) -> u8 {
		let cluster_idx = min(cluster_idx, 49) + 2;
		let (byte_idx, bit_idx) = InterpreterUtils::get_indices(cluster_idx, 6);

		self.inner.get_segmented_attribute_of_one(byte_idx as usize, bit_idx, 1)
	}

	/// Cluster(1...50) --> [(6, 2, 1)], [(6, 3, 1)], ..., [(12, 3, 1)]
	pub fn set_cluster(&mut self, cluster_idx: u8, value: u8) {
		let cluster_idx = min(cluster_idx, 49) + 2;
		let (byte_idx, bit_idx) = InterpreterUtils::get_indices(cluster_idx, 6);

		// Only 1 bit max for Cluster(1...50)
		let value = min(value, 0b0000_0001);

		self.inner.set_segmented_attribute_of_one(byte_idx as usize, bit_idx, 1, value)
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
