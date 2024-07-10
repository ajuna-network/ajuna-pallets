use super::*;

use core::{
	cmp::min,
	ops::{Deref, DerefMut},
};

pub(crate) struct CaptainInterpreter<'a, BlockNumber> {
	inner: &'a mut WrappedAvatar<BlockNumber>,
}

impl<'a, BlockNumber> DnaInterpreter<'a, BlockNumber, CaptainInterpreter<'a, BlockNumber>>
	for ResourceInterpreter<'a, BlockNumber>
{
	fn from_wrapper(
		wrap: &'a mut WrappedAvatar<BlockNumber>,
	) -> CaptainInterpreter<'a, BlockNumber> {
		Self { inner: wrap }
	}
}

impl<'a, BlockNumber> Deref for CaptainInterpreter<'a, BlockNumber> {
	type Target = WrappedAvatar<BlockNumber>;

	fn deref(&self) -> &Self::Target {
		self.inner
	}
}

impl<'a, BlockNumber> DerefMut for CaptainInterpreter<'a, BlockNumber> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.inner
	}
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) enum CaptainAttribute {
	Luck = 0,
	Defensiveness = 1,
	Intelligence = 2,
	Charisma = 3,
	Empathy = 4,
	Bravery = 5,
	Aggression = 6,
	ScanRangeMultiplier = 7,
	ScanCooldownMultiplier = 8,
}

