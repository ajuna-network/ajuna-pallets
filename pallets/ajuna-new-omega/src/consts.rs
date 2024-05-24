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

use crate::types::CommanderId;

pub const MAX_COMMANDERS: CommanderId = 4;
pub const XP_PER_LOOT_CRATE: u32 = 10;
pub const XP_PER_RANKED_WIN: u32 = 25;

pub const MAX_FLEET_WINGS: usize = 4;
pub(crate) const MAX_ROUNDS: usize = 50;
pub(crate) const FIT_TO_STAT: u16 = 20;
pub(crate) const MAX_LOG_ENTRIES: u32 = MAX_ROUNDS as u32 * 4;
