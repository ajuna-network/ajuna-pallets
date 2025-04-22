// Ajuna Node
// Copyright (C) 2022 BlogaTech AG
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

#![cfg_attr(not(feature = "std"), no_std)]

pub mod asset;
mod benchmarks;
pub mod error;
pub mod filter;
mod rules;
pub mod transition;

/// This contains all modules required for the runtime integration
/// of the gameplay logic into a SAGE instance.
pub mod prelude {
	pub use crate::{
		asset::{Asset, AssetId, AssetVariant, MachineSubVariant, MachineVariant, VariantType},
		benchmarks::GameBenchmarkHelper,
		error,
		filter::GameFilter,
		transition::{
			AssetType, CasinoAction, CasinoJamTransition, CasinoJamTransitionConfig, MachineType,
			MultiplierType, PlayerType, TokenType,
		},
	};
}
