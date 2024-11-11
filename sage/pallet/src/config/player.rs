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

use frame_support::pallet_prelude::*;
use sp_runtime::traits::Get;

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Debug, Default, PartialEq)]
pub enum StorageTier {
	#[default]
	One = 25,
	Two = 50,
	Three = 75,
	Four = 100,
	Five = 150,
	Max = 200,
}

impl StorageTier {
	pub(crate) fn upgrade(self) -> Self {
		match self {
			Self::One => Self::Two,
			Self::Two => Self::Three,
			Self::Three => Self::Four,
			Self::Four => Self::Five,
			Self::Five => Self::Max,
			Self::Max => Self::Max,
		}
	}
}

pub struct MaxAssetsPerPlayer;
impl Get<u32> for MaxAssetsPerPlayer {
	fn get() -> u32 {
		StorageTier::Max as u32
	}
}

pub struct MaxSeasons;
impl Get<u32> for MaxSeasons {
	fn get() -> u32 {
		1_000
	}
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Copy, Clone, Default, Debug, PartialEq)]
pub struct Locks {
	pub asset_transfer: bool,
	pub asset_trade: bool,
}

impl Locks {
	pub fn all_unlocked() -> Self {
		Self { asset_transfer: true, asset_trade: true }
	}
}

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Default, Debug, PartialEq)]
pub struct PlayerConfig {
	pub storage_tier: StorageTier,
	pub locks: Locks,
}

pub type Stat = u32;

#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Default, Debug, PartialEq)]
pub struct PlayerStats<BlockNumber> {
	pub minted_amount: Stat,
	pub forged_amount: Stat,
	pub bought_amount: Stat,
	pub sold_amount: Stat,
	pub first_mint: Option<BlockNumber>,
	pub latest_mint: Option<BlockNumber>,
	pub first_forge: Option<BlockNumber>,
	pub latest_forge: Option<BlockNumber>,
}
