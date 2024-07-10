mod moon;
mod ship;
mod unprospected_moon;

pub(super) use moon::*;
pub(super) use ship::*;
pub(super) use unprospected_moon::*;

use super::*;

pub(super) trait DnaInterpreter<'a, BlockNumber, T> {
	fn from_wrapper(wrap: &'a mut WrappedAvatar<BlockNumber>) -> T;
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub(super) enum Coord {
	X = 0,
	Y = 1,
}

struct InterpreterUtils;

impl InterpreterUtils {
	#[inline]
	fn get_indices(basis: u8, shift: u8) -> (u8, u8) {
		((basis / 8) + shift, basis % 8)
	}
}