impl<'a, BlockNumber> CaptainInterpreter<'a, BlockNumber>
where
	BlockNumber: sp_runtime::traits::BlockNumber,
{
	/// leaderboardPoints --> [(5, 0, 8), (6, 0, 8)]
	pub fn get_leaderboard_points(&self) -> u16 {
		self.inner.get_segmented_attribute_of_two(5, &[8, 8])
	}

	/// leaderboardPoints --> [(5, 0, 8), (6, 0, 8)]
	pub fn set_leaderboard_points(&mut self, value: u16) {
		self.inner.set_segmented_attribute_of_two(5, &[8, 8], value)
	}

	/// totalMoonCount --> [(7, 0, 6)]
	pub fn get_total_moon_count(&self) -> u8 {
		self.inner.get_segmented_attribute_of_one(7, 0, 6)
	}

	/// totalMoonCount --> [(7, 0, 6)]
	pub fn set_total_moon_count(&mut self, value: u8) {
		// Only 6 bits max for totalMoonCount
		let value = min(value, 0b0011_1111);
		self.inner.set_segmented_attribute_of_one(7, 0, 6, value)
	}

	/// uniqueNebulasDetected --> [(7, 6, 2), (8, 0, 4)]
	pub fn get_unique_nebulas_detected(&self) -> u8 {
		self.inner.get_segmented_attribute_of_two(7, &[2, 4]) as u8
	}

	/// uniqueNebulasDetected --> [(7, 6, 2), (8, 0, 4)]
	pub fn set_unique_nebulas_detected(&mut self, value: u8) {
		// Only 6 bits max for uniqueNebulasDetected
		let value = min(value, 0b0011_1111);
		self.inner.set_segmented_attribute_of_two(7, &[2, 4], value as u16)
	}

	/// travelPointsBought --> [(8, 4, 4), (9, 0, 2)]
	pub fn get_travel_points_bought(&self) -> u8 {
		self.inner.get_segmented_attribute_of_two(8, &[4, 2]) as u8
	}

	/// travelPointsBought --> [(8, 4, 4), (9, 0, 2)]
	pub fn set_travel_points_bought(&mut self, value: u8) {
		// Only 6 bits max for travelPointsBought
		let value = min(value, 0b0011_1111);
		self.inner.set_segmented_attribute_of_two(8, &[4, 2], value as u16)
	}

	/// moonsSold --> [(9, 2, 6)]
	pub fn get_moons_sold(&self) -> u8 {
		self.inner.get_segmented_attribute_of_one(9, 2, 6)
	}

	/// moonsSold --> [(9, 2, 6)]
	pub fn set_moons_sold(&mut self, value: u8) {
		// Only 6 bits max for moonsSold
		let value = min(value, 0b0011_1111);
		self.inner.set_segmented_attribute_of_one(9, 2, 6, value)
	}

	/// moonsProspected --> [(10, 0, 6)]
	pub fn get_moons_prospected(&self) -> u8 {
		self.inner.get_segmented_attribute_of_one(10, 0, 6)
	}

	/// moonsProspected --> [(10, 0, 6)]
	pub fn set_moons_prospected(&mut self, value: u8) {
		// Only 6 bits max for moonsProspected
		let value = min(value, 0b0011_1111);
		self.inner.set_segmented_attribute_of_one(10, 0, 6, value)
	}

	/// stardustOnAccount --> [(10, 6, 2), (11, 0, 8), (12, 0, 3)]
	pub fn get_stardust_on_account(&self) -> u16 {
		self.inner.get_segmented_attribute_of_three(10, &[2, 3]) as u16
	}

	/// stardustOnAccount --> [(10, 6, 2), (11, 0, 8), (12, 0, 3)]
	pub fn set_stardust_on_account(&mut self, value: u16) {
		// Only 13 bits max for stardustOnAccount
		let value = min(value, 0b0001_1111_1111_1111);
		self.inner.set_segmented_attribute_of_three(10, &[2, 3], value as u32)
	}

	/// stardustGatheredAllTime --> [(12, 3, 5), (13, 0, 8), (14, 0, 1)]
	pub fn get_stardust_gathered_all_time(&self) -> u16 {
		self.inner.get_segmented_attribute_of_three(12, &[5, 1]) as u16
	}

	/// stardustGatheredAllTime --> [(12, 3, 5), (13, 0, 8), (14, 0, 1)]
	pub fn set_stardust_gathered_all_time(&mut self, value: u16) {
		// Only 14 bits max for stardustGatheredAllTime
		let value = min(value, 0b0011_1111_1111_1111);
		self.inner.set_segmented_attribute_of_three(12, &[5, 1], value as u32)
	}

	/// travelPointsSold --> [(14, 1, 6)]
	pub fn get_travel_points_sold(&self) -> u8 {
		self.inner.get_segmented_attribute_of_one(14, 1, 6)
	}

	/// travelPointsSold --> [(14, 1, 6)]
	pub fn set_travel_points_sold(&mut self, value: u8) {
		// Only 6 bits max for travelPointsSold
		let value = min(value, 0b0011_1111);
		self.inner.set_segmented_attribute_of_one(14, 1, 6, value)
	}

	/// radioLastUse --> [(15, 0, 8), ..., (22, 0, 8)]
	pub fn get_radio_last_use(&self) -> u64 {
		self.inner.get_segmented_attribute_of_eight(15, &[8, 8])
	}

	/// radioLastUse --> [(15, 0, 8), ..., (22, 0, 8)]
	pub fn set_radio_last_use(&mut self, value: u64) {
		self.inner.set_segmented_attribute_of_eight(15, &[8, 8], value)
	}

	/// (captain_attribute) --> [(23, 0, 3)], ..., [(26, 0, 3)]
	pub fn get_captain_attribute(&self, captain_attribute: CaptainAttribute) -> u8 {
		match captain_attribute {
			CaptainAttribute::Luck => self.inner.get_segmented_attribute_of_one(23, 0, 3),
			CaptainAttribute::Defensiveness => self.inner.get_segmented_attribute_of_one(23, 3, 3),
			CaptainAttribute::Intelligence =>
				self.inner.get_segmented_attribute_of_two(23, &[2, 1]) as u8,
			CaptainAttribute::Charisma => self.inner.get_segmented_attribute_of_one(24, 1, 3),
			CaptainAttribute::Empathy => self.inner.get_segmented_attribute_of_one(24, 4, 3),
			CaptainAttribute::Bravery =>
				self.inner.get_segmented_attribute_of_two(24, &[1, 2]) as u8,
			CaptainAttribute::Aggression => self.inner.get_segmented_attribute_of_one(25, 2, 3),
			CaptainAttribute::ScanRangeMultiplier =>
				self.inner.get_segmented_attribute_of_one(25, 5, 3),
			CaptainAttribute::ScanCooldownMultiplier =>
				self.inner.get_segmented_attribute_of_one(26, 0, 3),
		}
	}

	/// (captain_attribute) --> [(23, 0, 3)], ..., [(26, 0, 3)]
	pub fn set_captain_attribute(&mut self, captain_attribute: CaptainAttribute, value: u8) {
		// Only 3 bits max for (captain_attribute)
		let value = min(value, 0b0111);

		match captain_attribute {
			CaptainAttribute::Luck => self.inner.set_segmented_attribute_of_one(23, 0, 3, value),
			CaptainAttribute::Defensiveness =>
				self.inner.set_segmented_attribute_of_one(23, 3, 3, value),
			CaptainAttribute::Intelligence =>
				self.inner.set_segmented_attribute_of_two(23, &[2, 1], value as u16),
			CaptainAttribute::Charisma =>
				self.inner.set_segmented_attribute_of_one(24, 1, 3, value),
			CaptainAttribute::Empathy => self.inner.set_segmented_attribute_of_one(24, 4, 3, value),
			CaptainAttribute::Bravery =>
				self.inner.set_segmented_attribute_of_two(24, &[1, 2], value as u16),
			CaptainAttribute::Aggression =>
				self.inner.set_segmented_attribute_of_one(25, 2, 3, value),
			CaptainAttribute::ScanRangeMultiplier =>
				self.inner.set_segmented_attribute_of_one(25, 5, 3, value),
			CaptainAttribute::ScanCooldownMultiplier =>
				self.inner.set_segmented_attribute_of_one(26, 0, 3, value),
		}
	}

	/// successfulForges --> [(26, 3, 5), (27, 0, 2)]
	pub fn get_successful_forges(&self) -> u8 {
		self.inner.get_segmented_attribute_of_two(26, &[5, 2]) as u8
	}

	/// successfulForges --> [(26, 3, 5), (27, 0, 2)]
	pub fn set_successful_forges(&mut self, value: u8) {
		// Only 7 bits max for successfulForges
		let value = min(value, 0b0111_1111);
		self.inner.set_segmented_attribute_of_two(26, &[5, 2], value as u16)
	}

	/// failedForges --> [(27, 2, 6), (28, 0, 1)]
	pub fn get_failed_forges(&self) -> u8 {
		self.inner.get_segmented_attribute_of_two(27, &[6, 1]) as u8
	}

	/// failedForges --> [(27, 2, 6), (28, 0, 1)]
	pub fn set_failed_forges(&mut self, value: u8) {
		// Only 7 bits max for failedForges
		let value = min(value, 0b0111_1111);
		self.inner.set_segmented_attribute_of_two(27, &[6, 1], value as u16)
	}

	/// numberOfPerksImbued --> [(28, 1, 7)]
	pub fn get_number_of_perks_imbued(&self) -> u8 {
		self.inner.get_segmented_attribute_of_one(28, 1, 7)
	}

	/// numberOfPerksImbued --> [(28, 1, 7)]
	pub fn set_number_of_perks_imbued(&mut self, value: u8) {
		// Only 7 bits max for numberOfPerksImbued
		let value = min(value, 0b0111_1111);
		self.inner.set_segmented_attribute_of_one(28, 1, 7, value)
	}

	/// numberOfMoonsHarvested --> [(29, 0, 7)]
	pub fn get_number_of_moons_harvested(&self) -> u8 {
		self.inner.get_segmented_attribute_of_one(29, 0, 7)
	}

	/// numberOfMoonsHarvested --> [(29, 0, 7)]
	pub fn set_number_of_moons_harvested(&mut self, value: u8) {
		// Only 7 bits max for numberOfMoonsHarvested
		let value = min(value, 0b0111_1111);
		self.inner.set_segmented_attribute_of_one(29, 0, 7, value)
	}

	/// nebulasHarvested --> [(29, 7, 1), (30, 0, 6)]
	pub fn get_nebulas_harvested(&self) -> u8 {
		self.inner.get_segmented_attribute_of_two(29, &[1, 6]) as u8
	}

	/// nebulasHarvested --> [(29, 7, 1), (30, 0, 6)]
	pub fn set_nebulas_harvested(&mut self, value: u8) {
		// Only 7 bits max for nebulasHarvested
		let value = min(value, 0b0111_1111);
		self.inner.set_segmented_attribute_of_two(29, &[1, 6], value as u16)
	}
}
