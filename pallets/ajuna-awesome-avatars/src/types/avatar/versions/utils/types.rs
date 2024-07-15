use super::*;
use sp_std::ops::Range;

#[derive(Copy, Clone, Default)]
pub(crate) enum ByteType {
	#[default]
	Full = 0b1111_1111,
	High = 0b0000_1111,
	Low = 0b1111_0000,
}

impl ByteConvertible for ByteType {
	fn from_byte(byte: u8) -> Self {
		match byte {
			0xFF => Self::Full,
			0x0F => Self::High,
			0xF0 => Self::Low,
			_ => Self::default(),
		}
	}

	fn as_byte(&self) -> u8 {
		*self as u8
	}
}

#[derive(Copy, Clone, Debug, Default, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) enum HexType {
	#[default]
	X0 = 0b0000,
	X1 = 0b0001,
	X2 = 0b0010,
	X3 = 0b0011,
	X4 = 0b0100,
	X5 = 0b0101,
	X6 = 0b0110,
	X7 = 0b0111,
	X8 = 0b1000,
	X9 = 0b1001,
	XA = 0b1010,
	XB = 0b1011,
	XC = 0b1100,
	XD = 0b1101,
	XE = 0b1110,
	XF = 0b1111,
}

impl ByteConvertible for HexType {
	fn from_byte(byte: u8) -> Self {
		match byte {
			0x0 => Self::X0,
			0x1 => Self::X1,
			0x2 => Self::X2,
			0x3 => Self::X3,
			0x4 => Self::X4,
			0x5 => Self::X5,
			0x6 => Self::X6,
			0x7 => Self::X7,
			0x8 => Self::X8,
			0x9 => Self::X9,
			0xA => Self::XA,
			0xB => Self::XB,
			0xC => Self::XC,
			0xD => Self::XD,
			0xE => Self::XE,
			0xF => Self::XF,
			_ => Self::default(),
		}
	}

	fn as_byte(&self) -> u8 {
		*self as u8
	}
}

#[derive(Copy, Clone, Debug, Default, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) enum NibbleType {
	#[default]
	X0 = 0b0000,
	X1 = 0b0001,
	X2 = 0b0010,
	X3 = 0b0011,
	X4 = 0b0100,
	X5 = 0b0101,
	X6 = 0b0110,
	X7 = 0b0111,
}

impl ByteConvertible for NibbleType {
	fn from_byte(byte: u8) -> Self {
		match byte {
			0b0000 => Self::X0,
			0b0001 => Self::X1,
			0b0010 => Self::X2,
			0b0011 => Self::X3,
			0b0100 => Self::X4,
			0b0101 => Self::X5,
			0b0110 => Self::X6,
			0b0111 => Self::X7,
			_ => Self::default(),
		}
	}

	fn as_byte(&self) -> u8 {
		*self as u8
	}
}

impl Ranged for NibbleType {
	fn range() -> Range<usize> {
		0..8
	}
}
