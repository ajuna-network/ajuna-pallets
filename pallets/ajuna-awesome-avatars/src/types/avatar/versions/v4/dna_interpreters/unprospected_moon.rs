use super::*;

use core::{
	cmp::min,
	ops::{Deref, DerefMut},
};

pub(crate) struct UnprospectedMoonInterpreter<'a, BlockNumber> {
	inner: &'a mut WrappedAvatar<BlockNumber>,
}

impl<'a, BlockNumber> DnaInterpreter<'a, BlockNumber, UnprospectedMoonInterpreter<'a, BlockNumber>>
	for UnprospectedMoonInterpreter<'a, BlockNumber>
{
	fn from_wrapper(
		wrap: &'a mut WrappedAvatar<BlockNumber>,
	) -> UnprospectedMoonInterpreter<'a, BlockNumber> {
		Self { inner: wrap }
	}
}

impl<'a, BlockNumber> Deref for UnprospectedMoonInterpreter<'a, BlockNumber> {
	type Target = WrappedAvatar<BlockNumber>;

	fn deref(&self) -> &Self::Target {
		self.inner
	}
}

impl<'a, BlockNumber> DerefMut for UnprospectedMoonInterpreter<'a, BlockNumber> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.inner
	}
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) enum ExtractedStardustAmtIdx {
	One = 1,
	Three = 3,
	Five = 5,
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) enum DecisionType {
	Good = 0,
	Bad = 1,
}

