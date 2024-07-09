mod unprospected_moon;

pub(crate) use unprospected_moon::*;

use super::*;

pub(crate) trait DnaInterpreter<'a, BlockNumber, T> {
	fn from_wrapper(wrap: &'a mut WrappedAvatar<BlockNumber>) -> T;
}

#[derive(Copy, Clone, Eq, PartialEq)]
pub(crate) enum Coord {
	X = 0,
	Y = 1,
}
