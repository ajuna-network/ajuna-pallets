// Ajuna Node
// Copyright (C) 2022 BlogaTech AG

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.

// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use crate::{Config, Error};
use frame_support::{
	dispatch::DispatchResult,
	ensure,
	pallet_prelude::{ConstU32, Decode, Encode, MaxEncodedLen, TypeInfo},
	BoundedVec,
};

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, PartialEq)]
pub struct Season<BlockNumber, Balance> {
	pub max_tier_forges: u32,
	pub max_variations: u8,
	pub max_components: u8,
	pub min_sacrifices: u8,
	pub per_period: BlockNumber,
	pub periods: u16,
	pub fee: Balance,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Debug, PartialEq)]
pub struct SeasonStatus<SeasonId> {
	pub season_id: SeasonId,
	pub early: bool,
	pub active: bool,
	pub early_ended: bool,
}
impl<SeasonId> SeasonStatus<SeasonId> {
	pub(crate) fn is_in_season(&self) -> bool {
		self.early || self.active || self.early_ended
	}
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, PartialEq)]
pub struct SeasonMetadata {
	pub name: BoundedVec<u8, ConstU32<100>>,
	pub description: BoundedVec<u8, ConstU32<1_000>>,
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, PartialEq)]
pub struct SeasonSchedule<BlockNumber> {
	pub early_start: BlockNumber,
	pub start: BlockNumber,
	pub end: BlockNumber,
}

impl<BlockNumber> SeasonSchedule<BlockNumber>
where
	BlockNumber: PartialOrd,
{
	pub(crate) fn is_active(&self, now: BlockNumber) -> bool {
		now >= self.start && now <= self.end
	}

	pub(crate) fn is_early(&self, now: BlockNumber) -> bool {
		now >= self.early_start && now < self.start
	}
}