impl<'a, BlockNumber> UnprospectedMoonInterpreter<'a, BlockNumber>
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

	/// extractedSdAmount_(1,3,5) --> [(5, 4, 1)], [(5, 5, 1)], [(5, 6, 1)]
	pub fn get_extracted_stardust_amt(&self, idx: ExtractedStardustAmtIdx) -> u8 {
		match idx {
			ExtractedStardustAmtIdx::One => self.inner.get_segmented_attribute_of_one(5, 4, 1),
			ExtractedStardustAmtIdx::Three => self.inner.get_segmented_attribute_of_one(5, 5, 1),
			ExtractedStardustAmtIdx::Five => self.inner.get_segmented_attribute_of_one(5, 6, 1),
		}
	}

	/// extractedSdAmount_(1,3,5) --> [(5, 4, 1)], [(5, 5, 1)], [(5, 6, 1)]
	pub fn set_extracted_stardust_amt(&mut self, idx: ExtractedStardustAmtIdx, value: u8) {
		// Only 1 bit max for extractedSdAmount_(1,3,5)
		let value = min(value, 0b1);
		match idx {
			ExtractedStardustAmtIdx::One =>
				self.inner.set_segmented_attribute_of_one(5, 4, 1, value),
			ExtractedStardustAmtIdx::Three =>
				self.inner.set_segmented_attribute_of_one(5, 5, 1, value),
			ExtractedStardustAmtIdx::Five =>
				self.inner.set_segmented_attribute_of_one(5, 6, 1, value),
		}
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

	/// isProspecting --> [(7, 5, 1)]
	pub fn get_is_prospecting(&self) -> bool {
		self.inner.get_segmented_attribute_of_one(7, 5, 1) != 0
	}

	/// isProspecting --> [(7, 5, 1)]
	pub fn set_is_prospecting(&mut self, value: bool) {
		self.inner.set_segmented_attribute_of_one(7, 5, 1, value as u8);
	}

	/// isProspected --> [(7, 6, 1)]
	pub fn get_is_prospected(&self) -> bool {
		self.inner.get_segmented_attribute_of_one(7, 6, 1) != 0
	}

	/// isProspected --> [(7, 6, 1)]
	pub fn set_is_prospected(&mut self, value: bool) {
		self.inner.set_segmented_attribute_of_one(7, 6, 1, value as u8);
	}

	/// isEventActive --> [(7, 7, 1)]
	pub fn get_is_event_active(&self) -> bool {
		self.inner.get_segmented_attribute_of_one(7, 7, 1) != 0
	}

	/// isEventActive --> [(7, 7, 1)]
	pub fn set_is_event_active(&mut self, value: bool) {
		self.inner.set_segmented_attribute_of_one(7, 7, 1, value as u8);
	}

	/// prospectingMinutesLeft --> [(8, 0, 8), (9, 0, 2)]
	pub fn get_prospecting_minutes_left(&self) -> u16 {
		self.inner.get_segmented_attribute_of_two(8, &[8, 2])
	}

	/// prospectingMinutesLeft --> [(8, 0, 8), (9, 0, 2)]
	pub fn set_prospecting_minutes_left(&mut self, value: u16) {
		// Only 10 bits max for prospectingMinutesLeft
		let value = min(value, 0b0011_1111_1111);
		self.inner.set_segmented_attribute_of_two(8, &[8, 2], value);
	}

	/// prospectingMinutesTotal --> [(9, 2, 6), (10, 0, 4)]
	pub fn get_prospecting_minutes_total(&self) -> u16 {
		self.inner.get_segmented_attribute_of_two(8, &[6, 4])
	}

	/// prospectingMinutesTotal --> [(9, 2, 6), (10, 0, 4)]
	pub fn set_prospecting_minutes_total(&mut self, value: u16) {
		// Only 10 bits max for prospectingMinutesTotal
		let value = min(value, 0b0011_1111_1111);
		self.inner.set_segmented_attribute_of_two(8, &[6, 4], value);
	}

	/// stardustPerHour --> [(10, 4, 4)]
	pub fn get_stardust_per_hour(&self) -> u8 {
		self.inner.get_segmented_attribute_of_one(8, 4, 4)
	}

	/// stardustPerHour --> [(10, 4, 4)]
	pub fn set_stardust_per_hour(&mut self, value: u8) {
		// Only 4 bits max for stardustPerHour
		let value = min(value, 0b1111);
		self.inner.set_segmented_attribute_of_one(8, 4, 4, value)
	}

	/// stardustOnMoon --> [(11, 0, 8)]
	pub fn get_stardust_on_moon(&self) -> u8 {
		self.inner.get_spec(SpecIdx::Byte7)
	}

	/// stardustOnMoon --> [(11, 0, 8)]
	pub fn set_stardust_on_moon(&mut self, value: u8) {
		self.inner.set_spec(SpecIdx::Byte7, value)
	}

	/// stardustOnMoon --> [(12, 0, 8)]
	pub fn get_stardust_sent_to_moon(&self) -> u8 {
		self.inner.get_spec(SpecIdx::Byte8)
	}

	/// stardustOnMoon --> [(12, 0, 8)]
	pub fn set_stardust_sent_to_moon(&mut self, value: u8) {
		self.inner.set_spec(SpecIdx::Byte8, value)
	}

	/// number(God|Bad)Decisions --> [(13, 0, 4)], [(13, 4, 4)]
	pub fn get_number_of_decisions(&self, decision_type: DecisionType) -> u8 {
		match decision_type {
			DecisionType::Good => self.inner.get_segmented_attribute_of_one(13, 0, 4),
			DecisionType::Bad => self.inner.get_segmented_attribute_of_one(13, 4, 4),
		}
	}

	/// number(Good|Bad)Decisions --> [(13, 0, 4)], [(13, 4, 4)]
	pub fn set_number_of_decisions(&mut self, decision_type: DecisionType, value: u8) {
		// Only 4 bits max for number(Good|Bad)Decisions
		let value = min(value, 0b1111);
		match decision_type {
			DecisionType::Good => self.inner.set_segmented_attribute_of_one(13, 0, 4, value),
			DecisionType::Bad => self.inner.set_segmented_attribute_of_one(13, 4, 4, value),
		}
	}

	/// influenceOf(Good|Bad)Decisions --> [(14, 0, 4)], [(14, 4, 4)]
	pub fn get_influence_of_decisions(&self, decision_type: DecisionType) -> u8 {
		match decision_type {
			DecisionType::Good => self.inner.get_segmented_attribute_of_one(14, 0, 4),
			DecisionType::Bad => self.inner.get_segmented_attribute_of_one(14, 4, 4),
		}
	}

	/// influenceOf(Good|Bad)Decisions --> [(14, 0, 4)], [(14, 4, 4)]
	pub fn set_influence_of_decisions(&mut self, decision_type: DecisionType, value: u8) {
		// Only 4 bits max for influenceOf(Good|Bad)Decisions
		let value = min(value, 0b1111);
		match decision_type {
			DecisionType::Good => self.inner.set_segmented_attribute_of_one(14, 0, 4, value),
			DecisionType::Bad => self.inner.set_segmented_attribute_of_one(14, 4, 4, value),
		}
	}

	/// nextEventAt --> [(15, 0, 8), ..., (22, 0, 8)]
	pub fn get_next_event_at(&self) -> u64 {
		self.inner.get_segmented_attribute_of_eight(15, &[8, 8])
	}

	/// nextEventAt --> [(15, 0, 8), ..., (22, 0, 8)]
	pub fn set_next_event_at(&mut self, value: u64) {
		self.inner.set_segmented_attribute_of_eight(15, &[8, 8], value)
	}

	/// eventLength --> [(23, 0, 6)]
	pub fn get_event_length(&self) -> u8 {
		self.inner.get_segmented_attribute_of_one(23, 0, 6)
	}

	/// eventLength --> [(23, 0, 6)]
	pub fn set_event_length(&mut self, value: u8) {
		// Only 6 bits max for eventLength
		let value = min(value, 0b0011_1111);
		self.inner.set_segmented_attribute_of_one(23, 0, 6, value)
	}

	/// eventType --> [(23, 6, 2), (24, 0, 4)]
	pub fn get_event_type(&self) -> u8 {
		self.inner.get_segmented_attribute_of_two(23, &[2, 4]) as u8
	}

	/// eventType --> [(23, 6, 2), (24, 0, 4)]
	pub fn set_event_type(&mut self, value: u8) {
		// Only 6 bits max for eventType
		let value = min(value, 0b0011_1111);
		self.inner.set_segmented_attribute_of_two(23, &[2, 4], value as u16)
	}

	/// harvestingTime --> [(25, 0, 8), (26, 0, 8), (27, 0, 8)]
	pub fn get_harvesting_time(&self) -> u32 {
		self.inner.get_segmented_attribute_of_three(23, &[8, 8])
	}

	/// harvestingTime --> [(25, 0, 8), (26, 0, 8), (27, 0, 8)]
	pub fn set_harvesting_time(&mut self, value: u32) {
		self.inner.set_segmented_attribute_of_three(23, &[8, 8], value)
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
