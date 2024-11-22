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

use frame_support::{
	pallet_prelude::{ConstU32, Decode, Encode, MaxEncodedLen, TypeInfo},
	BoundedVec,
};

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Clone, Debug, PartialEq)]
pub enum SeasonScheduledAction<SeasonId> {
	EarlyStart(SeasonId),
	Start(SeasonId),
	End(SeasonId),
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Debug, PartialEq)]
pub struct SeasonStatus<SeasonId> {
	pub season_id: SeasonId,
	pub early: bool,
	pub active: bool,
	pub early_ended: bool,
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
