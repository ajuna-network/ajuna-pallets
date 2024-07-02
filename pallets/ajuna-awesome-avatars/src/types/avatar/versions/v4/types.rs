use crate::{ByteConvertible, Ranged};
use sp_std::ops::Range;

#[derive(Copy, Clone, Debug, Default, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) enum ItemType {
	#[default]
	Celestial = 1,
	Construction = 2,
	Lifeform = 3,
	Resource = 4,
	Navigation = 5,
}

impl ByteConvertible for ItemType {
	fn from_byte(byte: u8) -> Self {
		match byte {
			1 => Self::Celestial,
			2 => Self::Construction,
			3 => Self::Lifeform,
			4 => Self::Resource,
			5 => Self::Navigation,
			_ => Self::default(),
		}
	}

	fn as_byte(&self) -> u8 {
		*self as u8
	}
}

#[derive(Copy, Clone, Debug, Default, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) enum CelestialItemType {
	#[default]
	UnprospectedMoon = 1,
	Moon = 2,
	TravelPoint = 3,
	Nebula = 4,
	TempNebula = 5,
}

impl ByteConvertible for CelestialItemType {
	fn from_byte(byte: u8) -> Self {
		match byte {
			1 => Self::UnprospectedMoon,
			2 => Self::Moon,
			3 => Self::TravelPoint,
			4 => Self::Nebula,
			5 => Self::TempNebula,
			_ => Self::default(),
		}
	}

	fn as_byte(&self) -> u8 {
		*self as u8
	}
}

impl Ranged for CelestialItemType {
	fn range() -> Range<usize> {
		1..6
	}
}

#[derive(Copy, Clone, Debug, Default, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) enum ConstructionItemType {
	#[default]
	Ship = 1,
}

impl ByteConvertible for ConstructionItemType {
	fn from_byte(byte: u8) -> Self {
		match byte {
			1 => Self::Ship,
			_ => Self::default(),
		}
	}

	fn as_byte(&self) -> u8 {
		*self as u8
	}
}

impl Ranged for ConstructionItemType {
	fn range() -> Range<usize> {
		1..2
	}
}

#[derive(Copy, Clone, Debug, Default, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) enum ResourceItemType {
	#[default]
	Resource = 1,
}

impl ByteConvertible for ResourceItemType {
	fn from_byte(byte: u8) -> Self {
		match byte {
			1 => Self::Resource,
			_ => Self::default(),
		}
	}

	fn as_byte(&self) -> u8 {
		*self as u8
	}
}

impl Ranged for ResourceItemType {
	fn range() -> Range<usize> {
		1..2
	}
}

#[derive(Copy, Clone, Debug, Default, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) enum LifeformItemType {
	#[default]
	Captain = 1,
	Cyoa = 2,
	ClusterMap = 3,
}

impl ByteConvertible for LifeformItemType {
	fn from_byte(byte: u8) -> Self {
		match byte {
			1 => Self::Captain,
			2 => Self::Cyoa,
			3 => Self::ClusterMap,
			_ => Self::default(),
		}
	}

	fn as_byte(&self) -> u8 {
		*self as u8
	}
}

impl Ranged for LifeformItemType {
	fn range() -> Range<usize> {
		1..4
	}
}

#[derive(Copy, Clone, Debug, Default, Ord, PartialOrd, Eq, PartialEq)]
pub(crate) enum NavigationItemType {
	#[default]
	Navigation = 1,
}

impl ByteConvertible for NavigationItemType {
	fn from_byte(byte: u8) -> Self {
		match byte {
			1 => Self::Navigation,
			_ => Self::default(),
		}
	}

	fn as_byte(&self) -> u8 {
		*self as u8
	}
}

impl Ranged for NavigationItemType {
	fn range() -> Range<usize> {
		1..2
	}
}
