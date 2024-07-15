use super::*;

use core::{
	cmp::min,
	ops::{Deref, DerefMut},
};

pub(crate) struct CyoaInterpreter<'a, BlockNumber> {
	inner: &'a mut WrappedAvatar<BlockNumber>,
}

impl<'a, BlockNumber> DnaInterpreter<'a, BlockNumber, CyoaInterpreter<'a, BlockNumber>>
	for CyoaInterpreter<'a, BlockNumber>
{
	fn from_wrapper(wrap: &'a mut WrappedAvatar<BlockNumber>) -> CyoaInterpreter<'a, BlockNumber> {
		Self { inner: wrap }
	}
}

impl<'a, BlockNumber> Deref for CyoaInterpreter<'a, BlockNumber> {
	type Target = WrappedAvatar<BlockNumber>;

	fn deref(&self) -> &Self::Target {
		self.inner
	}
}

impl<'a, BlockNumber> DerefMut for CyoaInterpreter<'a, BlockNumber> {
	fn deref_mut(&mut self) -> &mut Self::Target {
		self.inner
	}
}

impl<'a, BlockNumber> CyoaInterpreter<'a, BlockNumber>
where
	BlockNumber: sp_runtime::traits::BlockNumber,
{
	/// answerQ(1...30) --> [(5, 0, 3)], [(5, 3, 3)], ..., [(15, 7, 1), (16, 0, 2)]
	pub fn get_answer_question(&self, question_idx: u8) -> u8 {
		let question_idx = min(question_idx, 29) * 3;
		let (byte_idx, bit_idx) = InterpreterUtils::get_indices(question_idx, 5);

		if bit_idx > 5 {
			let upper_bits = 8_u8.saturating_sub(bit_idx);
			let lower_bits = 3_u8.saturating_sub(upper_bits);
			self.inner
				.get_segmented_attribute_of_two(byte_idx as usize, &[upper_bits, lower_bits]) as u8
		} else {
			self.inner.get_segmented_attribute_of_one(byte_idx as usize, bit_idx, 3)
		}
	}

	/// answerQ(1...30) --> [(5, 0, 3)], [(5, 3, 3)], ..., [(15, 7, 1), (16, 0, 2)]
	pub fn set_answer_question(&mut self, question_idx: u8, value: u8) {
		let question_idx = min(question_idx, 29) * 3;
		let (byte_idx, bit_idx) = InterpreterUtils::get_indices(question_idx, 5);

		// Only 3 bits max for answerQ(1...30)
		let value = min(value, 0b0000_0111);

		if bit_idx > 5 {
			let upper_bits = 8_u8.saturating_sub(bit_idx);
			let lower_bits = 3_u8.saturating_sub(upper_bits);
			self.inner.set_segmented_attribute_of_two(
				byte_idx as usize,
				&[upper_bits, lower_bits],
				value as u16,
			)
		} else {
			self.inner.set_segmented_attribute_of_one(byte_idx as usize, bit_idx, 3, value)
		}
	}
}
