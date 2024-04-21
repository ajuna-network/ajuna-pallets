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

pub mod v6;

use super::*;
use frame_support::traits::OnRuntimeUpgrade;

// The current storage version.
pub const STORAGE_VERSION: StorageVersion = StorageVersion::new(6);

const LOG_TARGET: &str = "runtime::ajuna-awesome-avatars::migration";

/// A unique identifier across all pallets.
///
/// This constant represents a unique identifier for the migrations of this pallet.
/// It helps differentiate migrations for this pallet from those of others. Note that we don't
/// directly pull the crate name from the environment, since that would change if the crate were
/// ever to be renamed and could cause historic migrations to run again.
pub const PALLET_MIGRATIONS_ID: &[u8; 32] = b"pallet-ajuna-awesome-avatars-mbm";
